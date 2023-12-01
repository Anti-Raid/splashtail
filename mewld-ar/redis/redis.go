// Redis manager for mewld
package redis

import (
	"context"
	"encoding/json"
	"mewld/config"
	"mewld/proc"
	"os"
	"reflect"
	"strconv"
	"syscall"

	log "github.com/sirupsen/logrus"

	"github.com/go-redis/redis/v9"
)

// A Handler for redis
type RedisHandler struct {
	Ctx   context.Context
	Redis *redis.Client
}

func CreateHandler(config config.CoreConfig) RedisHandler {
	ctx := context.Background()

	rdb := redis.NewClient(&redis.Options{
		Addr:        config.Redis,
		Password:    "", // no password set
		DB:          0,  // use default DB
		ReadTimeout: -1,
	})

	status := rdb.Ping(ctx)

	if status.Err() != nil {
		log.Fatal("Redis error: ", status.Err())
	}

	return RedisHandler{
		Ctx:   ctx,
		Redis: rdb,
	}
}

type LauncherCmd struct {
	Scope     string         `json:"scope"`
	Action    string         `json:"action"`
	Args      map[string]any `json:"args,omitempty"`
	CommandId string         `json:"command_id,omitempty"`
	Output    any            `json:"output,omitempty"`
	Data      map[string]any `json:"data,omitempty"` // Used in action logs
}

type status struct {
	Active    bool     `json:"active"`
	Status    string   `json:"status"`
	Name      string   `json:"name"`
	StartedAt int64    `json:"started_at"`
	ShardList []uint64 `json:"shard_list"`
}

type numproc struct {
	Clusters int    `json:"clusters"`
	Shards   uint64 `json:"shards"`
}

func (r *RedisHandler) Start(il *proc.InstanceList) {
	// Start pubsub
	pubsub := r.Redis.Subscribe(r.Ctx, il.Config.RedisChannel)

	defer pubsub.Close()

	// Start listening for messages
	for msg := range pubsub.Channel() {
		log.Debug("Got redis message: ", msg.Payload)

		var cmd LauncherCmd

		err := json.Unmarshal([]byte(msg.Payload), &cmd)

		if err != nil {
			log.Error("Could not unmarshal message: ", err, ": ", msg.Payload)
			continue
		}

		if cmd.Scope != "launcher" {
			continue
		}

		switch cmd.Action {
		case "diag":
			if str, ok := cmd.Output.(string); ok {
				log.Info(str)

				var diagPayload proc.DiagResponse

				err := json.Unmarshal([]byte(str), &diagPayload)

				if err != nil {
					log.Error("Could not unmarshal diag message: ", err, ": ", str)
					continue
				}

				proc.DiagChannel <- diagPayload
			} else {
				log.Error("Diagnostic message parse error: ", cmd.Output)
			}
		case "action_logs":
			go il.ActionLog(cmd.Data)
		case "restartproc":
			log.Info("Restarting process: ", cmd.CommandId)
			il.Acknowledge(cmd.CommandId)
			il.KillAll()
			os.Exit(1)
		case "launch_next":
			// Get cluster id from args
			typeOfId := reflect.TypeOf(cmd.Args["id"])

			log.Info("Got launch_next command for cluster ", cmd.Args["id"], " (", typeOfId, ")")

			clusterId, ok := cmd.Args["id"].(float64)

			if !ok {
				log.Error("Could not get cluster id from args: ", cmd.Args["id"])

				// Continue if its roll restarting, we cant continue
				if il.RollRestarting {
					continue
				}
			} else {
				instance := il.InstanceByID(int(clusterId))

				if instance == nil {
					log.Error("Could not find instance with id: ", clusterId)
					continue
				}

				instance.LaunchedFully = true
			}

			if il.RollRestarting {
				// Push to proc.RollRestartChannel
				proc.RollRestartChannel <- int(clusterId)
				continue
			}

			il.StartNext()
		case "rollingrestart":
			go func() {
				il.Acknowledge(cmd.CommandId)
				il.RollingRestart()
			}()
		case "statuses":
			payload := map[string]status{}

			for _, i := range il.Instances {
				statusStruct := status{
					Active:    i.Active,
					Name:      il.Cluster(i).Name,
					StartedAt: i.StartedAt.Unix(),
					ShardList: i.Shards,
				}
				payload[strconv.Itoa(i.ClusterID)] = statusStruct
			}

			il.SendMessage(cmd.CommandId, payload, "bot", "")
		case "shutdown":
			log.Warn("Got request to shutdown (hopefully you have systemctl)")
			il.Acknowledge(cmd.CommandId)
			il.KillAll()
			syscall.Kill(syscall.Getpid(), syscall.SIGINT)
		case "stop":
			typeOfId := reflect.TypeOf(cmd.Args["id"])

			log.Info("Got stop command for cluster ", cmd.Args["id"], " (", typeOfId, ")")

			clusterId, ok := cmd.Args["id"].(float64)

			if !ok {
				log.Error("Could not get cluster id from args: ", cmd.Args["id"])
				continue
			}

			for _, i := range il.Instances {
				if i.ClusterID == int(clusterId) {
					if !i.Active {
						log.Error("Instance is not active")
						continue
					}

					il.Acknowledge(cmd.CommandId)

					err := il.Stop(i)

					if err != proc.StopCodeNormal {
						log.Error("Could not stop instance: ", err)
						continue
					}

					break
				}
			}
		case "start":
			typeOfId := reflect.TypeOf(cmd.Args["id"])

			log.Info("Got start command for cluster ", cmd.Args["id"], " (", typeOfId, ")")

			clusterId, ok := cmd.Args["id"].(float64)

			if !ok {
				log.Error("Could not get cluster id from args: ", cmd.Args["id"])
				continue
			}

			for _, i := range il.Instances {
				if i.ClusterID == int(clusterId) {
					il.Acknowledge(cmd.CommandId)

					il.Start(i)
					break
				}
			}

		case "restart":
			typeOfId := reflect.TypeOf(cmd.Args["id"])

			log.Info("Got restart command for cluster ", cmd.Args["id"], " (", typeOfId, ")")

			clusterId, ok := cmd.Args["id"].(float64)

			if !ok {
				log.Error("Could not get cluster id from args: ", cmd.Args["id"])
				continue
			}

			for _, i := range il.Instances {
				if i.ClusterID == int(clusterId) {
					if !i.Active {
						log.Error("Instance is not active")
						continue
					}

					il.Acknowledge(cmd.CommandId)

					i.Lock(il, "Redis.restart", false)

					err := il.Stop(i)

					if err == proc.StopCodeNormal {
						il.Start(i)
					} else {
						log.Error("Could not stop instance: ", err)
					}

					i.Unlock()

					break
				}
			}
		case "num_processes":
			payload := numproc{
				Clusters: len(il.Instances),
				Shards:   il.ShardCount,
			}

			il.SendMessage(cmd.CommandId, payload, "bot", "")
		default:
			log.Error("Unknown action: ", cmd.Action, ": ", cmd.Args)
		}
	}
}

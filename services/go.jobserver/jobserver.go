package main

import (
	"fmt"
	"net/http"
	"os"
	"os/exec"
	"os/signal"
	"strconv"
	"strings"
	"syscall"

	"github.com/bwmarrin/discordgo"
	"github.com/go-chi/chi/v5"
	"github.com/infinitybotlist/eureka/jsonimpl"
	"go.jobserver/bgtasks"
	"go.jobserver/core"
	"go.jobserver/state"
	"go.std/config"
	"go.std/mewld_web"
	"go.std/mewldresponder"
	"go.std/utils"
	"go.uber.org/zap"
	"gopkg.in/yaml.v3"

	mconfig "github.com/cheesycod/mewld/config"
	mloader "github.com/cheesycod/mewld/loader"
	mproc "github.com/cheesycod/mewld/proc"
	mutils "github.com/cheesycod/mewld/utils"
)

func CreateJobServer() {
	// Set state of all pending tasks to 'failed'
	_, err := state.Pool.Exec(state.Context, "UPDATE tasks SET state = $1 WHERE state = $2", "failed", "pending")

	if err != nil {
		panic(err)
	}

	state.AnimusMagicClient.OnRequest = core.AnimusOnRequest

	// Start listening
	go state.AnimusMagicClient.ListenOnce(
		state.Context,
		state.Rueidis,
		state.Logger,
	)

	// Resume ongoing tasks
	go core.Resume()

	// Start all background tasks
	go bgtasks.StartAllTasks()
}

func CreateClusters() {
	state.SetupBase()

	wmldF, err := os.ReadFile("data/mewld/botv2-" + config.CurrentEnv + ".yaml")

	if err != nil {
		panic(err)
	}

	var wmldConfig mconfig.CoreConfig

	err = yaml.Unmarshal(wmldF, &wmldConfig)

	if err != nil {
		panic(err)
	}

	// Load mewld bot
	mldF, err := os.ReadFile("data/mewld/jobs-" + config.CurrentEnv + ".yaml")

	if err != nil {
		panic(err)
	}

	var mldConfig mconfig.CoreConfig

	err = yaml.Unmarshal(mldF, &mldConfig)

	if err != nil {
		panic(err)
	}

	state.Logger.Info("Setting up mewld")

	mldConfig.Proxy = state.Config.Meta.Proxy.Parse()
	mldConfig.Token = state.Config.DiscordAuth.Token
	mldConfig.Oauth = mconfig.Oauth{
		ClientID:     state.Config.DiscordAuth.ClientID,
		ClientSecret: state.Config.DiscordAuth.ClientSecret,
		RedirectURL:  state.Config.DiscordAuth.MewldRedirect,
	}

	if mldConfig.Redis == "" {
		mldConfig.Redis = state.Config.Meta.RedisURL.Parse()
	}

	if mldConfig.Redis != state.Config.Meta.RedisURL.Parse() {
		state.Logger.Warn("Redis URL in mewld.yaml does not match the one in config.yaml")
	}

	for _, clusterName := range wmldConfig.Names {
		var i uint64
		for i < wmldConfig.PerCluster {
			mldConfig.Names = append(mldConfig.Names, clusterName+"@"+strconv.FormatUint(i, 10))
			i++
		}
	}

	webh, err := utils.ParseWebhookURL(state.Config.Wafflepaw.StatusWebhook)

	if err != nil {
		state.Logger.Fatal("Error parsing webhook URL", zap.Error(err))
	}

	il, rh, err := mloader.Load(&mldConfig, &mproc.LoaderData{
		Start: func(l *mproc.InstanceList, i *mproc.Instance, cm *mproc.ClusterMap) error {
			cmd := exec.Command(
				os.Args[0],
				"jobs.node",
				mutils.ToPyListUInt64(i.Shards),
				strconv.Itoa(i.ClusterID),
				cm.Name,
				mldConfig.RedisChannel,
				mutils.UInt64ToString(l.ShardCount),
			)

			cmd.Stdout = os.Stdout
			cmd.Stderr = os.Stderr

			env := os.Environ()

			env = append(env, "MEWLD_CHANNEL="+l.Config.RedisChannel)
			env = append(env, "REDIS_URL="+state.Config.Meta.RedisURL.Parse())

			cmd.Env = env
			cmd.Dir = l.Dir

			i.Command = cmd

			// Spawn process
			return cmd.Start()
		},
		OnActionLog: func(payload map[string]any) error {
			// Send webhook
			go func() {
				payloadStr := strings.Builder{}

				for k, v := range payload {
					payloadStr.WriteString(k + ": " + fmt.Sprint(v) + "\n")
				}

				_, err := state.Discord.WebhookExecute(
					webh.ID,
					webh.Token,
					false,
					&discordgo.WebhookParams{
						Content: "@everyone **MEWLD ALERT [jobserver]**\n" + payloadStr.String(),
					},
				)

				if err != nil {
					state.Logger.Error("Error sending webhook", zap.Error(err))
				}
			}()

			return nil
		},
	})

	if err != nil {
		panic(err)
	}

	defer func() {
		a := recover()

		if a != nil {
			il.KillAll()
		}
	}()

	r := chi.NewMux()

	mewld_web.SetState(state.Config.Meta.DPSecret)
	r.Mount("/mewld", mewld_web.CreateServer(mewld_web.WebData{
		RedisHandler: rh,
		InstanceList: il,
	}))

	// Tableflip not supported
	state.Logger.Warn("Tableflip not supported on this platform, this is not a production-capable server.")
	err = http.ListenAndServe(":"+strconv.Itoa(state.Config.Meta.JobserverPort.Parse()), r)

	if err != nil {
		il.KillAll()
		state.Logger.Fatal("Error binding to socket", zap.Error(err))
	}
}

func LaunchJobserver() {
	state.CurrentOperationMode = "jobs"

	// Read cmd args
	if len(os.Args) < 7 {
		panic("Not enough arguments. Expected <cmd> jobs.node <shards> <clusterID> <clusterName> <redisChannel> <shard count>")
	}

	shardsStr := os.Args[2]

	var shards []uint16

	err := jsonimpl.Unmarshal([]byte(shardsStr), &shards)
	if err != nil {
		panic(err)
	}

	state.Shard = shards[0]

	clusterId := os.Args[3]
	clusterIdInt, err := strconv.Atoi(clusterId)
	if err != nil {
		panic(err)
	}

	state.ClusterID = uint16(clusterIdInt)

	clusterName := os.Args[4]
	state.ClusterName = clusterName

	redisChannel := os.Args[5]

	shardCount := os.Args[6]

	shardCountInt, err := strconv.Atoi(shardCount)

	if err != nil {
		panic(err)
	}

	state.ShardCount = uint16(shardCountInt)

	state.Setup()

	state.Logger = state.Logger.With(zap.Uint16("shard", state.Shard), zap.Int("clusterId", clusterIdInt), zap.String("clusterName", clusterName))

	state.Logger.Info("Starting node")

	state.MewldResponder = &mewldresponder.MewldResponder{
		ClusterID:   state.ClusterID,
		ClusterName: state.ClusterName,
		Shards:      shards,
		Channel:     redisChannel,
		OnDiag: func(payload *mewldresponder.MewldDiagPayload) (*mewldresponder.MewldDiagResponse, error) {
			data := []mewldresponder.MewldDiagShardHealth{
				{
					ShardID: state.Shard,
					Up:      true, // TODO: Check if shard is up once we add dgo
					Latency: 0,    // TODO: Get shard latency once we add dgo
					Guilds:  0,    // TODO: Get shard guild count once we add dgo
					Users:   0,    // TODO: Get shard user count once we add dgo
				},
			}

			return &mewldresponder.MewldDiagResponse{
				ClusterID: state.ClusterID,
				Nonce:     payload.Nonce,
				Data:      data,
			}, nil
		},
	}

	CreateJobServer()

	// Load jobs
	bgtasks.StartAllTasks()

	// Handle mewld by starting ping checks and sending launch_next
	go func() {
		err := state.MewldResponder.LaunchNext(state.Context, state.Rueidis, state.Logger)

		if err != nil {
			state.Logger.Fatal("Error sending launch_next command", zap.Error(err))
			return
		}

		state.Logger.Info("Sent launch_next command")
	}()

	go state.MewldResponder.Listen(state.Context, state.Rueidis, state.Logger)

	// Wait until signal is received
	c := make(chan os.Signal, 1)

	signal.Notify(c, syscall.SIGTERM, syscall.SIGINT, syscall.SIGHUP)

	<-c
}

func main() {
	if len(os.Args) > 2 {
		if os.Args[1] == "jobs.node" {
			LaunchJobserver()
			return
		}
	}

	CreateClusters()
}

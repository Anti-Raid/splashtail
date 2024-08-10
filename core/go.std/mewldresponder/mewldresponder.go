package mewldresponder

import (
	"context"
	"errors"
	"fmt"
	"strings"
	"time"

	mredis "github.com/cheesycod/mewld/redis"
	"github.com/infinitybotlist/eureka/jsonimpl"
	"github.com/redis/rueidis"
	"go.uber.org/zap"
)

type MewldResponder struct {
	ClusterID             uint16
	ClusterName           string
	Shards                []uint16
	Channel               string
	OnDiag                func(p *MewldDiagPayload) (*MewldDiagResponse, error)
	OnAllClustersLaunched func() error
	OnLaunchNext          func() error
	OnLauncherCmd         func(cmd mredis.LauncherCmd) error
}

// ListenOnce starts listening for messages from redis
//
// This is *blocking* and should be run in a goroutine
func (c *MewldResponder) ListenOnce(ctx context.Context, r rueidis.Client, l *zap.Logger) error {
	return r.Dedicated(
		func(redis rueidis.DedicatedClient) error {
			return redis.Receive(ctx, redis.B().Subscribe().Channel(c.Channel).Build(), func(msg rueidis.PubSubMessage) {
				if len(msg.Message) == 0 {
					return
				}

				if msg.Message[0] != '{' {
					return // TODO: support this
				}

				if strings.Contains(msg.Message, "\"diag\":true") {
					// Hack but its performant
					var payload MewldDiagPayload

					err := jsonimpl.Unmarshal([]byte(msg.Message), &payload)

					if err != nil {
						l.Error("[mewld] error unmarshalling diag message", zap.Error(err))
						return
					}

					if payload.ClusterID != c.ClusterID {
						return
					}

					l.Info("[mewld] received diag request")

					if c.OnDiag != nil {
						resp, err := c.OnDiag(&payload)

						if err != nil {
							l.Error("[mewld] error running OnDiag", zap.Error(err))
							return
						}

						bytes, err := jsonimpl.Marshal(resp)

						if err != nil {
							l.Error("[mewld] error marshalling diag response", zap.Error(err))
							return
						}

						lcmd := mredis.LauncherCmd{
							Scope:  "launcher",
							Action: "diag",
							Output: string(bytes),
						}

						bytes, err = jsonimpl.Marshal(lcmd)

						if err != nil {
							l.Error("[mewld] error marshalling diag response", zap.Error(err))
							return
						}

						err = r.Do(
							ctx,
							r.B().Publish().Channel(c.Channel).Message(string(bytes)).Build(),
						).Error()

						if err != nil {
							l.Error("[mewld] error sending diag response", zap.Error(err))
						}
					}

					return
				}

				bytesData := []byte(msg.Message)

				var launcherData mredis.LauncherCmd

				err := jsonimpl.Unmarshal(bytesData, &launcherData)

				if err != nil {
					l.Error("[mewld] error unmarshalling message", zap.Error(err))
					return
				}

				switch launcherData.Action {
				case "all_clusters_launched":
					if c.OnAllClustersLaunched != nil {
						err := c.OnAllClustersLaunched()

						if err != nil {
							l.Error("[mewld] error running OnAllClustersLaunched", zap.Error(err))
						}
					}
				case "launch_next":
					if c.OnLaunchNext != nil {
						err := c.OnLaunchNext()

						if err != nil {
							l.Error("[mewld] error running OnLaunchNext", zap.Error(err))
						}
					}
				}

				if c.OnLauncherCmd != nil {
					err := c.OnLauncherCmd(launcherData)

					if err != nil {
						l.Error("[mewld] error running OnLauncherCmd", zap.Error(err))
					}
				}
			})
		},
	)
}

// Listen starts listening for messages from redis
// and restarts the listener if it dies
func (c *MewldResponder) Listen(ctx context.Context, redis rueidis.Client, l *zap.Logger) error {
	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		default:
			err := c.ListenOnce(ctx, redis, l)

			if errors.Is(err, context.Canceled) {
				return nil
			}

			if err != nil {
				l.Error("[mewld] error listening to redis", zap.Error(err))
				time.Sleep(1 * time.Second)
			}
		}
	}
}

// Sends the launch_next command
func (c *MewldResponder) LaunchNext(ctx context.Context, redis rueidis.Client, l *zap.Logger) error {
	launchNextCmd := mredis.LauncherCmd{
		Scope:  "launcher",
		Action: "launch_next",
		Args: map[string]any{
			"id": c.ClusterID,
		},
	}

	bytes, err := jsonimpl.Marshal(launchNextCmd)

	if err != nil {
		return fmt.Errorf("error marshalling launch_next command: %w", err)
	}

	err = redis.Do(
		ctx,
		redis.B().Publish().Channel(c.Channel).Message(string(bytes)).Build(),
	).Error()

	if err != nil {
		return fmt.Errorf("error sending launch_next command: %w", err)
	}

	return nil
}

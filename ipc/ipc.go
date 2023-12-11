package ipc

import (
	"slices"
	"splashtail/ipc/core"
	"splashtail/ipc/endpoints/create_task"
	"splashtail/ipc/endpoints/get_task"
	"splashtail/state"

	mredis "github.com/cheesycod/mewld/redis"

	jsoniter "github.com/json-iterator/go"
	"github.com/redis/go-redis/v9"
	"go.uber.org/zap"
)

var json = jsoniter.ConfigFastest

var ipcEvents = map[string]core.IPC{
	"get_task":    get_task.GetTask,
	"create_task": create_task.CreateTask,
}

var IpcDone bool

func Start() {
	defer func() {
		if r := recover(); r != nil {
			state.Logger.Error("IPC error: ", zap.Any("error", r))
		}

		if !IpcDone {
			state.Logger.Error("IPC has exitted for an unknown reason. Restarting...")
			panic("IPC has exitted for an unknown reason. Restarting...")
		}
	}()

	pubsub := state.Redis.Subscribe(
		state.Context,
		state.Config.Meta.WebRedisChannel,
	)

	pubsub.PSubscribe(state.Context, state.MewldInstanceList.Config.RedisChannel+"/*")

	defer pubsub.Close()

	ch := pubsub.Channel()

	for msg := range ch {
		// NOTE: The below branches may look duplicated however this is to allow for future expansion
		if msg.Channel == state.Config.Meta.WebRedisChannel {
			go func(msg *redis.Message) {
				var cmd *mredis.LauncherCmd

				err := json.Unmarshal([]byte(msg.Payload), &cmd)

				// Invalid JSON, return to avoid costly allocations
				if err != nil {
					return
				}

				if cmd == nil {
					return
				}

				// Not for us, return
				if cmd.Scope != "splashtail-web" {
					return
				}

				if cmd.CommandId == "" {
					return
				}

				// If response, return
				if len(cmd.Data) > 0 {
					if _, ok := cmd.Data["respCluster"]; ok {
						return
					}
				}

				action, ok := ipcEvents[cmd.Action]

				if !ok {
					err = core.SendResponse(msg.Channel, &mredis.LauncherCmd{
						Scope:     cmd.Scope,
						Action:    cmd.Action,
						CommandId: cmd.CommandId,
						Output: map[string]any{
							"error": "Invalid/Unsupported action",
						},
					})

					if err != nil {
						state.Logger.Error("Error sending IPC response", zap.Any("error", err))
						return
					}

					return
				}

				if !slices.Contains(action.SupportedModes, core.IPCModeAPI) {
					err = core.SendResponse(msg.Channel, &mredis.LauncherCmd{
						Scope:     cmd.Scope,
						Action:    cmd.Action,
						CommandId: cmd.CommandId,
						Output: map[string]any{
							"error": "IPC not supported by the API",
						},
					})

					if err != nil {
						state.Logger.Error("Error sending IPC response", zap.Any("error", err))
						return
					}

					return
				}

				resp, err := action.Exec(cmd)

				if err != nil {
					state.Logger.Error("Error executing IPC command", zap.Any("error", err))
					err = core.SendResponse(msg.Channel, &mredis.LauncherCmd{
						Scope:     cmd.Scope,
						Action:    cmd.Action,
						CommandId: cmd.CommandId,
						Output: map[string]any{
							"error": err.Error(),
						},
					})

					if err != nil {
						state.Logger.Error("Error sending IPC response", zap.Any("error", err))
						return
					}

					return
				}

				if resp == nil {
					state.Logger.Error("Error executing IPC command", zap.Any("error", err))
					err = core.SendResponse(msg.Channel, &mredis.LauncherCmd{
						Scope:     cmd.Scope,
						Action:    cmd.Action,
						CommandId: cmd.CommandId,
						Output: map[string]any{
							"error": "Command returned nil",
						},
					})

					if err != nil {
						state.Logger.Error("Error sending IPC response", zap.Any("error", err))
						return
					}

					return
				}

				resp.Scope = cmd.Scope
				resp.Action = cmd.Action
				resp.CommandId = cmd.CommandId

				err = core.SendResponse(msg.Channel, resp)

				if err != nil {
					state.Logger.Error("Error sending IPC response", zap.Any("error", err))
					return
				}
			}(msg)
		} else {
			go func(msg *redis.Message) {
				var cmd *mredis.LauncherCmd

				err := json.Unmarshal([]byte(msg.Payload), &cmd)

				// Invalid JSON, return to avoid costly allocations
				if err != nil {
					return
				}

				if cmd == nil {
					return
				}

				// Not for us, return
				if cmd.Scope != "splashtail" {
					return
				}

				if cmd.CommandId == "" {
					return
				}

				// If response, return
				if len(cmd.Data) > 0 {
					if _, ok := cmd.Data["respCluster"]; ok {
						return
					}
				}

				action, ok := ipcEvents[cmd.Action]

				if !ok {
					err = core.SendResponse(msg.Channel, &mredis.LauncherCmd{
						Scope:     cmd.Scope,
						Action:    cmd.Action,
						CommandId: cmd.CommandId,
						Output: map[string]any{
							"error": "Invalid/Unsupported action",
						},
					})

					if err != nil {
						state.Logger.Error("Error sending IPC response", zap.Any("error", err))
						return
					}

					return
				}

				if !slices.Contains(action.SupportedModes, core.IPCModeBot) {
					err = core.SendResponse(msg.Channel, &mredis.LauncherCmd{
						Scope:     cmd.Scope,
						Action:    cmd.Action,
						CommandId: cmd.CommandId,
						Output: map[string]any{
							"error": "IPC not supported by the bot/cluster",
						},
					})

					if err != nil {
						state.Logger.Error("Error sending IPC response", zap.Any("error", err))
						return
					}

					return
				}

				resp, err := action.Exec(cmd)

				if err != nil {
					state.Logger.Error("Error executing IPC command", zap.Any("error", err))
					err = core.SendResponse(msg.Channel, &mredis.LauncherCmd{
						Scope:     cmd.Scope,
						Action:    cmd.Action,
						CommandId: cmd.CommandId,
						Output: map[string]any{
							"error": err.Error(),
						},
					})

					if err != nil {
						state.Logger.Error("Error sending IPC response", zap.Any("error", err))
						return
					}

					return
				}

				if resp == nil {
					state.Logger.Error("Error executing IPC command", zap.Any("error", err))
					err = core.SendResponse(msg.Channel, &mredis.LauncherCmd{
						Scope:     cmd.Scope,
						Action:    cmd.Action,
						CommandId: cmd.CommandId,
						Output: map[string]any{
							"error": "Command returned nil",
						},
					})

					if err != nil {
						state.Logger.Error("Error sending IPC response", zap.Any("error", err))
						return
					}

					return
				}

				resp.Scope = cmd.Scope
				resp.Action = cmd.Action
				resp.CommandId = cmd.CommandId

				err = core.SendResponse(msg.Channel, resp)

				if err != nil {
					state.Logger.Error("Error sending IPC response", zap.Any("error", err))
					return
				}
			}(msg)
		}
	}
}

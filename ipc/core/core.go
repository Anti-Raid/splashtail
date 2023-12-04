package core

import (
	"splashtail/state"

	mredis "github.com/cheesycod/mewld/redis"

	jsoniter "github.com/json-iterator/go"
)

var json = jsoniter.ConfigFastest

func SendResponse(resp *mredis.LauncherCmd) error {
	resp.Scope = "splashtail"

	if len(resp.Data) == 0 {
		resp.Data = map[string]any{}
	}

	resp.Data["respCluster"] = -1 // IPC protocol needs this set

	bytes, err := json.Marshal(resp)

	if err != nil {
		return err
	}

	return state.Redis.Publish(state.Context, state.MewldInstanceList.Config.RedisChannel, bytes).Err()
}

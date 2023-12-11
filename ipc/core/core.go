package core

import (
	"splashtail/state"

	mredis "github.com/cheesycod/mewld/redis"

	jsoniter "github.com/json-iterator/go"
)

type IPCMode int

const (
	// IPCModeBot means that the IPC event is supported by the bot (cluster) channel
	IPCModeBot IPCMode = iota
	// IPCModeAPI means that the IPC event is supported by the API channel
	IPCModeAPI
)

type IPC struct {
	// Description is the description of the IPC event
	Description string
	// SupportedModes is the modes that the IPC event is supported in
	SupportedModes []IPCMode
	// Deprecated is whether or not the IPC event is deprecated.
	// If yes, then this field should be the reason why.
	Deprecated string
	// Exec is the function to execute when the IPC event is received
	Exec func(c *mredis.LauncherCmd) (*mredis.LauncherCmd, error)
}

var json = jsoniter.ConfigFastest

func SendResponse(reqChannel string, resp *mredis.LauncherCmd) error {
	if len(resp.Data) == 0 {
		resp.Data = map[string]any{}
	}

	resp.Data["respCluster"] = -1 // IPC protocol needs this set

	bytes, err := json.Marshal(resp)

	if err != nil {
		return err
	}

	return state.Redis.Publish(state.Context, reqChannel, bytes).Err()
}

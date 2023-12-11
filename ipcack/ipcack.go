package ipcack

import (
	"splashtail/state"
	"splashtail/syncmap"

	mredis "github.com/cheesycod/mewld/redis"
	jsoniter "github.com/json-iterator/go"
)

var json = jsoniter.ConfigFastest

type Ack struct {
	Verify func(r *mredis.LauncherCmd) error
	Chan   chan *mredis.LauncherCmd
}

var AckQueue = syncmap.Map[string, *Ack]{} // taskID -> chan

func Acker() {
	if state.CurrentOperationMode != "webserver" {
		panic("cannot start acker in non-webserver mode")
	}

	sp := state.Redis.Subscribe(state.Context, state.Config.Meta.WebRedisChannel)

	defer sp.Close()

	ch := sp.Channel()

	for msg := range ch {
		var cmd *mredis.LauncherCmd

		err := json.Unmarshal([]byte(msg.Payload), &cmd)

		// Invalid JSON, return to avoid costly allocations
		if err != nil {
			continue
		}

		if cmd.Scope != "splashtail-web" {
			continue
		}

		if cmd.CommandId == "" {
			continue
		}

		if cmd.Data["respCluster"] == nil {
			continue
		}

		if cmd.Data["respCluster"].(float64) != -1 {
			continue
		}

		ack, ok := AckQueue.Load(cmd.CommandId)

		if !ok {
			continue
		}

		if ack.Verify != nil {
			err := ack.Verify(cmd)

			if err != nil {
				continue
			}
		}

		ack.Chan <- cmd
	}

	panic("this should never exit")
}

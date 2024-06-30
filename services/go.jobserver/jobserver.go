package jobserver

import (
	"github.com/anti-raid/splashtail/services/go.jobserver/bgtasks"
	"github.com/anti-raid/splashtail/services/go.jobserver/core"
	"github.com/anti-raid/splashtail/services/go.jobserver/state"
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

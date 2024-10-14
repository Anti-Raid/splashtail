package main

import (
	"os"
	"os/signal"
	"syscall"

	"go.jobserver/core"
	"go.jobserver/rpc"
	"go.jobserver/state"
)

func CreateJobServer() {
	// Set state of all pending tasks to 'failed'
	_, err := state.Pool.Exec(state.Context, "UPDATE jobs SET state = $1 WHERE state = $2", "failed", "pending")

	if err != nil {
		panic(err)
	}

	go rpc.JobserverRpcServer()

	// Resume ongoing jobs
	go core.Resume()
}

func LaunchJobserver() {
	state.CurrentOperationMode = "jobs"

	state.Setup()

	state.Logger.Info("Starting jobserver")

	CreateJobServer()

	// Wait until signal is received
	c := make(chan os.Signal, 1)

	signal.Notify(c, syscall.SIGTERM, syscall.SIGINT, syscall.SIGHUP)

	<-c
}

func main() {
	LaunchJobserver() // Just launch the jobserver
}

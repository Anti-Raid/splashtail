package rpc

import (
	"context"
	"fmt"

	"go.api/rpc_messages"
	"go.api/state"
)

// Calls the CheckCommandPermission method to check whether or not a command is runnable
func JobserverSpawnTask(
	ctx context.Context,
	clusterId int,
	spawnTask *rpc_messages.JobserverSpawnTask,
) (res *rpc_messages.JobserverSpawnTaskResponse, err error) {
	return RpcQuery[rpc_messages.JobserverSpawnTaskResponse](
		ctx,
		state.IpcClient,
		"POST",
		fmt.Sprintf("%s/spawn-task", CalcJobserverAddr(clusterId)),
		spawnTask,
		true,
	)
}

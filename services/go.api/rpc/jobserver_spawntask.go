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
	spawnTask *rpc_messages.JobserverSpawn,
) (res *rpc_messages.JobserverSpawnResponse, err error) {
	return RpcQuery[rpc_messages.JobserverSpawnResponse](
		ctx,
		state.IpcClient,
		"POST",
		fmt.Sprintf("%s/spawn", CalcJobserverAddr()),
		spawnTask,
		true,
	)
}

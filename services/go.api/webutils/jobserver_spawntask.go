package webutils

import (
	"context"
	"fmt"

	"go.api/rpc_messages"
)

// Calls the CheckCommandPermission method to check whether or not a command is runnable
func JobserverSpawnTask(
	ctx context.Context,
	clusterId int,
	spawnTask *rpc_messages.JobserverSpawnTask,
) (res *rpc_messages.JobserverSpawnTaskResponse, err error) {
	return RpcQuery[rpc_messages.JobserverSpawnTaskResponse](
		ctx,
		"POST",
		fmt.Sprintf("%s/spawn-task", CalcJobserverAddr(clusterId)),
		spawnTask,
	)
}

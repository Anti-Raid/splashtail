package rpc

import (
	"context"
	"fmt"

	"go.api/rpc_messages"
	"go.api/state"
)

// ParsePermissionChecks verifies permission checks. This currently needs an animus magic call
func ExecutePerModuleFunction(ctx context.Context, clusterId int, data *rpc_messages.ExecutePerModuleFunctionRequest) (*rpc_messages.ExecutePerModuleFunctionResponse, error) {
	return RpcQuery[rpc_messages.ExecutePerModuleFunctionResponse](
		ctx,
		state.IpcClient,
		"POST",
		fmt.Sprintf("%s/execute-per-module-function", CalcBotAddr(clusterId)),
		data,
		true,
	)
}

package webutils

import (
	"context"
	"fmt"

	"go.api/rpc_messages"
)

// ParsePermissionChecks verifies permission checks. This currently needs an animus magic call
func ExecutePerModuleFunction(ctx context.Context, clusterId int, data *rpc_messages.ExecutePerModuleFunctionRequest) (*rpc_messages.ExecutePerModuleFunctionResponse, error) {
	return RpcQuery[rpc_messages.ExecutePerModuleFunctionResponse](
		ctx,
		"POST",
		fmt.Sprintf("%s/execute-per-module-function", CalcBotAddr(clusterId)),
		data,
	)
}

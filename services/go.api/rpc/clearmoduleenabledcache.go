package rpc

import (
	"context"
	"fmt"

	"go.api/rpc_messages"
	"go.api/state"
)

// Clears the bots internal modules enabled cache
func ClearModulesEnabledCache(ctx context.Context, data *rpc_messages.ClearModulesEnabledCacheRequest) (*rpc_messages.ClearModulesEnabledCacheResponse, error) {
	return RpcQuery[rpc_messages.ClearModulesEnabledCacheResponse](
		ctx,
		state.IpcClient,
		"POST",
		fmt.Sprintf("%s/clear-modules-enabled-cache", CalcBotAddr()),
		data,
		true,
	)
}

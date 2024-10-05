package rpc

import (
	"context"
	"fmt"

	"go.api/rpc_messages"
	"go.api/state"
)

// Dispatches a trusted web event to the bot
func DispatchTrustedWebEvent(ctx context.Context, clusterId int, data *rpc_messages.DispatchTrustedWebEventRequest) (*rpc_messages.DispatchTrustedWebEventResponse, error) {
	return RpcQuery[rpc_messages.DispatchTrustedWebEventResponse](
		ctx,
		state.IpcClient,
		"POST",
		fmt.Sprintf("%s/dispatch-trusted-web-event", CalcBotAddr(clusterId)),
		data,
		true,
	)
}

package rpc

import (
	"context"
	"fmt"

	"go.api/rpc_messages"
	"go.api/state"
)

// Calls the SettingsOperation method to execute a settings operation (settings-operation/:guild_id/:user_id)
func SettingsOperation(
	ctx context.Context,
	clusterId int,
	guildID string,
	userID string,
	settingsOpReq *rpc_messages.SettingsOperationRequest,
) (res *rpc_messages.CanonicalSettingsResult, err error) {
	return RpcQuery[rpc_messages.CanonicalSettingsResult](
		ctx,
		state.IpcClient,
		"POST",
		fmt.Sprintf("%s/settings-operation/%s/%s", CalcBotAddr(clusterId), guildID, userID),
		settingsOpReq,
		true,
	)
}

package rpc

import (
	"context"
	"fmt"

	"go.api/rpc_messages"
	"go.api/state"
)

// Calls the CheckCommandPermission method to check whether or not a command is runnable
func CheckCommandPermission(
	ctx context.Context,
	guildID string,
	userID string,
	command string,
	checkCommandOptions rpc_messages.RpcCheckCommandOptions,
) (res *rpc_messages.CheckCommandPermission, err error) {
	return RpcQuery[rpc_messages.CheckCommandPermission](
		ctx,
		state.IpcClient,
		"GET",
		fmt.Sprintf("%s/check-command-permission/%s/%s", CalcBotAddr(), guildID, userID),
		rpc_messages.CheckCommandPermissionRequest{
			Command: command,
			Opts:    checkCommandOptions,
		},
		true,
	)
}

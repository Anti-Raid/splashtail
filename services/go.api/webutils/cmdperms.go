package webutils

import (
	"context"
	"fmt"

	"go.api/rpc_messages"
)

// Calls the CheckCommandPermission method to check whether or not a command is runnable
func CheckCommandPermission(
	ctx context.Context,
	clusterId int,
	guildID string,
	userID string,
	command string,
	checkCommandOptions rpc_messages.RpcCheckCommandOptions,
) (res *rpc_messages.CheckCommandPermission, err error) {
	return RpcQuery[rpc_messages.CheckCommandPermission](
		ctx,
		"GET",
		fmt.Sprintf("%s/check-command-permission/%s/%s", CalcBotAddr(clusterId), guildID, userID),
		rpc_messages.CheckCommandPermissionRequest{
			Command: command,
			Opts:    checkCommandOptions,
		},
	)
}

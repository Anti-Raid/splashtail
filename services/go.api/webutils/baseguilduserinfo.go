package webutils

import (
	"context"
	"fmt"

	"go.api/rpc_messages"
)

// Calls the BaseGuildUserInfo method to get basic user + guild info (base-guild-user-info/:guild_id/:user_id)
func BaseGuildUserInfo(
	ctx context.Context,
	clusterId int,
	guildID string,
	userID string,
) (res *rpc_messages.BaseGuildUserInfo, err error) {
	return RpcQuery[rpc_messages.BaseGuildUserInfo](
		ctx,
		"GET",
		fmt.Sprintf("%s/base-guild-user-info/%s/%s", CalcBotAddr(clusterId), guildID, userID),
		nil,
	)
}

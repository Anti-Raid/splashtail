package rpc

import (
	"context"
	"fmt"

	"go.api/state"
)

// guilds-exist

// Calls the GuildsExist method to find out if the bot is in the specified list of guilds
func GuildsExist(
	ctx context.Context,
	guildIds []string,
) (res *[]uint16, err error) {
	return RpcQuery[[]uint16](
		ctx,
		state.IpcClient,
		"GET",
		fmt.Sprintf("%s/guilds-exist", CalcBotAddr()),
		guildIds,
		true,
	)
}

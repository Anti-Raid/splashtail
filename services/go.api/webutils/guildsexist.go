package webutils

import (
	"context"
	"fmt"
)

// guilds-exist

// Calls the GuildsExist method to find out if the bot is in the specified list of guilds
func GuildsExist(
	ctx context.Context,
	clusterId int,
	guildIds []string,
) (res *[]uint16, err error) {
	return RpcQuery[[]uint16](
		ctx,
		"GET",
		fmt.Sprintf("%s/guilds-exist", CalcBotAddr(clusterId)),
		guildIds,
	)
}

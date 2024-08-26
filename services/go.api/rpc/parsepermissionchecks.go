package rpc

import (
	"context"
	"fmt"

	"go.api/state"
	"go.std/silverpelt"
)

// ParsePermissionChecks verifies permission checks. This currently needs an animus magic call
func ParsePermissionChecks(ctx context.Context, clusterId int, guildId string, permChecks *silverpelt.PermissionChecks) (*silverpelt.PermissionChecks, error) {
	return RpcQuery[silverpelt.PermissionChecks](
		ctx,
		state.IpcClient,
		"GET",
		fmt.Sprintf("%s/parse-permission-checks/%s", CalcBotAddr(clusterId), guildId),
		permChecks,
		true,
	)
}

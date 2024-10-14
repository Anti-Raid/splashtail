package rpc

import (
	"context"
	"fmt"

	"go.api/state"
	"go.std/silverpelt"
)

// ParsePermissionChecks verifies permission checks for a guild
func ParsePermissionChecks(ctx context.Context, guildId string, permChecks *silverpelt.PermissionChecks) (*silverpelt.PermissionChecks, error) {
	return RpcQuery[silverpelt.PermissionChecks](
		ctx,
		state.IpcClient,
		"GET",
		fmt.Sprintf("%s/parse-permission-checks/%s", CalcBotAddr(), guildId),
		permChecks,
		true,
	)
}

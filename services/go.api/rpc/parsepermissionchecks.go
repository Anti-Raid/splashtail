package rpc

import (
	"context"
	"fmt"

	"go.api/state"
	"go.std/silverpelt"
)

// ParsePermissionChecks verifies permission checks for a guild
func ParsePermissionChecks(ctx context.Context, permChecks *silverpelt.PermissionCheck) (*silverpelt.PermissionCheck, error) {
	return RpcQuery[silverpelt.PermissionCheck](
		ctx,
		state.IpcClient,
		"GET",
		fmt.Sprintf("%s/parse-permission-checks", CalcBotAddr()),
		permChecks,
		true,
	)
}

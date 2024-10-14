package rpc

import (
	"context"
	"fmt"

	"go.api/state"
	"go.std/silverpelt"
)

func Modules(ctx context.Context) (*[]silverpelt.CanonicalModule, error) {
	return RpcQuery[[]silverpelt.CanonicalModule](
		ctx,
		state.IpcClient,
		"GET",
		fmt.Sprintf("%s/modules", CalcBotAddr()),
		nil,
		true,
	)
}

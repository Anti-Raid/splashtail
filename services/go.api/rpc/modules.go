package rpc

import (
	"context"
	"fmt"

	"github.com/Anti-Raid/corelib_go/silverpelt"
	"go.api/state"
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

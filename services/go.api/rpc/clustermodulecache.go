package rpc

import (
	"context"
	"fmt"

	"go.api/rpc_messages"
	"go.api/state"
	"go.std/utils/syncmap"
)

var ClusterModuleCache = &ClusterModuleCacher{}

type ClusterModuleCacher struct {
	cache syncmap.Map[int, rpc_messages.ClusterModules]
}

func (c *ClusterModuleCacher) GetClusterModules(ctx context.Context, clusterId int) (*rpc_messages.ClusterModules, error) {
	if v, ok := c.cache.Load(clusterId); ok {
		return &v, nil
	}

	modules, err := RpcQuery[rpc_messages.ClusterModules](
		ctx,
		state.IpcClient,
		"GET",
		fmt.Sprintf("%s/modules", CalcBotAddr(clusterId)),
		nil,
		true,
	)

	if err != nil {
		return nil, err
	}

	c.cache.Store(clusterId, *modules)

	return modules, nil
}

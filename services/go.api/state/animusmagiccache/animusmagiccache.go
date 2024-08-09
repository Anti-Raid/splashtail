package animusmagiccache

import (
	"context"
	"fmt"

	"github.com/redis/rueidis"
	"go.api/animusmagic_messages"
	"go.std/animusmagic"
	"go.std/utils/syncmap"
)

// Wrapper around animusmagic.AnimusMagicClient with cache support
type CachedAnimusMagicClient struct {
	*animusmagic.AnimusMagicClient

	ClusterModuleCache syncmap.Map[uint16, animusmagic_messages.ClusterModules]
}

// New returns a new CachedAnimusMagicClient
func New(c *animusmagic.AnimusMagicClient) *CachedAnimusMagicClient {
	return &CachedAnimusMagicClient{
		AnimusMagicClient:  c,
		ClusterModuleCache: syncmap.Map[uint16, animusmagic_messages.ClusterModules]{},
	}
}

// GetClusterModules returns the modules that are currently running on the cluster.
func (c *CachedAnimusMagicClient) GetClusterModules(ctx context.Context, redis rueidis.Client, clusterId uint16) (animusmagic_messages.ClusterModules, error) {
	if v, ok := c.ClusterModuleCache.Load(clusterId); ok {
		return v, nil
	}

	mlr, err := c.Request(
		ctx,
		redis,
		animusmagic_messages.BotAnimusMessage{
			Modules: &struct{}{},
		},
		&animusmagic.RequestOptions{
			ClusterID: &clusterId,
			To:        animusmagic.AnimusTargetBot,
			Op:        animusmagic.OpRequest,
		},
	)

	if err != nil {
		return nil, err
	}

	if len(mlr) == 0 {
		return nil, animusmagic.ErrNilMessage
	}

	if len(mlr) > 1 {
		return nil, fmt.Errorf("expected 1 response, got %d", len(mlr))
	}

	upr := mlr[0]

	resp, err := animusmagic.ParseClientResponse[animusmagic_messages.BotAnimusResponse](upr)

	if err != nil {
		return nil, err
	}

	if resp.ClientResp.Meta.Op == animusmagic.OpError {
		return nil, animusmagic.ErrOpError
	}

	if resp.Resp == nil || resp.Resp.Modules == nil {
		return nil, animusmagic.ErrNilMessage
	}

	modules := resp.Resp.Modules.Modules

	c.ClusterModuleCache.Store(clusterId, modules)

	return modules, nil
}

package state

import (
	"context"
	"fmt"
	"io"
	"net/http"

	"github.com/infinitybotlist/eureka/jsonimpl"
	"go.api/rpc_messages"
	"go.std/utils/syncmap"
)

type ClusterModuleCacher struct {
	cache syncmap.Map[uint16, rpc_messages.ClusterModules]
}

func (c *ClusterModuleCacher) GetClusterModules(ctx context.Context, clusterId uint16) (*rpc_messages.ClusterModules, error) {
	if v, ok := c.cache.Load(clusterId); ok {
		return &v, nil
	}

	req, err := http.NewRequestWithContext(ctx, "GET", fmt.Sprintf("http://%s:%d/modules", Config.BasePorts.BotBaseAddr.Parse(), Config.BasePorts.Bot.Parse()), nil)

	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}

	resp, err := IpcClient.Do(req)

	if err != nil {
		return nil, fmt.Errorf("failed to send request: %w", err)
	}

	if resp.StatusCode != http.StatusOK {
		var bodyText string

		if resp.Body != nil {
			bytes, err := io.ReadAll(resp.Body)

			if err != nil {
				bodyText = fmt.Sprintf("failed to read response body: %v, status code: %d", err, resp.StatusCode)
			} else {
				bodyText = string(bytes)
			}
		}

		return nil, fmt.Errorf(bodyText)
	}

	var modules rpc_messages.ClusterModules

	err = jsonimpl.UnmarshalReader(resp.Body, &modules)

	if err != nil {
		return nil, fmt.Errorf("failed to unmarshal response: %w", err)
	}

	c.cache.Store(clusterId, modules)

	return &modules, nil
}

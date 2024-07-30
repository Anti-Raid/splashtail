package webutils

import (
	"context"
	"fmt"

	"github.com/anti-raid/splashtail/core/go.std/animusmagic"
	"github.com/anti-raid/splashtail/core/go.std/silverpelt"
	"github.com/anti-raid/splashtail/services/go.api/animusmagic_messages"
	"github.com/redis/rueidis"
)

// ParsePermissionChecks verifies permission checks. This currently needs an animus magic call
func ParsePermissionChecks(ctx context.Context, c *animusmagic.AnimusMagicClient, redis rueidis.Client, clusterId uint16, guildId string, permChecks *silverpelt.PermissionChecks) (*silverpelt.PermissionChecks, error) {
	mlr, err := c.Request(
		ctx,
		redis,
		animusmagic_messages.BotAnimusMessage{
			ParsePermissionChecks: &struct {
				GuildID string `json:"guild_id"`
				Checks *silverpelt.PermissionChecks `json:"checks"`
			}{
				GuildID: guildId,
				Checks: permChecks,
			},
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

	if resp.Resp == nil || resp.Resp.ParsePermissionChecks == nil {
		return nil, animusmagic.ErrNilMessage
	}

	return resp.Resp.ParsePermissionChecks.Checks, nil
}

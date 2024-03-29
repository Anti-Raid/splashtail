package webutils

import (
	"context"
	"fmt"

	"github.com/anti-raid/splashtail/splashcore/animusmagic"
	"github.com/anti-raid/splashtail/splashcore/silverpelt"
	"github.com/anti-raid/splashtail/splashcore/utils"
	"github.com/redis/rueidis"
)

// Calls the CheckCommandPermission animus magic method to check whether or not a command is runnable
func CheckCommandPermission(
	c *animusmagic.AnimusMagicClient,
	ctx context.Context,
	redis rueidis.Client,
	clusterId uint16,
	guildID string,
	userID string,
	command string,
	permLimits []string,
) (res *silverpelt.PermissionResult, ok bool, err error) {
	var permLimitsPtr *[]string

	if len(permLimits) > 0 {
		permLimitsPtr = &permLimits
	}

	mlr, err := c.Request(
		ctx,
		redis,
		animusmagic.BotAnimusMessage{
			CheckCommandPermission: &struct {
				GuildID                     string    "json:\"guild_id\""
				UserID                      string    "json:\"user_id\""
				Command                     string    "json:\"command\""
				CustomResolvedKittycatPerms *[]string "json:\"custom_resolved_kittycat_perms,omitempty\""
				EnsureUserHasCustomResolved *bool     "json:\"ensure_user_has_custom_resolved,omitempty\""
			}{
				GuildID:                     guildID,
				UserID:                      userID,
				Command:                     command,
				CustomResolvedKittycatPerms: permLimitsPtr,
				EnsureUserHasCustomResolved: utils.Pointer(true),
			},
		},
		&animusmagic.RequestOptions{
			ClusterID: &clusterId,
		},
	)

	if err != nil {
		return nil, false, err
	}

	if len(mlr) == 0 {
		return nil, false, animusmagic.ErrNilMessage
	}

	if len(mlr) > 1 {
		return nil, false, fmt.Errorf("expected 1 response, got %d", len(mlr))
	}

	upr := mlr[0]

	resp, err := animusmagic.ParseClientResponse[animusmagic.BotAnimusResponse](upr)

	if err != nil {
		return nil, false, err
	}

	if resp.ClientResp.Meta.Op == animusmagic.OpError {
		return nil, false, animusmagic.ErrOpError
	}

	if resp.Resp == nil || resp.Resp.CheckCommandPermission == nil {
		return nil, false, animusmagic.ErrNilMessage
	}

	return &resp.Resp.CheckCommandPermission.PermRes, resp.Resp.CheckCommandPermission.IsOk, nil

}

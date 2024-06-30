package webutils

import (
	"context"
	"fmt"

	"github.com/anti-raid/splashtail/core/go.std/animusmagic"
	"github.com/anti-raid/splashtail/core/go.std/silverpelt"
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
	checkCommandOptions animusmagic.AmCheckCommandOptions,
) (res *silverpelt.PermissionResult, ok bool, err error) {
	mlr, err := c.Request(
		ctx,
		redis,
		animusmagic.BotAnimusMessage{
			CheckCommandPermission: &struct {
				GuildID             string                            "json:\"guild_id\""
				UserID              string                            "json:\"user_id\""
				Command             string                            "json:\"command\""
				CheckCommandOptions animusmagic.AmCheckCommandOptions `json:"opts"`
			}{
				GuildID:             guildID,
				UserID:              userID,
				Command:             command,
				CheckCommandOptions: checkCommandOptions,
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

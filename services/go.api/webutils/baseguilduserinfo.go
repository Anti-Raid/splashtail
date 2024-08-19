package webutils

import (
	"context"
	"fmt"
	"io"
	"net/http"

	"github.com/infinitybotlist/eureka/jsonimpl"
	"go.api/rpc_messages"
	"go.api/state"
)

// Calls the BaseGuildUserInfo method to get basic user + guild info (base-guild-user-info/:guild_id/:user_id)
func BaseGuildUserInfo(
	ctx context.Context,
	clusterId int,
	guildID string,
	userID string,
) (res *rpc_messages.BaseGuildUserInfo, err error) {
	req, err := http.NewRequestWithContext(ctx, "GET", fmt.Sprintf("%s/base-guild-user-info/%s/%s", CalcBotAddr(clusterId), guildID, userID), nil)

	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}

	req.Header.Set("Content-Type", "application/json")

	resp, err := state.IpcClient.Do(req)

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

	var bgui rpc_messages.BaseGuildUserInfo

	err = jsonimpl.UnmarshalReader(resp.Body, &bgui)

	if err != nil {
		return nil, fmt.Errorf("failed to unmarshal response: %w", err)
	}

	return &bgui, nil
}

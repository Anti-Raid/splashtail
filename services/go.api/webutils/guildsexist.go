package webutils

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"net/http"

	"github.com/infinitybotlist/eureka/jsonimpl"
	"go.api/state"
)

// guilds-exist

// Calls the GuildsExist method to find out if the bot is in the specified list of guilds
func GuildsExist(
	ctx context.Context,
	clusterId int,
	guildIds []string,
) (res []uint16, err error) {
	var body bytes.Buffer
	err = jsonimpl.MarshalToWriter(&body, guildIds)

	if err != nil {
		return nil, fmt.Errorf("failed to marshal guilds exist: %w", err)
	}

	req, err := http.NewRequestWithContext(ctx, "GET", fmt.Sprintf("%s/guilds-exist", CalcBotAddr(clusterId)), &body)

	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}

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

	err = jsonimpl.UnmarshalReader(resp.Body, &res)

	if err != nil {
		return nil, fmt.Errorf("failed to unmarshal response: %w", err)
	}

	return res, nil
}

package webutils

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"net/http"

	"github.com/infinitybotlist/eureka/jsonimpl"
	"go.api/rpc_messages"
	"go.api/state"
)

// Calls the SettingsOperation method to execute a settings operation (settings-operation/:guild_id/:user_id)
func SettingsOperation(
	ctx context.Context,
	clusterId int,
	guildID string,
	userID string,
	settingsOpReq *rpc_messages.SettingsOperationRequest,
) (res *rpc_messages.CanonicalSettingsResult, err error) {
	var body bytes.Buffer
	err = jsonimpl.MarshalToWriter(&body, settingsOpReq)

	if err != nil {
		return nil, fmt.Errorf("failed to marshal settings operation request: %w", err)
	}

	req, err := http.NewRequestWithContext(ctx, "POST", fmt.Sprintf("%s/settings-operation/%s/%s", CalcBotAddr(clusterId), guildID, userID), &body)

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

		return nil, fmt.Errorf("failed to send request: %s, status code %v", bodyText, resp.StatusCode)
	}

	var csr rpc_messages.CanonicalSettingsResult

	err = jsonimpl.UnmarshalReader(resp.Body, &csr)

	if err != nil {
		return nil, fmt.Errorf("failed to unmarshal response: %w", err)
	}

	return &csr, nil
}

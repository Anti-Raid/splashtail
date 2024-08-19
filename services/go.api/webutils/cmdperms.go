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
	"go.std/silverpelt"
)

// Calls the CheckCommandPermission method to check whether or not a command is runnable
func CheckCommandPermission(
	ctx context.Context,
	clusterId int,
	guildID string,
	userID string,
	command string,
	checkCommandOptions rpc_messages.RpcCheckCommandOptions,
) (res *silverpelt.PermissionResult, ok bool, err error) {
	var body bytes.Buffer
	err = jsonimpl.MarshalToWriter(&body, rpc_messages.CheckCommandPermissionRequest{
		Command: command,
		Opts:    checkCommandOptions,
	})

	if err != nil {
		return nil, false, fmt.Errorf("failed to marshal check command option request: %w", err)
	}

	req, err := http.NewRequestWithContext(ctx, "GET", fmt.Sprintf("%s/check-command-permission/%s/%s", CalcBotAddr(clusterId), guildID, userID), &body)

	if err != nil {
		return nil, false, fmt.Errorf("failed to create request: %w", err)
	}

	resp, err := state.IpcClient.Do(req)

	if err != nil {
		return nil, false, fmt.Errorf("failed to send request: %w", err)
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

		return nil, false, fmt.Errorf(bodyText)
	}

	var checkResp rpc_messages.CheckCommandPermission

	err = jsonimpl.UnmarshalReader(resp.Body, &resp)

	if err != nil {
		return nil, false, fmt.Errorf("failed to unmarshal response: %w", err)
	}

	return &checkResp.PermRes, checkResp.IsOk, nil
}

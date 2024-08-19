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

// Calls the CheckCommandPermission method to check whether or not a command is runnable
func JobserverSpawnTask(
	ctx context.Context,
	clusterId int,
	spawnTask *rpc_messages.JobserverSpawnTask,
) (res *rpc_messages.JobserverSpawnTaskResponse, err error) {
	var body bytes.Buffer
	err = jsonimpl.MarshalToWriter(&body, spawnTask)

	if err != nil {
		return nil, fmt.Errorf("failed to marshal jobserver spawn task request: %w", err)
	}

	req, err := http.NewRequestWithContext(ctx, "GET", fmt.Sprintf("%s/spawn-task", CalcJobserverAddr(clusterId)), &body)

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

	var str rpc_messages.JobserverSpawnTaskResponse

	err = jsonimpl.UnmarshalReader(resp.Body, &str)

	if err != nil {
		return nil, fmt.Errorf("failed to unmarshal response: %w", err)
	}

	return &str, nil
}

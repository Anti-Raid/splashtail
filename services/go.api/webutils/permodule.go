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

// ParsePermissionChecks verifies permission checks. This currently needs an animus magic call
func ExecutePerModuleFunction(ctx context.Context, clusterId int, data *rpc_messages.ExecutePerModuleFunctionRequest) error {
	var body bytes.Buffer
	err := jsonimpl.MarshalToWriter(&body, data)

	if err != nil {
		return fmt.Errorf("failed to marshal execute_per_module_function request: %w", err)
	}

	req, err := http.NewRequestWithContext(ctx, "POST", fmt.Sprintf("%s/execute-per-module-function", CalcBotAddr(clusterId)), &body)

	if err != nil {
		return fmt.Errorf("failed to create request: %w", err)
	}

	req.Header.Set("Content-Type", "application/json")

	resp, err := state.IpcClient.Do(req)

	if err != nil {
		return fmt.Errorf("failed to send request: %w", err)
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

		return fmt.Errorf(bodyText)
	}

	return nil
}

package webutils

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"net/http"

	"github.com/infinitybotlist/eureka/jsonimpl"
	"go.api/state"
	"go.std/silverpelt"
)

// ParsePermissionChecks verifies permission checks. This currently needs an animus magic call
func ParsePermissionChecks(ctx context.Context, clusterId int, guildId string, permChecks *silverpelt.PermissionChecks) (*silverpelt.PermissionChecks, error) {
	var body bytes.Buffer
	err := jsonimpl.MarshalToWriter(&body, permChecks)

	if err != nil {
		return nil, fmt.Errorf("failed to marshal permission checks: %w", err)
	}

	req, err := http.NewRequestWithContext(ctx, "GET", fmt.Sprintf("%s/parse-permission-checks/%s", CalcBotAddr(clusterId), guildId), &body)

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

	var checks silverpelt.PermissionChecks

	err = jsonimpl.UnmarshalReader(resp.Body, &checks)

	if err != nil {
		return nil, fmt.Errorf("failed to unmarshal response: %w", err)
	}

	return &checks, nil
}

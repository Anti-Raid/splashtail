package rpc

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"net/http"
	"strconv"

	"github.com/infinitybotlist/eureka/jsonimpl"
	"go.api/state"
)

func CalcBotPort() int {
	return state.Config.BasePorts.Bot
}

func CalcJobserverPort() int {
	return state.Config.BasePorts.Jobserver
}

func CalcBotAddr() string {
	return state.Config.BasePorts.BotBaseAddr + ":" + strconv.Itoa(CalcBotPort())
}

func CalcJobserverAddr() string {
	return state.Config.BasePorts.JobserverBaseAddr + ":" + strconv.Itoa(CalcJobserverPort())
}

// Calls a route using the RPC protocol
func RpcQuery[T any](
	ctx context.Context,
	client http.Client,
	method string,
	url string,
	body any,
	sendJsonHeader bool,
) (res *T, err error) {
	var reader io.Reader = nil
	if body != nil {
		var buf bytes.Buffer
		err := jsonimpl.MarshalToWriter(&buf, body)

		if err != nil {
			return nil, fmt.Errorf("failed to marshal request body: %w", err)
		}

		reader = &buf
	}

	req, err := http.NewRequestWithContext(ctx, method, url, reader)

	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}

	if sendJsonHeader {
		req.Header.Set("Content-Type", "application/json")
	}

	resp, err := client.Do(req)

	if err != nil {
		return nil, fmt.Errorf("failed to send request: %w", err)
	}

	//nolint:errcheck
	defer resp.Body.Close()
	//nolint:errcheck
	defer io.Copy(io.Discard, resp.Body)

	if resp.StatusCode != http.StatusOK {
		bytes, err := io.ReadAll(resp.Body)

		if err != nil {
			return nil, fmt.Errorf("failed to read response body for route %s: %w, status code: %d", url, err, resp.StatusCode)
		}

		return nil, fmt.Errorf("failed to get route %s: %s, status code: %d", url, string(bytes), resp.StatusCode)
	}

	var bgui T

	err = jsonimpl.UnmarshalReader(resp.Body, &bgui)

	if err != nil {
		return nil, fmt.Errorf("failed to unmarshal response: %w", err)
	}

	return &bgui, nil
}

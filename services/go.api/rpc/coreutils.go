package rpc

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"net/http"
	"strconv"

	"github.com/infinitybotlist/eureka/jsonimpl"
	"github.com/infinitybotlist/eureka/uapi"
	"go.api/state"
	"go.api/types"
)

func CalcBotPort(clusterId int) int {
	return state.Config.BasePorts.Bot.Parse() + clusterId
}

func CalcJobserverPort(clusterId int) int {
	return state.Config.BasePorts.Jobserver.Parse() + clusterId
}

func CalcBotAddr(clusterId int) string {
	return state.Config.BasePorts.BotBaseAddr.Parse() + ":" + strconv.Itoa(CalcBotPort(clusterId))
}

func CalcJobserverAddr(clusterId int) string {
	return state.Config.BasePorts.JobserverBaseAddr.Parse() + ":" + strconv.Itoa(CalcJobserverPort(clusterId))
}

func ClusterCheck(clusterId int) (resp uapi.HttpResponse, ok bool) {
	if state.MewldInstanceList == nil {
		return uapi.HttpResponse{
			Status: http.StatusPreconditionFailed,
			Json: types.ApiError{
				Message: "Mewld instance list not exposed yet. Please try again in 5 seconds!",
			},
			Headers: map[string]string{
				"Retry-After": "5",
			},
		}, false
	}

	// Check mewld instance list if the cluster actually exists
	var flag bool
	for _, v := range state.MewldInstanceList.Instances {
		if v.ClusterID == int(clusterId) {
			flag = true
			break
		}
	}

	if !flag {
		return uapi.HttpResponse{
			Status: http.StatusNotFound,
			Json: types.ApiError{
				Message: "Cluster not found",
			},
		}, false
	}

	for _, v := range state.MewldInstanceList.Instances {
		if v.ClusterID == clusterId {
			if !v.Active || v.CurrentlyKilling || len(v.ClusterHealth) == 0 {
				return uapi.HttpResponse{
					Status: http.StatusInternalServerError,
					Json: types.ApiError{
						Message: "Cluster is not healthy",
					},
					Headers: map[string]string{
						"Retry-After": "10",
					},
				}, false
			}
		}
	}

	return uapi.HttpResponse{}, true
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

package get_cluster_modules

import (
	"bytes"
	"context"
	"errors"
	"io"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/anti-raid/splashtail/state"
	"github.com/anti-raid/splashtail/types"
	"github.com/anti-raid/splashtail/types/silverpelt"
	"github.com/go-chi/chi/v5"
	jsoniter "github.com/json-iterator/go"
	orderedmap "github.com/wk8/go-ordered-map/v2"

	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/uapi"
)

var json = jsoniter.ConfigFastest

var reqBody = map[string]any{
	"Modules": map[string]string{},
}

var reqBodyBytes []byte

func Setup() {
	var err error
	reqBodyBytes, err = json.Marshal(reqBody)

	if err != nil {
		panic(err)
	}
}

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Get Cluster Modules",
		Description: "This endpoint returns the modules that are currently running on the cluster.",
		Resp:        orderedmap.OrderedMap[string, silverpelt.CanonicalModule]{},
		Params: []docs.Parameter{
			{
				Name:        "clusterId",
				Description: "The ID of the cluster to get the modules of.",
				In:          "path",
				Required:    true,
				Schema:      docs.IdSchema,
			},
		},
	}
}

func Route(d uapi.RouteData, r *http.Request) uapi.HttpResponse {
	if state.MewldInstanceList == nil {
		return uapi.HttpResponse{
			Status: http.StatusPreconditionFailed,
			Json: types.ApiError{
				Message: "Mewld instance list not exposed yet. Please try again in 5 seconds!",
			},
			Headers: map[string]string{
				"Retry-After": "5",
			},
		}
	}

	clusterIdStr := chi.URLParam(r, "clusterId")

	clusterId, err := strconv.Atoi(clusterIdStr)

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json: types.ApiError{
				Message: "Invalid cluster ID",
			},
		}
	}

	client := http.Client{
		Timeout: 10 * time.Second,
	}

	port := state.Config.Meta.BotIServerBasePort.Parse() + clusterId

	if port > 65535 {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json: types.ApiError{
				Message: "Invalid cluster ID [port > 65535]",
			},
		}
	}

	req, err := http.NewRequestWithContext(d.Context, "POST", "http://localhost:"+strconv.Itoa(port), bytes.NewReader(reqBodyBytes))

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Failed to create request: " + err.Error(),
			},
		}
	}

	req.Header.Set("Content-Type", "application/json")

	resp, err := client.Do(req)

	if err != nil {
		if errors.Is(err, context.DeadlineExceeded) {
			return uapi.HttpResponse{
				Status: http.StatusGatewayTimeout,
				Json: types.ApiError{
					Message: "Request to bot cluster timed out",
				},
			}
		}

		err := err.Error()

		if strings.Contains(err, "connection refused") {
			return uapi.HttpResponse{
				Status: http.StatusBadGateway,
				Json: types.ApiError{
					Message: "Failed to connect to bot cluster",
				},
			}
		}

		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Failed to send request: " + err,
			},
		}
	}

	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Failed to read response body: " + err.Error(),
			},
		}
	}

	if resp.StatusCode != http.StatusOK {
		return uapi.HttpResponse{
			Status: resp.StatusCode,
			Json: types.ApiError{
				Message: "Failed to get modules: " + string(body),
			},
		}
	}

	var cm *orderedmap.OrderedMap[string, silverpelt.CanonicalModule]

	err = json.Unmarshal(body, &cm)

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Failed to parse response body: " + err.Error(),
			},
		}
	}

	return uapi.HttpResponse{
		Json: cm,
	}
}

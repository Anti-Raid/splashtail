package get_cluster_modules

import (
	"fmt"
	"net/http"
	"strconv"

	"github.com/anti-raid/splashtail/animusmagic"
	"github.com/anti-raid/splashtail/types"
	"github.com/anti-raid/splashtail/types/silverpelt"
	"github.com/anti-raid/splashtail/webserver/state"
	"github.com/go-chi/chi/v5"
	"go.uber.org/zap"

	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/uapi"
)

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Get Cluster Modules",
		Description: "This endpoint returns the modules that are currently running on the cluster.",
		Resp:        []silverpelt.CanonicalModule{},
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

	// Get cluster id as uint16
	clusterId64, err := strconv.ParseUint(clusterIdStr, 10, 16)

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json: types.ApiError{
				Message: "Failed to parse cluster id: " + err.Error(),
			},
		}
	}

	clusterId := uint16(clusterId64)

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
		}
	}

	// Use animus magic to fetch module list
	if v, ok := state.AnimusMagicClient.Cache.ClusterModules.Load(clusterId); ok && v != nil {
		return uapi.HttpResponse{
			Json: v,
		}
	}

	moduleListResp, err := state.AnimusMagicClient.Request(d.Context, state.Rueidis, &animusmagic.RequestData{
		ClusterID: &clusterId,
		Message: &animusmagic.AnimusMessage{
			Modules: &struct{}{},
		},
	})

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Failed to fetch module list: " + err.Error(),
			},
		}
	}

	if len(moduleListResp) == 0 {
		return uapi.HttpResponse{
			Status: http.StatusNotFound,
			Json: types.ApiError{
				Message: "Data not found",
			},
		}
	}

	state.Logger.Info("Got response from animus magic", zap.Any("resp", moduleListResp[0]))

	if moduleListResp[0].Op == animusmagic.OpError {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Failed to fetch module list: " + fmt.Sprint(moduleListResp[0].Error),
			},
		}
	}

	return uapi.HttpResponse{
		Json: &moduleListResp[0].Resp.Modules.Modules,
	}
}

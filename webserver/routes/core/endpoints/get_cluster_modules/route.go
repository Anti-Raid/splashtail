package get_cluster_modules

import (
	"net/http"
	"strconv"

	"github.com/anti-raid/splashtail/types"
	"github.com/anti-raid/splashtail/types/silverpelt"
	"github.com/anti-raid/splashtail/webserver/state"
	"github.com/go-chi/chi/v5"

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

	modules, err := state.CachedAnimusMagicClient.GetClusterModules(d.Context, state.Rueidis, clusterId)

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Failed to fetch module list: " + err.Error(),
			},
		}
	}

	return uapi.HttpResponse{
		Json: modules,
	}
}

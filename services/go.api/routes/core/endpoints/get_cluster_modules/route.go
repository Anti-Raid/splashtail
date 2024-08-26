package get_cluster_modules

import (
	"net/http"
	"strconv"

	"github.com/go-chi/chi/v5"
	"go.api/rpc"
	"go.api/state"
	"go.api/types"
	"go.std/silverpelt"

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

	// Get cluster id as int
	clusterId64, err := strconv.Atoi(clusterIdStr)

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json: types.ApiError{
				Message: "Failed to parse cluster id: " + err.Error(),
			},
		}
	}

	hresp, ok := rpc.ClusterCheck(clusterId64)

	if !ok {
		return hresp
	}

	modules, err := rpc.ClusterModuleCache.GetClusterModules(d.Context, clusterId64)

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

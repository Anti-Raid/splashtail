package get_cluster_modules

import (
	"net/http"
	"strconv"

	"github.com/anti-raid/splashtail/animusmagic"
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

	// Use animus magic to fetch module list
	if v, ok := state.AnimusMagicClient.Cache.ClusterModules.Load(clusterId); ok && v != nil {
		return uapi.HttpResponse{
			Json: v,
		}
	}

	moduleListResp, err := state.AnimusMagicClient.Request(d.Context, state.Rueidis, &animusmagic.RequestData{
		ClusterID: &clusterId,
		Message: &animusmagic.AnimusMessage{
			Modules: map[string]string{},
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

	return uapi.HttpResponse{
		Json: moduleListResp,
	}
}

package get_cluster_health

import (
	"mewld"
	"net/http"

	"splashtail/types"

	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/uapi"

	mewldproc "mewld/proc"
)

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Get Cluster Health",
		Description: "This endpoint will return the health of all Anti-Raid clusters as collected by ``mewld``.",
		Resp:        mewldproc.InstanceList{},
	}
}

func Route(d uapi.RouteData, r *http.Request) uapi.HttpResponse {
	if mewld.InstanceList == nil {
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

	return uapi.HttpResponse{
		Json: *mewld.InstanceList,
	}
}

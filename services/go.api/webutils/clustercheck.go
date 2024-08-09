package webutils

import (
	"net/http"

	"github.com/infinitybotlist/eureka/uapi"
	"go.api/state"
	"go.api/types"
)

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

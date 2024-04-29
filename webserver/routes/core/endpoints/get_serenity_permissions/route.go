package get_serenity_permissions

import (
	"net/http"

	"github.com/anti-raid/splashtail/splashcore/types"
	"github.com/anti-raid/splashtail/webserver/state"

	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/uapi"
)

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Get Serenity Permissions",
		Description: "This endpoint returns all available serenity permissions from a random cluster.",
		Resp:        map[string]uint64{},
	}
}

func Route(d uapi.RouteData, r *http.Request) uapi.HttpResponse {
	// Permissions across clusters will be the same pretty much
	// so fast-path for that:
	var spl *map[string]uint64

	state.CachedAnimusMagicClient.SerenityPermissionsList.Range(func(k uint16, v map[string]uint64) bool {
		if len(v) == 0 {
			return true
		}

		spl = &v
		return false
	})

	if spl != nil {
		return uapi.HttpResponse{
			Json: *spl,
		}
	}

	var clusterId *uint16
	for {
		if clusterId != nil {
			break
		}

		select {
		case <-d.Context.Done():
			return uapi.HttpResponse{
				Status: http.StatusPreconditionFailed,
				Json: types.ApiError{
					Message: "Cluster list not available yet. Please try again in 5 seconds!",
				},
				Headers: map[string]string{
					"Retry-After": "5",
				},
			}
		default:
			if state.MewldInstanceList == nil {
				continue
			} else {
				var clustersUp []uint16

				for _, v := range state.MewldInstanceList.Instances {
					if !v.Active || v.CurrentlyKilling || len(v.ClusterHealth) == 0 {
						continue
					} else {
						clustersUp = append(clustersUp, uint16(v.ClusterID))
					}
				}

				if len(clustersUp) == 0 {
					continue
				}

				clusterId = &clustersUp[0]
			}
		}
	}

	sp, err := state.CachedAnimusMagicClient.GetSerenityPermissionList(d.Context, state.Rueidis, *clusterId)

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Failed to fetch serenity permission list: " + err.Error(),
			},
		}
	}

	return uapi.HttpResponse{
		Json: sp,
	}
}

package get_apps_meta

import (
	"net/http"

	"github.com/anti-raid/splashtail/core/go.std/types"
	"github.com/anti-raid/splashtail/services/go.api/apps"

	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/uapi"
)

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Get Apps Meta",
		Description: "Gets the current applications metadata. Returns a ``AppMeta`` object. See schema for more info.",
		Resp:        types.AppMeta{},
	}
}

func Route(d uapi.RouteData, r *http.Request) uapi.HttpResponse {
	return uapi.HttpResponse{
		Json: types.AppMeta{
			Positions: apps.Apps,
			Stable:    apps.Stable,
		},
	}
}

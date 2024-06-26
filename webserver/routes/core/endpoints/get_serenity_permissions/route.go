package get_serenity_permissions

import (
	"net/http"

	"github.com/anti-raid/splashtail/webserver/state"
	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/uapi"
)

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Get Serenity Permissions",
		Description: "This endpoint returns all available serenity permissions.",
		Resp:        map[string]uint64{},
	}
}

func Route(d uapi.RouteData, r *http.Request) uapi.HttpResponse {
	return uapi.HttpResponse{
		Json: state.SerenityPermissions,
	}
}

package get_modules

import (
	"net/http"

	"go.api/rpc"
	"go.api/state"
	"go.api/types"
	"go.std/silverpelt"

	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/uapi"
)

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Get Modules",
		Description: "This endpoint returns the modules on AntiRaid.",
		Resp:        []silverpelt.CanonicalModule{},
		Params:      []docs.Parameter{},
	}
}

var ModulesCache *[]silverpelt.CanonicalModule

func Route(d uapi.RouteData, r *http.Request) uapi.HttpResponse {
	if ModulesCache != nil {
		return uapi.HttpResponse{
			Json: ModulesCache,
		}
	}

	modules, err := rpc.Modules(state.Context)

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json:   types.ApiError{Message: "Error getting modules:" + err.Error()},
		}
	}

	ModulesCache = modules

	return uapi.HttpResponse{
		Json: modules,
	}
}

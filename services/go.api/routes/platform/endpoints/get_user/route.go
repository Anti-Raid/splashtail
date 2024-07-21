package get_user

import (
	"net/http"

	"github.com/anti-raid/splashtail/services/go.api/state"
	"github.com/anti-raid/splashtail/services/go.api/types"

	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/dovewing"
	"github.com/infinitybotlist/eureka/dovewing/dovetypes"
	"github.com/infinitybotlist/eureka/uapi"
	"go.uber.org/zap"

	"github.com/go-chi/chi/v5"
)

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Get Platform User",
		Description: "This endpoint will return a user object based on the user id for a given platform. This is useful for getting a user's avatar/username/discriminator etc.",
		Params: []docs.Parameter{
			{
				Name:        "id",
				In:          "path",
				Description: "The user's ID",
				Required:    true,
				Schema:      docs.IdSchema,
			},
			{
				Name:        "platform",
				In:          "query",
				Description: "The platform to get the user from.",
				Required:    true,
				Schema:      docs.IdSchema,
			},
		},
		Resp: dovetypes.PlatformUser{},
	}
}

func Route(d uapi.RouteData, r *http.Request) uapi.HttpResponse {
	var id = chi.URLParam(r, "id")
	var platform = r.URL.Query().Get("platform")

	var dovewingPlatform dovewing.Platform

	switch platform {
	case "discord":
		dovewingPlatform = state.DovewingPlatformDiscord
	default:
		return uapi.HttpResponse{
			Status: http.StatusUnsupportedMediaType,
			Json: types.ApiError{
				Message: "Unsupported platform. Only `discord` is supported at this time as a platform.",
			},
		}
	}

	user, err := dovewing.GetUser(d.Context, id, dovewingPlatform)

	if err != nil {
		state.Logger.Error("Error fetching user [dovewing]", zap.Error(err), zap.String("id", id), zap.String("platform", platform))
		return uapi.DefaultResponse(http.StatusNotFound)
	}

	return uapi.HttpResponse{
		Json: user,
	}
}

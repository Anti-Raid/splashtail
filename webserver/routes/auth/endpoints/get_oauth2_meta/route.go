package get_oauth2_meta

import (
	"net/http"
	"time"

	"github.com/anti-raid/splashtail/types"
	"github.com/anti-raid/splashtail/webserver/state"

	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/ratelimit"
	"github.com/infinitybotlist/eureka/uapi"
	"go.uber.org/zap"
)

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Get OAuth2 Metadata",
		Description: "Get the OAuth2 metadata for web login",
		Resp:        types.OauthMeta{},
		Params:      []docs.Parameter{},
	}
}

func Route(d uapi.RouteData, r *http.Request) uapi.HttpResponse {
	limit, err := ratelimit.Ratelimit{
		Expiry:      5 * time.Minute,
		MaxRequests: 15,
		Bucket:      "oauth2_login",
		Identifier: func(r *http.Request) string {
			return d.Auth.ID
		},
	}.Limit(d.Context, r)

	if err != nil {
		state.Logger.Error("Error while ratelimiting", zap.Error(err), zap.String("bucket", "ioauth"))
		return uapi.DefaultResponse(http.StatusInternalServerError)
	}

	if limit.Exceeded {
		return uapi.HttpResponse{
			Json: types.ApiError{
				Message: "You are being ratelimited. Please try again in " + limit.TimeToReset.String(),
			},
			Headers: limit.Headers(),
			Status:  http.StatusTooManyRequests,
		}
	}

	return uapi.HttpResponse{
		Json: types.OauthMeta{
			ClientID:   state.Config.DiscordAuth.ClientID,
			Scopes:     []string{"identify", "guilds"},
			Oauth2Base: "https://discord.com/api/oauth2/authorize",
		},
	}
}

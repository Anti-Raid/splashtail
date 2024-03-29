package get_user_guild_base_info

import (
	"errors"
	"net/http"
	"time"

	"github.com/anti-raid/splashtail/splashcore/animusmagic"
	"github.com/anti-raid/splashtail/splashcore/types"
	"github.com/anti-raid/splashtail/splashcore/utils"
	"github.com/anti-raid/splashtail/splashcore/utils/mewext"
	"github.com/anti-raid/splashtail/webserver/state"
	"github.com/anti-raid/splashtail/webserver/webutils"
	"github.com/go-chi/chi/v5"
	"go.uber.org/zap"

	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/ratelimit"
	"github.com/infinitybotlist/eureka/uapi"
)

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Get User Guild Base Info",
		Description: "This endpoint will return basic user and guild information given their IDs",
		Resp:        types.UserGuildBaseData{},
		Params: []docs.Parameter{
			{
				Name:        "user_id",
				Description: "The ID of the user to get information about",
				In:          "path",
				Required:    true,
				Schema:      docs.IdSchema,
			},
			{
				Name:        "guild_id",
				Description: "Whether to refresh the user's guilds from discord",
				In:          "path",
				Required:    true,
				Schema:      docs.IdSchema,
			},
		},
	}
}

func Route(d uapi.RouteData, r *http.Request) uapi.HttpResponse {
	limit, err := ratelimit.Ratelimit{
		Expiry:      5 * time.Minute,
		MaxRequests: 5,
		Bucket:      "get_user_guild_base_info",
	}.Limit(d.Context, r)

	if err != nil {
		state.Logger.Error("Error while ratelimiting", zap.Error(err), zap.String("bucket", "get_user_guild_base_info"))
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

	guildId := chi.URLParam(r, "guild_id")
	userId := chi.URLParam(r, "user_id")

	if guildId == "" || userId == "" {
		return uapi.DefaultResponse(http.StatusBadRequest)
	}

	clusterId, err := mewext.GetClusterIDFromGuildID(guildId, state.MewldInstanceList.Map, int(state.MewldInstanceList.ShardCount))

	if err != nil {
		state.Logger.Error("Error getting cluster ID", zap.Error(err))
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Error getting cluster ID: " + err.Error(),
			},
			Headers: map[string]string{
				"Retry-After": "10",
			},
		}
	}

	hresp, ok := webutils.ClusterCheck(clusterId)

	if !ok {
		return hresp
	}

	resps, err := state.AnimusMagicClient.Request(
		d.Context,
		state.Rueidis,
		animusmagic.BotAnimusMessage{
			BaseGuildUserInfo: &struct {
				GuildID string `json:"guild_id"`
				UserID  string `json:"user_id"`
			}{
				GuildID: guildId,
				UserID:  userId,
			},
		},
		&animusmagic.RequestOptions{
			ClusterID: utils.Pointer(uint16(clusterId)),
		},
	)

	if err != nil {
		state.Logger.Error("Error sending request to animus magic", zap.Error(err))

		if errors.Is(err, animusmagic.ErrOpError) && len(resps) > 0 {
			p, err := animusmagic.ParseClientResponse[animusmagic.BotAnimusResponse](resps[0])

			if err != nil {
				state.Logger.Error("Error parsing response", zap.Error(err))
				return uapi.HttpResponse{
					Status: http.StatusInternalServerError,
					Json: types.ApiError{
						Message: "Error parsing error response: " + err.Error(),
					},
				}
			}

			return uapi.HttpResponse{
				Status: http.StatusInternalServerError,
				Json:   p.Err,
				Headers: map[string]string{
					"Retry-After": "10",
				},
			}
		}

		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Error sending request to animus magic: " + err.Error(),
			},
			Headers: map[string]string{
				"Retry-After": "10",
			},
		}
	}

	if len(resps) != 1 {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Unexpected number of responses",
			},
		}
	}

	resp := resps[0]

	pr, err := animusmagic.ParseClientResponse[animusmagic.BotAnimusResponse](resp)

	if err != nil {
		state.Logger.Error("Error parsing response", zap.Error(err))
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Error parsing response: " + err.Error(),
			},
		}
	}

	return uapi.HttpResponse{
		Json: pr.Resp.BaseGuildUserInfo,
	}
}

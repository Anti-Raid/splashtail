package get_user_guild_base_info

import (
	"errors"
	"net/http"
	"time"

	"github.com/anti-raid/splashtail/core/go.std/animusmagic"
	"github.com/anti-raid/splashtail/core/go.std/types"
	"github.com/anti-raid/splashtail/core/go.std/utils"
	"github.com/anti-raid/splashtail/core/go.std/utils/mewext"
	"github.com/anti-raid/splashtail/services/go.api/animusmagic_messages"
	"github.com/anti-raid/splashtail/services/go.api/state"
	"github.com/anti-raid/splashtail/services/go.api/webutils"
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

	if guildId == "" {
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
		animusmagic_messages.BotAnimusMessage{
			BaseGuildUserInfo: &struct {
				GuildID string `json:"guild_id"`
				UserID  string `json:"user_id"`
			}{
				GuildID: guildId,
				UserID:  d.Auth.ID,
			},
		},
		&animusmagic.RequestOptions{
			ClusterID: utils.Pointer(uint16(clusterId)),
		},
	)

	if err != nil {
		state.Logger.Error("Error sending request to animus magic", zap.Error(err))

		if errors.Is(err, animusmagic.ErrOpError) && len(resps) > 0 {
			p, err := animusmagic.ParseClientResponse[animusmagic_messages.BotAnimusResponse](resps[0])

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

	pr, err := animusmagic.ParseClientResponse[animusmagic_messages.BotAnimusResponse](resp)

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
		Json: types.UserGuildBaseData{
			OwnerID:   pr.Resp.BaseGuildUserInfo.OwnerID,
			Name:      pr.Resp.BaseGuildUserInfo.Name,
			Icon:      pr.Resp.BaseGuildUserInfo.Icon,
			Roles:     pr.Resp.BaseGuildUserInfo.Roles,
			UserRoles: pr.Resp.BaseGuildUserInfo.UserRoles,
			BotRoles:  pr.Resp.BaseGuildUserInfo.BotRoles,
			Channels:  pr.Resp.BaseGuildUserInfo.Channels,
		},
	}
}

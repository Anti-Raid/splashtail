package toggle_module

import (
	"net/http"
	"strconv"
	"time"

	"github.com/anti-raid/splashtail/splashcore/animusmagic"
	"github.com/anti-raid/splashtail/splashcore/types"
	"github.com/anti-raid/splashtail/splashcore/utils"
	"github.com/anti-raid/splashtail/splashcore/utils/mewext"
	"github.com/anti-raid/splashtail/webserver/state"
	"github.com/anti-raid/splashtail/webserver/webutils"
	"github.com/go-chi/chi/v5"
	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/ratelimit"
	"github.com/infinitybotlist/eureka/uapi"
	"go.uber.org/zap"
)

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Toggle Module",
		Description: "Enable or disable a module for a specific guild",
		Resp:        types.ApiError{},
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
			{
				Name:        "module",
				Description: "The module to enable/disable",
				In:          "query",
				Required:    true,
				Schema:      docs.IdSchema,
			},
			{
				Name:        "disabled",
				Description: "Whether the module is disabled or not",
				In:          "query",
				Required:    true,
				Schema:      docs.IdSchema,
			},
		},
	}
}

func Route(d uapi.RouteData, r *http.Request) uapi.HttpResponse {
	limit, err := ratelimit.Ratelimit{
		Expiry:      2 * time.Minute,
		MaxRequests: 10,
		Bucket:      "module_configuration",
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

	module := r.URL.Query().Get("module")

	if module == "" {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json: types.ApiError{
				Message: "Query parameter `module` is required",
			},
		}
	}

	var disabled bool = r.URL.Query().Get("disabled") == "true"

	// INSERT ON CONFLICT UPDATE RETURNING id
	var id string
	err = state.Pool.QueryRow(
		d.Context,
		"INSERT INTO guild_module_configurations (guild_id, module, disabled) VALUES ($1, $2, $3) ON CONFLICT (guild_id, module) DO UPDATE SET disabled = $3 RETURNING id",
		guildId,
		module,
		disabled,
	).Scan(&id)

	if err != nil {
		state.Logger.Error("Failed to insert guild_module_configuration", zap.Error(err))
		return uapi.HttpResponse{
			Json: types.ApiError{
				Message: "Failed to insert guild_module_configuration: " + err.Error(),
			},
			Status: http.StatusInternalServerError,
		}
	}

	resps, err := state.AnimusMagicClient.Request(
		d.Context,
		state.Rueidis,
		animusmagic.BotAnimusMessage{
			ToggleModule: &struct {
				GuildID string `json:"guild_id"`
				Module  string `json:"module"`
				Enabled bool   `json:"enabled"`
			}{
				GuildID: guildId,
				Module:  module,
				Enabled: !disabled,
			},
		},
		&animusmagic.RequestOptions{
			ClusterID: utils.Pointer(uint16(clusterId)),
		},
	)

	if err != nil {
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
				Message: "Error sending request to animus magic: [unexpected response count of " + strconv.Itoa(len(resps)) + "]",
			},
			Headers: map[string]string{
				"Retry-After": "10",
			},
		}
	}

	return uapi.DefaultResponse(http.StatusNoContent)
}

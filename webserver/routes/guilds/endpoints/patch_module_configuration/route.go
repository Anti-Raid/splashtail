package patch_module_configuration

import (
	"net/http"
	"strconv"
	"time"

	"github.com/anti-raid/splashtail/splashcore/animusmagic"
	"github.com/anti-raid/splashtail/splashcore/silverpelt"
	"github.com/anti-raid/splashtail/splashcore/types"
	"github.com/anti-raid/splashtail/splashcore/utils"
	"github.com/anti-raid/splashtail/splashcore/utils/mewext"
	"github.com/anti-raid/splashtail/webserver/api"
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
		Summary:     "Patch Module Configuration",
		Description: "Updates the module configuration for a specific guild",
		Req:         types.PatchGuildModuleConfiguration{},
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
				In:          "path",
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
	module := chi.URLParam(r, "module")

	if guildId == "" || userId == "" || module == "" {
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

	// Find module from cluster
	modules, err := state.CachedAnimusMagicClient.GetClusterModules(d.Context, state.Rueidis, uint16(clusterId))

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Failed to fetch module list: " + err.Error(),
			},
		}
	}

	var moduleData *silverpelt.CanonicalModule

	for _, m := range modules {
		if m.ID == module {
			moduleData = &m
			break
		}
	}

	if moduleData == nil {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json: types.ApiError{
				Message: "Module not found",
			},
		}
	}

	// Read body
	var body types.PatchGuildModuleConfiguration

	hresp, ok = uapi.MarshalReqWithHeaders(r, &body, limit.Headers())

	if !ok {
		return hresp
	}

	var disabled *bool

	if body.Disabled != nil {
		if !moduleData.Toggleable {
			return uapi.HttpResponse{
				Status: http.StatusBadRequest,
				Json: types.ApiError{
					Message: "Module cannot be enabled/disablable (is not toggleable)",
				},
			}
		}

		// Check for permissions next
		if *body.Disabled {
			hresp, ok = api.HandlePermissionCheck(d.Auth.ID, guildId, "modules disable", api.PermLimits(d.Auth))

			if !ok {
				return hresp
			}
		} else {
			hresp, ok = api.HandlePermissionCheck(d.Auth.ID, guildId, "modules enable", api.PermLimits(d.Auth))

			if !ok {
				return hresp
			}
		}

		disabled = body.Disabled
	}

	if disabled != nil {
		// INSERT ON CONFLICT UPDATE RETURNING id
		var id string
		err = state.Pool.QueryRow(
			d.Context,
			"INSERT INTO guild_module_configurations (guild_id, module, disabled) VALUES ($1, $2, $3) ON CONFLICT (guild_id, module) DO UPDATE SET disabled = $3 RETURNING id",
			guildId,
			module,
			*disabled,
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
				ExecutePerModuleFunction: &struct {
					Module  string         `json:"module"`
					Toggle  string         `json:"toggle"`
					Options map[string]any `json:"options,omitempty"`
				}{
					Module: "settings",
					Toggle: "toggle_module",
					Options: map[string]any{
						"guild_id": guildId,
						"module":   module,
						"enabled":  !*disabled,
					},
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
	}

	return uapi.DefaultResponse(http.StatusNoContent)
}

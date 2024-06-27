package patch_module_configuration

import (
	"net/http"
	"strconv"
	"strings"
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

const (
	CACHE_FLUSH_NONE                           = 0      // No cache flush operation
	CACHE_FLUSH_MODULE_TOGGLE                  = 1 << 1 // Must trigger a module trigger
	CACHE_FLUSH_COMMAND_PERMISSION_CACHE_CLEAR = 1 << 2 // Must trigger a command permission cache clear
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

	// Fetch permission limits
	permLimits := api.PermLimits(d.Auth)

	var updateCols []string
	var updateArgs []any
	var cacheFlushFlag = CACHE_FLUSH_NONE

	var isDisabled bool // This must be set to ensure cache flushes are done correctly

	// Perm check area
	if body.Disabled != nil {
		value, clear, err := body.Disabled.Get()

		if err != nil {
			return uapi.HttpResponse{
				Status: http.StatusBadRequest,
				Json: types.ApiError{
					Message: "Error parsing disabled value: " + err.Error(),
				},
			}
		}

		if !moduleData.Toggleable {
			return uapi.HttpResponse{
				Status: http.StatusBadRequest,
				Json: types.ApiError{
					Message: "Module cannot be enabled/disablable (is not toggleable)",
				},
			}
		}

		if clear {
			// Differences from bot: we do not have the existing state of the module beforehand for simplicity
			//
			// This means that all the fast-path optimization used by the bot are not available to us
			if moduleData.IsDefaultEnabled {
				hresp, ok = api.HandlePermissionCheck(d.Auth.ID, guildId, "modules enable", animusmagic.AmCheckCommandOptions{
					CustomResolvedKittycatPerms: &permLimits,
				})

				if !ok {
					return hresp
				}
			} else {
				hresp, ok = api.HandlePermissionCheck(d.Auth.ID, guildId, "modules disable", animusmagic.AmCheckCommandOptions{
					CustomResolvedKittycatPerms: &permLimits,
				})

				if !ok {
					return hresp
				}
			}

			// Set isDisabled to ensure cache flushes are done correctly
			isDisabled = !moduleData.IsDefaultEnabled

			updateCols = append(updateCols, "disabled")
			updateArgs = append(updateArgs, nil)
		} else {
			// Check for permissions next
			if *value {
				hresp, ok = api.HandlePermissionCheck(d.Auth.ID, guildId, "modules disable", animusmagic.AmCheckCommandOptions{
					CustomResolvedKittycatPerms: &permLimits,
				})

				if !ok {
					return hresp
				}
			} else {
				hresp, ok = api.HandlePermissionCheck(d.Auth.ID, guildId, "modules enable", animusmagic.AmCheckCommandOptions{
					CustomResolvedKittycatPerms: &permLimits,
				})

				if !ok {
					return hresp
				}
			}

			// Set isDisabled to ensure cache flushes are done correctly
			isDisabled = *value

			updateCols = append(updateCols, "disabled")
			updateArgs = append(updateArgs, *value)
		}

		if cacheFlushFlag&CACHE_FLUSH_MODULE_TOGGLE != CACHE_FLUSH_MODULE_TOGGLE {
			cacheFlushFlag |= CACHE_FLUSH_MODULE_TOGGLE
		}
	}

	if body.DefaultPerms != nil {
		value, clear, err := body.DefaultPerms.Get()

		if err != nil {
			return uapi.HttpResponse{
				Status: http.StatusBadRequest,
				Json: types.ApiError{
					Message: "Error parsing default_perms value: " + err.Error(),
				},
			}
		}

		// Check for permissions next
		hresp, ok = api.HandlePermissionCheck(d.Auth.ID, guildId, "modules modperms", animusmagic.AmCheckCommandOptions{
			CustomResolvedKittycatPerms: &permLimits,
		})

		if !ok {
			return hresp
		}

		hresp, ok = api.HandlePermissionCheck(d.Auth.ID, guildId, "acl__modules_modperms "+module, animusmagic.AmCheckCommandOptions{
			CustomResolvedKittycatPerms: &permLimits,
		})

		if !ok {
			return hresp
		}

		if clear {
			updateCols = append(updateCols, "default_perms")
			updateArgs = append(updateArgs, nil)
		} else {
			parsedValue, err := webutils.ParsePermissionChecks(value)

			if err != nil {
				return uapi.HttpResponse{
					Status: http.StatusBadRequest,
					Json: types.ApiError{
						Message: "Error parsing permission checks: " + err.Error(),
					},
				}
			}

			if len(value.Checks) > 0 {
				updateCols = append(updateCols, "default_perms")
				updateArgs = append(updateArgs, parsedValue)
			}
		}

		if cacheFlushFlag&CACHE_FLUSH_MODULE_TOGGLE != CACHE_FLUSH_MODULE_TOGGLE && cacheFlushFlag&CACHE_FLUSH_COMMAND_PERMISSION_CACHE_CLEAR != CACHE_FLUSH_COMMAND_PERMISSION_CACHE_CLEAR {
			cacheFlushFlag |= CACHE_FLUSH_COMMAND_PERMISSION_CACHE_CLEAR
		}
	}

	if len(updateCols) == 0 {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json: types.ApiError{
				Message: "No valid fields to update",
			},
		}
	}

	// Create sql, insertParams is $N, $N+1... while updateParams are <col> = $N, <col2> = $N+1...
	var insertParams = make([]string, 0, len(updateCols))
	var updateParams = make([]string, 0, len(updateCols))
	var paramNo = 3 // 1 and 2 are guild_id and module
	for _, col := range updateCols {
		insertParams = append(insertParams, "$"+strconv.Itoa(paramNo))
		updateParams = append(updateParams, col+" = $"+strconv.Itoa(paramNo))
		paramNo++
	}

	var sqlString = "INSERT INTO guild_module_configurations (guild_id, module, " + strings.Join(updateCols, ", ") + ") VALUES ($1, $2, " + strings.Join(insertParams, ",") + ") ON CONFLICT (guild_id, module) DO UPDATE SET " + strings.Join(updateParams, ", ") + " RETURNING id"

	// Execute sql
	updateArgs = append([]any{guildId, module}, updateArgs...) // $1 and $2
	var id string
	err = state.Pool.QueryRow(
		d.Context,
		sqlString,
		updateArgs...,
	).Scan(&id)

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Error updating module configuration: " + err.Error(),
			},
		}
	}

	if cacheFlushFlag&CACHE_FLUSH_MODULE_TOGGLE == CACHE_FLUSH_MODULE_TOGGLE && body.Disabled != nil {
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
						"enabled":  isDisabled,
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

	if cacheFlushFlag&CACHE_FLUSH_COMMAND_PERMISSION_CACHE_CLEAR == CACHE_FLUSH_COMMAND_PERMISSION_CACHE_CLEAR {
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
					Toggle: "clear_command_permission_cache",
					Options: map[string]any{
						"guild_id": guildId,
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

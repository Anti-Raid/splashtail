package patch_module_configuration

import (
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/go-chi/chi/v5"
	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/ratelimit"
	"github.com/infinitybotlist/eureka/uapi"
	"github.com/jackc/pgx/v5"
	"go.api/api"
	"go.api/rpc"
	"go.api/rpc_messages"
	"go.api/state"
	"go.api/types"
	"go.std/silverpelt"
	"go.std/structparser/db"
	"go.std/utils"
	"go.std/utils/mewext"
	"go.uber.org/zap"
)

var (
	guildModuleConfigurationColsArr = db.GetCols(silverpelt.GuildModuleConfiguration{})
	guildModuleConfigurationCols    = strings.Join(guildModuleConfigurationColsArr, ", ")
)

const (
	CACHE_FLUSH_NONE          = 0      // No cache flush operation
	CACHE_FLUSH_MODULE_TOGGLE = 1 << 1 // Must trigger a module trigger
)

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Patch Module Configuration",
		Description: "Updates the configuration of a specific module for a specific guild",
		Req:         types.PatchGuildModuleConfiguration{},
		Resp:        silverpelt.GuildModuleConfiguration{},
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
		Expiry:      2 * time.Minute,
		MaxRequests: 10,
		Bucket:      "module_configuration",
	}.Limit(d.Context, r)

	if err != nil {
		state.Logger.Error("Error while ratelimiting", zap.Error(err), zap.String("bucket", "module_configuration"))
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

	hresp, ok := rpc.ClusterCheck(clusterId)

	if !ok {
		return hresp
	}

	// Read body
	var body types.PatchGuildModuleConfiguration

	hresp, ok = uapi.MarshalReqWithHeaders(r, &body, limit.Headers())

	if !ok {
		return hresp
	}

	if body.Module == "" {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json: types.ApiError{
				Message: "Module is required",
			},
		}
	}

	// Find module from cluster
	modules, err := rpc.ClusterModuleCache.GetClusterModules(d.Context, clusterId)

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Failed to fetch module list: " + err.Error(),
			},
		}
	}

	var moduleData *silverpelt.CanonicalModule

	for _, m := range *modules {
		if m.ID == body.Module {
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
			if moduleData.IsDefaultEnabled {
				hresp, ok = api.HandlePermissionCheck(d.Auth.ID, guildId, "modules enable", rpc_messages.RpcCheckCommandOptions{
					CustomResolvedKittycatPerms: permLimits,
				})

				if !ok {
					return hresp
				}
			} else {
				hresp, ok = api.HandlePermissionCheck(d.Auth.ID, guildId, "modules disable", rpc_messages.RpcCheckCommandOptions{
					CustomResolvedKittycatPerms: permLimits,
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
				// Disable
				hresp, ok = api.HandlePermissionCheck(d.Auth.ID, guildId, "modules disable", rpc_messages.RpcCheckCommandOptions{
					CustomResolvedKittycatPerms: permLimits,
				})

				if !ok {
					return hresp
				}
			} else {
				// Enable
				hresp, ok = api.HandlePermissionCheck(d.Auth.ID, guildId, "modules enable", rpc_messages.RpcCheckCommandOptions{
					CustomResolvedKittycatPerms: permLimits,
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
		hresp, ok = api.HandlePermissionCheck(d.Auth.ID, guildId, "modules modperms", rpc_messages.RpcCheckCommandOptions{
			CustomResolvedKittycatPerms: permLimits,
		})

		if !ok {
			return hresp
		}

		hresp, ok = api.HandlePermissionCheck(d.Auth.ID, guildId, "acl__modules_modperms "+body.Module, rpc_messages.RpcCheckCommandOptions{
			CustomResolvedKittycatPerms: permLimits,
		})

		if !ok {
			return hresp
		}

		if clear {
			hresp, ok = api.HandlePermissionCheck(d.Auth.ID, guildId, "acl__"+body.Module+"_defaultperms_check", rpc_messages.RpcCheckCommandOptions{
				CustomResolvedKittycatPerms: permLimits,
				CustomModuleConfiguration: silverpelt.GuildModuleConfiguration{
					Disabled:     utils.Pointer(false),
					Module:       body.Module,
					DefaultPerms: nil,
				}.Fill(),
			})

			if !ok {
				return hresp
			}

			updateCols = append(updateCols, "default_perms")
			updateArgs = append(updateArgs, nil)
		} else {
			parsedValue, err := rpc.ParsePermissionChecks(d.Context, clusterId, guildId, value)

			if err != nil {
				return uapi.HttpResponse{
					Status: http.StatusBadRequest,
					Json: types.ApiError{
						Message: "Error parsing permission checks: " + err.Error(),
					},
				}
			}

			hresp, ok = api.HandlePermissionCheck(d.Auth.ID, guildId, "acl__"+body.Module+"_defaultperms_check", rpc_messages.RpcCheckCommandOptions{
				CustomResolvedKittycatPerms: permLimits,
				CustomModuleConfiguration: silverpelt.GuildModuleConfiguration{
					Disabled:     utils.Pointer(false),
					Module:       body.Module,
					DefaultPerms: parsedValue,
				}.Fill(),
			})

			if !ok {
				return hresp
			}

			updateCols = append(updateCols, "default_perms")
			updateArgs = append(updateArgs, parsedValue)
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

	// Start a transaction
	tx, err := state.Pool.Begin(d.Context)

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Error starting transaction: " + err.Error(),
			},
		}
	}

	//nolint:errcheck
	defer tx.Rollback(d.Context)

	var sqlString = "INSERT INTO guild_module_configurations (guild_id, module, " + strings.Join(updateCols, ", ") + ") VALUES ($1, $2, " + strings.Join(insertParams, ",") + ") ON CONFLICT (guild_id, module) DO UPDATE SET " + strings.Join(updateParams, ", ") + " RETURNING id"

	// Execute sql
	updateArgs = append([]any{guildId, body.Module}, updateArgs...) // $1 and $2
	var id string
	err = tx.QueryRow(
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

	// Fetch the gmc
	row, err := tx.Query(d.Context, "SELECT "+guildModuleConfigurationCols+" FROM guild_module_configurations WHERE id = $1", id)

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Error fetching updated module configuration: " + err.Error(),
			},
		}
	}

	gmc, err := pgx.CollectOneRow(row, pgx.RowToStructByName[silverpelt.GuildModuleConfiguration])

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Error collecting updated module configuration: " + err.Error(),
			},
		}
	}

	// Commit transaction
	err = tx.Commit(d.Context)

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Error committing transaction: " + err.Error(),
			},
		}
	}

	if cacheFlushFlag&CACHE_FLUSH_MODULE_TOGGLE == CACHE_FLUSH_MODULE_TOGGLE && body.Disabled != nil {
		_, err := rpc.ExecutePerModuleFunction(
			d.Context,
			clusterId,
			&rpc_messages.ExecutePerModuleFunctionRequest{
				Module:   "settings",
				Function: "toggle_module",
				Args: map[string]any{
					"guild_id": guildId,
					"module":   body.Module,
					"enabled":  isDisabled,
				},
			},
		)

		if err != nil {
			return uapi.HttpResponse{
				Status: http.StatusInternalServerError,
				Json: types.ApiError{
					Message: "Error clearing bot caches: " + err.Error(),
				},
				Headers: map[string]string{
					"Retry-After": "10",
				},
			}
		}
	}

	return uapi.HttpResponse{
		Json: gmc,
	}
}

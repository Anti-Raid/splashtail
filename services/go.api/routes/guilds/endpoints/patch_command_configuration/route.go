package patch_command_configuration

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
	"go.api/rpc_messages"
	"go.api/state"
	"go.api/types"
	"go.api/webutils"
	"go.std/silverpelt"
	"go.std/structparser/db"
	"go.std/utils"
	"go.std/utils/mewext"
	"go.uber.org/zap"
)

var (
	fullGuildCommandConfigurationColsArr = db.GetCols(silverpelt.FullGuildCommandConfiguration{})
	fullGuildCommandConfigurationCols    = strings.Join(fullGuildCommandConfigurationColsArr, ", ")
)

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Patch Command Configuration",
		Description: "Updates the configuration of a specific command for a specific guild.",
		Req:         types.PatchGuildCommandConfiguration{},
		Resp:        silverpelt.FullGuildCommandConfiguration{},
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
		Bucket:      "command_configuration",
	}.Limit(d.Context, r)

	if err != nil {
		state.Logger.Error("Error while ratelimiting", zap.Error(err), zap.String("bucket", "command_configuration"))
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

	// Read body
	var body types.PatchGuildCommandConfiguration

	hresp, ok = uapi.MarshalReqWithHeaders(r, &body, limit.Headers())

	if !ok {
		return hresp
	}

	if body.Command == "" {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json: types.ApiError{
				Message: "Command is required",
			},
		}
	}

	baseCommand := strings.Split(body.Command, " ")[0]

	// Find module from cluster
	modules, err := webutils.ClusterModuleCache.GetClusterModules(d.Context, clusterId)

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Failed to fetch module list: " + err.Error(),
			},
		}
	}

	var moduleData *silverpelt.CanonicalModule
	var commandData *silverpelt.CanonicalCommand

	for _, m := range *modules {
		for _, cmd := range m.Commands {
			if cmd.Command.Name == baseCommand || cmd.Command.QualifiedName == baseCommand {
				moduleData = &m
				commandData = &cmd
				break
			}
		}
	}

	if moduleData == nil || commandData == nil {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json: types.ApiError{
				Message: "Command not found",
			},
		}

	}

	commandExtendedData := silverpelt.GetCommandExtendedData(silverpelt.PermuteCommandNames(body.Command), commandData.ExtendedData)

	if commandExtendedData == nil {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json: types.ApiError{
				Message: "Command extended data not found",
			},
		}
	}

	// Fetch permission limits
	permLimits := api.PermLimits(d.Auth)

	// Ensure user has permission to use the command
	hresp, ok = api.HandlePermissionCheck(d.Auth.ID, guildId, body.Command, rpc_messages.RpcCheckCommandOptions{
		CustomResolvedKittycatPerms: permLimits,
		Flags:                       rpc_messages.RpcCheckCommandOptionsFlagIgnoreCommandDisabled,
	})

	if !ok {
		return hresp
	}

	var updateCols []string
	var updateArgs []any

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

		if !moduleData.CommandsToggleable {
			return uapi.HttpResponse{
				Status: http.StatusBadRequest,
				Json: types.ApiError{
					Message: "Commands on this module cannot be enabled/disablable (is not toggleable)",
				},
			}
		}

		if clear {
			if commandExtendedData.IsDefaultEnabled {
				hresp, ok = api.HandlePermissionCheck(d.Auth.ID, guildId, "commands enable", rpc_messages.RpcCheckCommandOptions{
					CustomResolvedKittycatPerms: permLimits,
				})

				if !ok {
					return hresp
				}
			} else {
				hresp, ok = api.HandlePermissionCheck(d.Auth.ID, guildId, "commands disable", rpc_messages.RpcCheckCommandOptions{
					CustomResolvedKittycatPerms: permLimits,
				})

				if !ok {
					return hresp
				}
			}

			updateCols = append(updateCols, "disabled")
			updateArgs = append(updateArgs, nil)
		} else {
			// Check for permissions next
			if *value {
				// Enable
				hresp, ok = api.HandlePermissionCheck(d.Auth.ID, guildId, "commands enable", rpc_messages.RpcCheckCommandOptions{
					CustomResolvedKittycatPerms: permLimits,
				})

				if !ok {
					return hresp
				}
			} else {
				// Disable
				hresp, ok = api.HandlePermissionCheck(d.Auth.ID, guildId, "commands disable", rpc_messages.RpcCheckCommandOptions{
					CustomResolvedKittycatPerms: permLimits,
				})

				if !ok {
					return hresp
				}
			}

			updateCols = append(updateCols, "disabled")
			updateArgs = append(updateArgs, *value)
		}
	}

	if body.Perms != nil {
		value, clear, err := body.Perms.Get()

		if err != nil {
			return uapi.HttpResponse{
				Status: http.StatusBadRequest,
				Json: types.ApiError{
					Message: "Error parsing perms value: " + err.Error(),
				},
			}
		}

		// Check for permissions next
		hresp, ok = api.HandlePermissionCheck(d.Auth.ID, guildId, "commands modperms", rpc_messages.RpcCheckCommandOptions{
			CustomResolvedKittycatPerms: permLimits,
		})

		if !ok {
			return hresp
		}

		if clear {
			// Ensure user has permission to use the command
			hresp, ok = api.HandlePermissionCheck(d.Auth.ID, guildId, body.Command, rpc_messages.RpcCheckCommandOptions{
				CustomResolvedKittycatPerms: permLimits,
				CustomCommandConfiguration: &silverpelt.GuildCommandConfiguration{
					Command:  body.Command,
					Perms:    nil,
					Disabled: utils.Pointer(false),
				},
				Flags: rpc_messages.RpcCheckCommandOptionsFlagIgnoreCommandDisabled,
			})

			if !ok {
				return hresp
			}

			updateCols = append(updateCols, "perms")
			updateArgs = append(updateArgs, nil)
		} else {
			parsedValue, err := webutils.ParsePermissionChecks(d.Context, clusterId, guildId, value)

			if err != nil {
				return uapi.HttpResponse{
					Status: http.StatusBadRequest,
					Json: types.ApiError{
						Message: "Error parsing permission checks: " + err.Error(),
					},
				}
			}

			// Ensure user has permission to use the command
			hresp, ok = api.HandlePermissionCheck(d.Auth.ID, guildId, body.Command, rpc_messages.RpcCheckCommandOptions{
				CustomResolvedKittycatPerms: permLimits,
				CustomCommandConfiguration: &silverpelt.GuildCommandConfiguration{
					Command:  body.Command,
					Perms:    parsedValue,
					Disabled: utils.Pointer(false),
				},
				Flags: rpc_messages.RpcCheckCommandOptionsFlagIgnoreCommandDisabled,
			})

			if !ok {
				return hresp
			}

			updateCols = append(updateCols, "perms")
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

	// Update audit fields
	updateCols = append(updateCols, "last_updated_at", "last_updated_by")
	updateArgs = append(updateArgs, time.Now(), d.Auth.ID)

	// Create sql, insertParams is $N, $N+1... while updateParams are <col> = $N, <col2> = $N+1...
	var insertParams = make([]string, 0, len(updateCols))
	var updateParams = make([]string, 0, len(updateCols))
	var paramNo = 4 // 1, 2 and 3 are guild_id, command and created_by
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

	defer tx.Rollback(d.Context)

	var sqlString = "INSERT INTO guild_command_configurations (guild_id, command, created_by, " + strings.Join(updateCols, ", ") + ") VALUES ($1, $2, $3, " + strings.Join(insertParams, ",") + ") ON CONFLICT (guild_id, command) DO UPDATE SET " + strings.Join(updateParams, ", ") + " RETURNING id"

	// Execute sql
	updateArgs = append([]any{guildId, body.Command, d.Auth.ID}, updateArgs...) // $1 and $2
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
				Message: "Error updating command configuration: " + err.Error(),
			},
		}
	}

	// Fetch the gcc
	row, err := tx.Query(d.Context, "SELECT "+fullGuildCommandConfigurationCols+" FROM guild_command_configurations WHERE id = $1", id)

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Error fetching updated command configuration: " + err.Error(),
			},
		}
	}

	gcc, err := pgx.CollectOneRow(row, pgx.RowToStructByName[silverpelt.FullGuildCommandConfiguration])

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

	return uapi.HttpResponse{
		Json: gcc,
	}
}

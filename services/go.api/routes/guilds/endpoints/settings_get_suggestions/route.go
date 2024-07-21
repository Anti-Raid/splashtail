package settings_get_suggestions

import (
	"net/http"
	"strings"
	"time"

	"github.com/anti-raid/splashtail/core/go.std/silverpelt"
	"github.com/anti-raid/splashtail/core/go.std/utils/mewext"
	"github.com/anti-raid/splashtail/services/go.api/animusmagic_messages"
	"github.com/anti-raid/splashtail/services/go.api/api"
	"github.com/anti-raid/splashtail/services/go.api/state"
	"github.com/anti-raid/splashtail/services/go.api/types"
	"github.com/anti-raid/splashtail/services/go.api/webutils"
	"github.com/go-chi/chi/v5"
	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/ratelimit"
	"github.com/infinitybotlist/eureka/uapi"
	"go.uber.org/zap"
)

const suggestionsLimit = 10

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Settings Get Suggestions",
		Description: "Retrieve the suggestions for a setting",
		Req:         types.SettingsGetSuggestions{},
		Resp:        types.SettingsGetSuggestionsResponse{},
		Params: []docs.Parameter{
			{
				Name:        "guild_id",
				Description: "The guild ID to use",
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
		Bucket:      "settings_execute",
	}.Limit(d.Context, r)

	if err != nil {
		state.Logger.Error("Error while ratelimiting", zap.Error(err), zap.String("bucket", "settings_execute"))
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

	var body types.SettingsGetSuggestions

	hresp, ok = uapi.MarshalReqWithHeaders(r, &body, limit.Headers())

	if !ok {
		return hresp
	}

	if body.Module == "" || body.Setting == "" || body.Column == "" {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json: types.ApiError{
				Message: "Both `module`, `setting` and `column` must be provided",
			},
		}
	}

	if !body.Operation.Parse() {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json: types.ApiError{
				Message: "Invalid `operation` provided. `operation` must be one of: " + strings.Join(body.Operation.List(), ", "),
			},
		}
	}

	if body.Operation == "View" || body.Operation == "Delete" {
		return uapi.HttpResponse{
			Status: http.StatusNotImplemented,
			Json: types.ApiError{
				Message: "Suggestions can only be fetched for Create and Update operations",
			},
		}
	}

	clusterId, err = mewext.GetClusterIDFromGuildID(guildId, state.MewldInstanceList.Map, int(state.MewldInstanceList.ShardCount))

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

	hresp, ok = webutils.ClusterCheck(clusterId)

	if !ok {
		return hresp
	}

	modules, err := state.CachedAnimusMagicClient.GetClusterModules(d.Context, state.Rueidis, uint16(clusterId))

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Failed to fetch module list: " + err.Error(),
			},
		}
	}

	// Find the suggestions
	var suggestions *silverpelt.CanonicalColumnSuggestion
	var suggestionSetting *silverpelt.CanonicalConfigOption

	for _, module := range modules {
		if module.ID == body.Module {
			for _, setting := range module.ConfigOptions {
				if setting.ID == body.Setting {
					suggestionSetting = &setting

					for _, column := range setting.Columns {
						if column.ID == body.Column {
							suggestions = &column.Suggestions
							break
						}
					}
					break
				}
			}

			break
		}
	}

	if suggestionSetting == nil {
		return uapi.HttpResponse{
			Status: http.StatusNotFound,
			Json: types.ApiError{
				Message: "Could not find setting",
			},
		}

	}

	if suggestions == nil {
		return uapi.HttpResponse{
			Status: http.StatusNotFound,
			Json: types.ApiError{
				Message: "Module, setting or column not found",
			},
		}
	}

	// If static, just return the suggestions
	if suggestions.Static != nil {
		suggestionArray := make([]types.SettingsGetSuggestionSuggestion, len(suggestions.Static.Suggestions))

		for i, suggestion := range suggestions.Static.Suggestions {
			suggestionArray[i] = types.SettingsGetSuggestionSuggestion{
				ID:    suggestion,
				Value: suggestion,
			}
		}

		return uapi.HttpResponse{
			Json: types.SettingsGetSuggestionsResponse{
				Suggestions: suggestionArray,
			},
		}
	}

	// If dynamic, check permissions and return the suggestions for the module-defined suggestion
	//
	// SAFETY: All suggestions are defined by the module and not user input and so should be safe insofar as parameterized queries are used
	// to prevent SQL injection
	if suggestions.Dynamic != nil {
		opSpecific, ok := suggestionSetting.Operations.Get(body.Operation)

		if !ok {
			return uapi.HttpResponse{
				Status: http.StatusNotFound,
				Json: types.ApiError{
					Message: "Operation specific data not found. This operation may not be supported",
				},
			}
		}

		hresp, ok := api.HandlePermissionCheck(d.Auth.ID, guildId, opSpecific.CorrespondingCommand, animusmagic_messages.AmCheckCommandOptions{
			CustomResolvedKittycatPerms: api.PermLimits(d.Auth),
			Flags:                       animusmagic_messages.AmCheckCommandOptionsFlagIgnoreModuleDisabled,
		})

		if !ok {
			return hresp
		}

		sqlStmt := "SELECT " + suggestions.Dynamic.ValueColumn + "," + suggestions.Dynamic.IDColumn + " FROM " + suggestions.Dynamic.TableName + " WHERE guild_id = $1"
		sqlArgs := []any{guildId}

		if body.Filter != nil {
			sqlStmt += " AND (" + suggestions.Dynamic.ValueColumn + " ILIKE $2 OR " + suggestions.Dynamic.IDColumn + " ILIKE $2)"
			sqlArgs = append(sqlArgs, "%"+*body.Filter+"%")
		}

		// Add limit of suggestionsLimit
		sqlStmt += " LIMIT $2"
		sqlArgs = append(sqlArgs, suggestionsLimit)

		rows, err := state.Pool.Query(d.Context, sqlStmt, sqlArgs...)

		if err != nil {
			state.Logger.Error("Failed to fetch suggestions", zap.Error(err))
			return uapi.HttpResponse{
				Status: http.StatusInternalServerError,
				Json: types.ApiError{
					Message: "Failed to fetch suggestions: " + err.Error(),
				},
			}
		}

		defer rows.Close()

		var suggestionsArray []types.SettingsGetSuggestionSuggestion

		for rows.Next() {
			var value, id any

			err = rows.Scan(&value, &id)

			if err != nil {
				state.Logger.Error("Failed to scan suggestions", zap.Error(err))
				return uapi.HttpResponse{
					Status: http.StatusInternalServerError,
					Json: types.ApiError{
						Message: "Failed to scan suggestions: " + err.Error(),
					},
				}
			}

			suggestionsArray = append(suggestionsArray, types.SettingsGetSuggestionSuggestion{
				ID:    id,
				Value: value,
			})
		}

		return uapi.HttpResponse{
			Json: types.SettingsGetSuggestionsResponse{
				Suggestions: suggestionsArray,
			},
		}
	}

	return uapi.HttpResponse{
		Status: http.StatusNotImplemented,
		Json: types.ApiError{
			Message: "Suggestion type not implemented",
		},
	}
}

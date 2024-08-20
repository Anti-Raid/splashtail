package settings_execute

import (
	"net/http"
	"strings"
	"time"

	"github.com/go-chi/chi/v5"
	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/ratelimit"
	"github.com/infinitybotlist/eureka/uapi"
	"go.api/rpc_messages"
	"go.api/state"
	"go.api/types"
	"go.api/webutils"
	"go.std/utils/mewext"
	"go.uber.org/zap"
)

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Settings Execute",
		Description: "Execute a settings operation (list/create/update/delete). Note that, for dynamic values, all referenced variables must also be sent in the request.",
		Req:         types.SettingsExecute{},
		Resp:        types.SettingsExecuteResponse{},
		Params: []docs.Parameter{
			{
				Name:        "guild_id",
				Description: "The guild ID to execute the operation in",
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
		}
	}

	hresp, ok := webutils.ClusterCheck(clusterId)

	if !ok {
		return hresp
	}

	var body types.SettingsExecute

	hresp, ok = uapi.MarshalReqWithHeaders(r, &body, limit.Headers())

	if !ok {
		return hresp
	}

	if body.Module == "" || body.Setting == "" {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json: types.ApiError{
				Message: "Both `module` and `setting` must be provided",
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

	resp, err := webutils.SettingsOperation(
		d.Context,
		clusterId,
		guildId,
		d.Auth.ID,
		&rpc_messages.SettingsOperationRequest{
			Fields:  body.Fields,
			Op:      body.Operation,
			Module:  body.Module,
			Setting: body.Setting,
		},
	)

	if err != nil {
		state.Logger.Error("Error executing settings operation", zap.Error(err))
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Error executing settings operation: " + err.Error(),
			},
		}
	}

	if resp.Ok != nil {
		return uapi.HttpResponse{
			Json: types.SettingsExecuteResponse{
				Fields: resp.Ok.Fields,
			},
		}
	}

	if resp.PermissionError != nil {
		return uapi.HttpResponse{
			Status: http.StatusForbidden,
			Json:   resp.PermissionError.Res,
			Headers: map[string]string{
				"X-Error-Type": "permission_check",
			},
		}
	}

	if resp.Err != nil {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json:   resp.Err.Error,
			Headers: map[string]string{
				"X-Error-Type": "settings_error",
			},
		}
	}

	return uapi.HttpResponse{
		Status: http.StatusNotImplemented,
		Json: types.ApiError{
			Message: "Unknown response from animus magic [resp.Resp.SettingsOperation != nil, but unsupported res]",
		},
	}
}

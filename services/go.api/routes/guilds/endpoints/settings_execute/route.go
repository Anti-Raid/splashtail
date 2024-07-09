package settings_execute

import (
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/anti-raid/splashtail/core/go.std/animusmagic"
	"github.com/anti-raid/splashtail/core/go.std/silverpelt"
	"github.com/anti-raid/splashtail/core/go.std/types"
	"github.com/anti-raid/splashtail/core/go.std/utils"
	"github.com/anti-raid/splashtail/core/go.std/utils/mewext"
	"github.com/anti-raid/splashtail/services/go.api/animusmagic_messages"
	"github.com/anti-raid/splashtail/services/go.api/state"
	"github.com/anti-raid/splashtail/services/go.api/webutils"
	"github.com/go-chi/chi/v5"
	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/ratelimit"
	"github.com/infinitybotlist/eureka/uapi"
	orderedmap "github.com/wk8/go-ordered-map/v2"
	"go.uber.org/zap"
)

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Settings Execute",
		Description: "Execute a settings operation (list/create/update/delete)",
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

	resps, err := state.AnimusMagicClient.Request(
		d.Context,
		state.Rueidis,
		animusmagic_messages.BotAnimusMessage{
			SettingsOperation: &struct {
				Fields  orderedmap.OrderedMap[string, any] `json:"fields"`
				Op      silverpelt.CanonicalOperationType  `json:"op"`
				Module  string                             `json:"module"`
				Setting string                             `json:"setting"`
				GuildID string                             `json:"guild_id"`
				UserID  string                             `json:"user_id"`
			}{
				Fields:  body.Fields,
				Op:      body.Operation,
				Module:  body.Module,
				Setting: body.Setting,
				GuildID: guildId,
				UserID:  d.Auth.ID,
			},
		},
		&animusmagic.RequestOptions{
			ClusterID: utils.Pointer(uint16(clusterId)),
			To:        animusmagic.AnimusTargetBot,
			Op:        animusmagic.OpRequest,
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

	upr := resps[0]

	resp, err := animusmagic.ParseClientResponse[animusmagic_messages.BotAnimusResponse](upr)

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Error parsing response from animus magic: " + err.Error(),
			},
		}
	}

	if resp.ClientResp.Meta.Op == animusmagic.OpError {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Error executing operation: " + string(resp.ClientResp.RawPayload),
			},
		}
	}

	if resp.Resp == nil || resp.Resp.SettingsOperation == nil {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Error executing operation: [nil response]",
			},
		}
	}

	if resp.Resp.SettingsOperation.Res.Ok != nil {
		return uapi.HttpResponse{
			Json: types.SettingsExecuteResponse{
				Fields: resp.Resp.SettingsOperation.Res.Ok.Fields,
			},
		}
	}

	if resp.Resp.SettingsOperation.Res.PermissionError != nil {
		return uapi.HttpResponse{
			Status: http.StatusForbidden,
			Json:   resp.Resp.SettingsOperation.Res.PermissionError,
			Headers: map[string]string{
				"X-Error-Type": "permission_check",
			},
		}
	}

	if resp.Resp.SettingsOperation.Res.Err != nil {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json:   resp.Resp.SettingsOperation.Res.Err,
		}
	}

	return uapi.HttpResponse{
		Status: http.StatusNotImplemented,
		Json: types.ApiError{
			Message: "Unknown response from animus magic [resp.Resp.SettingsOperation != nil, but unsupported res]",
		},
	}
}

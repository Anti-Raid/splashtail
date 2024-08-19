package create_guild_task

import (
	"io"
	"net/http"
	"time"

	"go.api/api"
	"go.api/rpc_messages"
	"go.api/state"
	"go.api/types"
	"go.api/webutils"
	jobs "go.jobs"
	"go.std/ext_types"
	"go.std/utils/mewext"

	"github.com/go-chi/chi/v5"
	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/jsonimpl"
	"github.com/infinitybotlist/eureka/ratelimit"
	"github.com/infinitybotlist/eureka/uapi"
	"go.uber.org/zap"
)

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Create Guild Task",
		Description: "Creates a task for a guild. Returns the task data if this is successful",
		Params: []docs.Parameter{
			{
				Name:        "guild_id",
				Description: "Guild ID",
				Required:    true,
				In:          "path",
				Schema:      docs.IdSchema,
			},
			{
				Name:        "name",
				Description: "The name of the task",
				Required:    true,
				In:          "path",
				Schema:      docs.IdSchema,
			},
			{
				Name:        "guild_id",
				Description: "The ID of the guild to run the task in",
				Required:    true,
				In:          "query",
				Schema:      docs.IdSchema,
			},
		},
		Req:  "The tasks fields",
		Resp: ext_types.TaskCreateResponse{},
	}
}

func Route(d uapi.RouteData, r *http.Request) uapi.HttpResponse {
	limit, err := ratelimit.Ratelimit{
		Expiry:      1 * time.Hour,
		MaxRequests: 50,
		Bucket:      "create_task",
		Identifier: func(r *http.Request) string {
			return d.Auth.ID
		},
	}.Limit(d.Context, r)

	if err != nil {
		state.Logger.Error("Error while ratelimiting", zap.Error(err), zap.String("bucket", "create_task"))
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

	taskName := chi.URLParam(r, "name")

	if taskName == "" {
		return uapi.HttpResponse{
			Json: types.ApiError{
				Message: "Missing name",
			},
			Status: http.StatusBadRequest,
		}
	}

	guildId := chi.URLParam(r, "guild_id")

	if guildId == "" {
		return uapi.HttpResponse{
			Json: types.ApiError{
				Message: "Missing guild_id",
			},
			Status: http.StatusBadRequest,
		}
	}

	clusterId, err := mewext.GetClusterIDFromGuildID(guildId, state.MewldInstanceList.Map, int(state.MewldInstanceList.ShardCount))

	if err != nil {
		state.Logger.Error("Error getting cluster ID", zap.Error(err))
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Error getting cluster ID:" + err.Error(),
			},
		}
	}

	baseTaskDef, ok := jobs.TaskDefinitionRegistry[taskName]

	if !ok {
		return uapi.HttpResponse{
			Json: types.ApiError{
				Message: "Unknown task name",
			},
			Status: http.StatusBadRequest,
		}
	}

	tBytes, err := io.ReadAll(r.Body)

	if err != nil {
		state.Logger.Error("Error reading body", zap.Error(err))
		return uapi.DefaultResponse(http.StatusInternalServerError)
	}

	task := baseTaskDef // Copy task

	err = jsonimpl.Unmarshal(tBytes, &task)

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json: types.ApiError{
				Message: "Invalid task data: " + err.Error(),
			},
		}
	}

	// Check permissions
	permLimits := api.PermLimits(d.Auth)
	resp, ok := api.HandlePermissionCheck(d.Auth.ID, guildId, task.CorrespondingBotCommand_Create(), rpc_messages.RpcCheckCommandOptions{
		CustomResolvedKittycatPerms: permLimits,
	})

	if !ok {
		return resp
	}

	var data map[string]any

	err = jsonimpl.Unmarshal(tBytes, &data)

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json: types.ApiError{
				Message: "Invalid task data: " + err.Error(),
			},
		}
	}

	str, err := webutils.JobserverSpawnTask(d.Context, clusterId, &rpc_messages.JobserverSpawnTask{
		Name:    taskName,
		Data:    data,
		Create:  true,
		Execute: true,
		UserID:  d.Auth.ID,
	})

	if err != nil {
		state.Logger.Error("Error while spawning task on jobserver", zap.Error(err))
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Error while spawning task on jobserver: " + err.Error(),
			},
		}
	}

	return uapi.HttpResponse{
		Json: ext_types.PartialTask{
			TaskID: str.TaskID,
		},
	}
}

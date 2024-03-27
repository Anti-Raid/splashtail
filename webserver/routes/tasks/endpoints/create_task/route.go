package create_task

import (
	"fmt"
	"io"
	"net/http"
	"time"

	"github.com/anti-raid/splashtail/splashcore/animusmagic"
	"github.com/anti-raid/splashtail/splashcore/types"
	"github.com/anti-raid/splashtail/splashcore/utils"
	"github.com/anti-raid/splashtail/splashcore/utils/mewext"
	"github.com/anti-raid/splashtail/tasks"
	"github.com/anti-raid/splashtail/webserver/api"
	"github.com/anti-raid/splashtail/webserver/state"
	jsoniter "github.com/json-iterator/go"

	"github.com/go-chi/chi/v5"
	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/ratelimit"
	"github.com/infinitybotlist/eureka/uapi"
	"go.uber.org/zap"
)

var json = jsoniter.ConfigFastest

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Create Task",
		Description: "Creates a task. Returns the task data if this is successful",
		Params: []docs.Parameter{
			{
				Name:        "id",
				Description: "User ID",
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
		Resp: types.TaskCreateResponse{},
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

	guildId := r.URL.Query().Get("guild_id")

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

	baseTaskDef, ok := tasks.TaskDefinitionRegistry[taskName]

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

	err = json.Unmarshal(tBytes, &task)

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json: types.ApiError{
				Message: "Invalid task data: " + err.Error(),
			},
		}
	}

	taskInfo := task.Info()

	// Check permissions
	resp, ok := api.HandlePermissionCheck(d.Auth.ID, guildId, taskInfo.CorrespondingBotCommand, api.PermLimits(d.Auth))

	if !ok {
		return resp
	}

	var data map[string]any

	err = json.Unmarshal(tBytes, &data)

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json: types.ApiError{
				Message: "Invalid task data: " + err.Error(),
			},
		}
	}

	// Make animus magic request
	animusResp, err := state.AnimusMagicClient.Request(
		d.Context,
		state.Rueidis,
		animusmagic.JobserverMessage{
			SpawnTask: &struct {
				Name    string                 `json:"name"`
				Data    map[string]interface{} `json:"data"`
				Create  bool                   `json:"create"`
				Execute bool                   `json:"execute"`
				TaskID  string                 `json:"task_id"`
				UserID  string                 `json:"user_id"`
			}{
				Name:    taskName,
				Data:    data,
				Create:  true,
				Execute: true,
				UserID:  d.Auth.ID,
			},
		},
		&animusmagic.RequestOptions{
			ClusterID:             utils.Pointer(uint16(clusterId)),
			ExpectedResponseCount: 1,
			To:                    animusmagic.AnimusTargetJobserver,
		},
	)

	if err != nil {
		state.Logger.Error("Error while making animus request", zap.Error(err))
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Error while making animus request: " + err.Error(),
			},
		}
	}

	if len(animusResp) != 1 {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: fmt.Sprintf("Unexpected response count: %d", len(animusResp)),
			},
		}
	}

	acr := animusResp[0]

	parsedAnimusResp, err := animusmagic.ParseClientResponse[animusmagic.JobserverResponse](acr)

	if err != nil {
		state.Logger.Error("Error while parsing animus response", zap.Error(err))
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Error while parsing animus response: " + err.Error(),
			},
		}
	}

	if parsedAnimusResp.Resp.SpawnTask == nil {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Unexpected response",
			},
		}
	}

	return uapi.HttpResponse{
		Json: types.TaskCreateResponse{
			TaskID:   parsedAnimusResp.Resp.SpawnTask.TaskID,
			TaskInfo: taskInfo,
		},
	}
}

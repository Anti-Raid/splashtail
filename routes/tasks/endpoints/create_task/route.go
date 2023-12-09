package create_task

import (
	"encoding/json"
	"net/http"
	"splashtail/state"
	"splashtail/tasks"
	"splashtail/types"
	"time"

	"github.com/go-chi/chi/v5"
	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/ratelimit"
	"github.com/infinitybotlist/eureka/uapi"
	"go.uber.org/zap"
)

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Get Task",
		Description: "Gets a task. Returns the task data if this is successful",
		Params: []docs.Parameter{
			{
				Name:        "id",
				Description: "User/Server ID",
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

	baseTaskDef, ok := tasks.TaskDefinitionRegistry[taskName]

	if !ok {
		return uapi.HttpResponse{
			Json: types.ApiError{
				Message: "Unknown task name",
			},
			Status: http.StatusBadRequest,
		}
	}

	task := baseTaskDef // Copy task

	err = json.NewDecoder(r.Body).Decode(&task)

	if err != nil {
		return uapi.HttpResponse{
			Json: types.ApiError{
				Message: "Error decoding task: " + err.Error(),
			},
			Status: http.StatusBadRequest,
		}
	}

	tInfo := task.Info()

	// Access Control check
	if tInfo.TaskFor != nil {
		if tInfo.TaskFor.ID == "" || tInfo.TaskFor.TargetType == "" {
			return uapi.HttpResponse{
				Status: http.StatusInternalServerError,
				Json:   types.ApiError{Message: "Invalid task.TaskFor. Missing ID or TargetType"},
			}
		}

		if tInfo.TaskFor.TargetType != d.Auth.TargetType {
			return uapi.HttpResponse{
				Status: http.StatusForbidden,
				Json:   types.ApiError{Message: "This task has a for of " + tInfo.TaskFor.TargetType + " but you are authenticated as a" + d.Auth.TargetType + "!"},
			}
		}

		if tInfo.TaskFor.ID != d.Auth.ID {
			return uapi.HttpResponse{
				Status: http.StatusForbidden,
				Json:   types.ApiError{Message: "You are not authorized to fetch this task!"},
			}
		}
	}

	tcr, err := tasks.CreateTask(state.Context, task)

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json:   types.ApiError{Message: "Error creating task: " + err.Error()},
		}
	}

	go tasks.NewTask(tcr, task)

	return uapi.HttpResponse{
		Json: tcr,
	}
}

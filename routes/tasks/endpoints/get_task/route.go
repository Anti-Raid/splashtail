package get_task

import (
	"errors"
	"net/http"
	"strings"

	"github.com/anti-raid/splashtail/db"
	"github.com/anti-raid/splashtail/state"
	"github.com/anti-raid/splashtail/tasks"
	types "github.com/anti-raid/splashtail/types"

	"github.com/go-chi/chi/v5"
	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/uapi"
	"github.com/jackc/pgx/v5"
	"go.uber.org/zap"
)

var (
	taskColsArr = db.GetCols(types.Task{})
	taskColsStr = strings.Join(taskColsArr, ", ")
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
				Name:        "tid",
				Description: "The task ID",
				Required:    true,
				In:          "path",
				Schema:      docs.IdSchema,
			},
			{
				Name:        "task_key",
				Description: "The task key if required. This is used to authenticate the request.",
				Required:    false,
				In:          "query",
				Schema:      docs.IdSchema,
			},
		},
		Resp: types.Task{},
	}
}

func Route(d uapi.RouteData, r *http.Request) uapi.HttpResponse {
	// Check that the user owns the task
	taskId := chi.URLParam(r, "tid")
	entityId := chi.URLParam(r, "id")

	if taskId == "" {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json:   types.ApiError{Message: "task id is required"},
		}
	}

	if d.Auth.ID != entityId {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json:   types.ApiError{Message: "Internal server error: entityId != d.Auth.ID"},
		}
	}

	// Delete expired tasks first
	_, err := state.Pool.Exec(d.Context, "DELETE FROM tasks WHERE created_at + expiry < NOW()")

	if err != nil {
		state.Logger.Error("Failed to delete expired tasks [db delete]", zap.Error(err))
		return uapi.DefaultResponse(http.StatusInternalServerError)
	}

	row, err := state.Pool.Query(d.Context, "SELECT "+taskColsStr+" FROM tasks WHERE task_id = $1", taskId)

	if err != nil {
		state.Logger.Error("Failed to fetch task [db fetch]", zap.Error(err))
		return uapi.DefaultResponse(http.StatusInternalServerError)
	}

	task, err := pgx.CollectOneRow(row, pgx.RowToStructByName[types.Task])

	if errors.Is(err, pgx.ErrNoRows) {
		return uapi.HttpResponse{
			Status: http.StatusNotFound,
			Json:   types.ApiError{Message: "Task not found"},
		}
	}

	if err != nil {
		state.Logger.Error("Failed to fetch task [db fetch]", zap.Error(err))
		return uapi.DefaultResponse(http.StatusInternalServerError)
	}

	if task.TaskKey != nil {
		if *task.TaskKey != r.URL.Query().Get("task_key") {
			return uapi.HttpResponse{
				Status: http.StatusUnauthorized,
				Json:   types.ApiError{Message: "Invalid task key"},
			}
		}
	}

	if !task.AllowUnauthenticated {
		if d.Auth.ID == "" {
			return uapi.HttpResponse{
				Status: http.StatusUnauthorized,
				Json:   types.ApiError{Message: "This task requires authentication"},
			}
		}

		if task.TaskForRaw != nil {
			task.TaskFor = tasks.ParseTaskFor(*task.TaskForRaw)

			if task.TaskFor == nil {
				return uapi.HttpResponse{
					Status: http.StatusInternalServerError,
					Json:   types.ApiError{Message: "Invalid task.TaskFor. Parsing error occurred"},
				}
			}

			if task.TaskFor.ID == "" || task.TaskFor.TargetType == "" {
				return uapi.HttpResponse{
					Status: http.StatusInternalServerError,
					Json:   types.ApiError{Message: "Invalid task.TaskFor. Missing ID or TargetType"},
				}
			}

			if task.TaskFor.TargetType != d.Auth.TargetType {
				return uapi.HttpResponse{
					Status: http.StatusForbidden,
					Json:   types.ApiError{Message: "This task has a for of " + task.TaskFor.TargetType + " but you are authenticated as a" + d.Auth.TargetType + "!"},
				}
			}

			if task.TaskFor.ID != d.Auth.ID {
				return uapi.HttpResponse{
					Status: http.StatusForbidden,
					Json:   types.ApiError{Message: "You are not authorized to fetch this task!"},
				}
			}
		}
	}

	return uapi.HttpResponse{
		Status: http.StatusOK,
		Json:   task,
	}
}

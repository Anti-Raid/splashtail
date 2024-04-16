package get_task

import (
	"errors"
	"net/http"
	"strings"

	"github.com/anti-raid/splashtail/splashcore/structparser/db"
	types "github.com/anti-raid/splashtail/splashcore/types"
	"github.com/anti-raid/splashtail/jobs/tasks"
	"github.com/anti-raid/splashtail/webserver/state"

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
		},
		Resp: types.Task{},
	}
}

func Route(d uapi.RouteData, r *http.Request) uapi.HttpResponse {
	// Check that the user owns the task
	taskId := chi.URLParam(r, "tid")

	if taskId == "" {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json:   types.ApiError{Message: "task id is required"},
		}
	}

	// Delete expired tasks first
	_, err := state.Pool.Exec(d.Context, "DELETE FROM tasks WHERE created_at + expiry < NOW()")

	if err != nil {
		state.Logger.Error("Failed to delete expired tasks [db delete]", zap.Error(err))
		return uapi.DefaultResponse(http.StatusInternalServerError)
	}

	var row pgx.Rows

	if d.Auth.ID != "" {
		taskForStrPtr, err := tasks.FormatTaskFor(&types.TaskFor{
			ID:         d.Auth.ID,
			TargetType: d.Auth.TargetType,
		})

		if err != nil {
			state.Logger.Error("Failed to format task for [task format]", zap.Error(err))
			return uapi.HttpResponse{
				Status: http.StatusInternalServerError,
				Json:   types.ApiError{Message: "Internal server error: Failed to format task for: " + err.Error()},
			}
		}

		row, err = state.Pool.Query(d.Context, "SELECT "+taskColsStr+" FROM tasks WHERE task_id = $1 AND (task_for IS NULL OR task_for = $2)", taskId, taskForStrPtr)

		if err != nil {
			state.Logger.Error("Failed to fetch task [db fetch]", zap.Error(err))
			return uapi.DefaultResponse(http.StatusInternalServerError)
		}
	} else {
		row, err = state.Pool.Query(d.Context, "SELECT "+taskColsStr+" FROM tasks WHERE task_id = $1 AND task_for IS NULL", taskId)

		if err != nil {
			state.Logger.Error("Failed to fetch task [db fetch]", zap.Error(err))
			return uapi.DefaultResponse(http.StatusInternalServerError)
		}
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

	return uapi.HttpResponse{
		Status: http.StatusOK,
		Json:   task,
	}
}

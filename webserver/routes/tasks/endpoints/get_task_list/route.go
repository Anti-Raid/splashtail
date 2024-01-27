package get_task_list

import (
	"errors"
	"net/http"
	"strings"

	"github.com/anti-raid/splashtail/db"
	"github.com/anti-raid/splashtail/tasks"
	"github.com/anti-raid/splashtail/types"
	"github.com/anti-raid/splashtail/webserver/state"
	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/uapi"
	"github.com/jackc/pgx/v5"
	"go.uber.org/zap"
)

var (
	taskColsArr = db.GetCols(types.PartialTask{})
	taskColsStr = strings.Join(taskColsArr, ", ")
)

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Get Task List",
		Description: "Gets the list of all tasks as a PartialTask object",
		Params: []docs.Parameter{
			{
				Name:        "id",
				Description: "User/Server ID",
				Required:    true,
				In:          "path",
				Schema:      docs.IdSchema,
			},
		},
		Resp: types.TaskListResponse{},
	}
}

func Route(d uapi.RouteData, r *http.Request) uapi.HttpResponse {
	// Delete expired tasks first
	_, err := state.Pool.Exec(d.Context, "DELETE FROM tasks WHERE created_at + expiry < NOW()")

	if err != nil {
		state.Logger.Error("Failed to delete expired tasks [db delete]", zap.Error(err))
		return uapi.DefaultResponse(http.StatusInternalServerError)
	}

	var taskForStr *string

	if d.Auth.ID != "" {
		taskForStr, err = tasks.FormatTaskFor(&types.TaskFor{
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
	}

	row, err := state.Pool.Query(d.Context, "SELECT "+taskColsStr+" FROM tasks WHERE task_for IS NULL OR task_for = $1", taskForStr)

	if err != nil {
		state.Logger.Error("Failed to fetch task [db fetch]", zap.Error(err))
		return uapi.DefaultResponse(http.StatusInternalServerError)
	}

	tasks, err := pgx.CollectRows(row, pgx.RowToStructByName[types.PartialTask])

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
		Json:   types.TaskListResponse{Tasks: tasks},
	}
}

package get_guild_task

import (
	"errors"
	"net/http"
	"strings"

	"go.api/api"
	"go.api/rpc_messages"
	"go.api/state"
	"go.api/types"
	jobs "go.jobs"
	jobtypes "go.jobs/types"
	"go.std/structparser/db"

	"github.com/go-chi/chi/v5"
	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/uapi"
	"github.com/jackc/pgx/v5"
	"go.uber.org/zap"
)

var (
	taskColsArr = db.GetCols(jobtypes.Task{})
	taskColsStr = strings.Join(taskColsArr, ", ")
)

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Get Guild Task",
		Description: "Gets a task created on a guild",
		Params: []docs.Parameter{
			{
				Name:        "guild_id",
				Description: "Guild ID",
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
		Resp: jobtypes.Task{},
	}
}

func Route(d uapi.RouteData, r *http.Request) uapi.HttpResponse {
	taskId := chi.URLParam(r, "tid")
	guildId := chi.URLParam(r, "guild_id")

	if taskId == "" || guildId == "" {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json:   types.ApiError{Message: "tid/guild_id is required"},
		}
	}

	// Delete expired tasks first
	_, err := state.Pool.Exec(d.Context, "DELETE FROM tasks WHERE created_at + expiry < NOW()")

	if err != nil {
		state.Logger.Error("Failed to delete expired tasks [db delete]", zap.Error(err))
		return uapi.DefaultResponse(http.StatusInternalServerError)
	}

	var row pgx.Rows
	row, err = state.Pool.Query(d.Context, "SELECT "+taskColsStr+" FROM tasks WHERE id = $1 AND guild_id = $2", taskId, guildId)

	if err != nil {
		state.Logger.Error("Failed to fetch task [db fetch]", zap.Error(err))
		return uapi.DefaultResponse(http.StatusInternalServerError)
	}

	task, err := pgx.CollectOneRow(row, pgx.RowToStructByName[jobtypes.Task])

	if errors.Is(err, pgx.ErrNoRows) {
		return uapi.HttpResponse{
			Status: http.StatusNotFound,
			Json:   types.ApiError{Message: "Task not found"},
		}
	}

	if err != nil {
		state.Logger.Error("Failed to fetch job [db fetch]", zap.Error(err))
		return uapi.DefaultResponse(http.StatusInternalServerError)
	}

	baseJobImpl, ok := jobs.JobImplRegistry[task.Name]

	if !ok {
		return uapi.HttpResponse{
			Json: types.ApiError{
				Message: "Internal Error: Unknown job name",
			},
			Status: http.StatusInternalServerError,
		}
	}

	// Check permissions
	permLimits := api.PermLimits(d.Auth)
	resp, ok := api.HandlePermissionCheck(d.Auth.ID, guildId, baseJobImpl.CorrespondingBotCommand_View(), rpc_messages.RpcCheckCommandOptions{
		CustomResolvedKittycatPerms: permLimits,
	})

	if !ok {
		return resp
	}

	return uapi.HttpResponse{
		Status: http.StatusOK,
		Json:   task,
	}
}

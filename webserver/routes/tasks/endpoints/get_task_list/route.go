package get_task_list

import (
	"errors"
	"net/http"
	"strings"

	"github.com/anti-raid/splashtail/jobs/tasks"
	"github.com/anti-raid/splashtail/splashcore/animusmagic"
	"github.com/anti-raid/splashtail/splashcore/structparser/db"
	"github.com/anti-raid/splashtail/splashcore/types"
	"github.com/anti-raid/splashtail/webserver/api"
	"github.com/anti-raid/splashtail/webserver/state"
	"github.com/go-chi/chi/v5"
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
				Description: "User ID",
				Required:    true,
				In:          "path",
				Schema:      docs.IdSchema,
			},
			{
				Name:        "guild_id",
				Description: "Guild ID",
				Required:    true,
				In:          "path",
				Schema:      docs.IdSchema,
			},
			{
				Name:        "error_if_no_permissions",
				Description: "Whether or not to return an error if the user does not have permission to view a task on the task list",
				Required:    true,
				In:          "query",
				Schema:      docs.IdSchema,
			},
			{
				Name:        "error_on_unknown_task",
				Description: "Whether or not to return an error if a task on the task list is unknown. Otherwise, the task will simply not be returned",
				Required:    true,
				In:          "query",
				Schema:      docs.IdSchema,
			},
		},
		Resp: types.TaskListResponse{},
	}
}

func Route(d uapi.RouteData, r *http.Request) uapi.HttpResponse {
	guildId := chi.URLParam(r, "id")

	if guildId == "" {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json:   types.ApiError{Message: "Guild ID is required"},
		}
	}

	errorIfNoPermissions := r.URL.Query().Get("error_if_no_permissions") == "true"
	errorOnUnknownTask := r.URL.Query().Get("error_on_unknown_task") == "true"

	// Delete expired tasks first
	_, err := state.Pool.Exec(d.Context, "DELETE FROM tasks WHERE created_at + expiry < NOW()")

	if err != nil {
		state.Logger.Error("Failed to delete expired tasks [db delete]", zap.Error(err))
		return uapi.DefaultResponse(http.StatusInternalServerError)
	}

	row, err := state.Pool.Query(d.Context, "SELECT "+taskColsStr+" FROM tasks WHERE guild_id = $1", guildId)

	if err != nil {
		state.Logger.Error("Failed to fetch task [db fetch]", zap.Error(err))
		return uapi.DefaultResponse(http.StatusInternalServerError)
	}

	tasksFetched, err := pgx.CollectRows(row, pgx.RowToStructByName[types.PartialTask])

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

	var checksDone = map[string]bool{}
	var parsedTasks = []types.PartialTask{}
	for _, task := range tasksFetched {
		// NOTE/WARNING: This is a fastpath that depends on the assumption that the corresponding bot command
		// does not change for a task. If this assumption is broken, this code will break if the corresponding
		// command changes while in this loop
		if _, ok := checksDone[task.TaskName]; ok {
			parsedTasks = append(parsedTasks, task)
		}

		baseTaskDef, ok := tasks.TaskDefinitionRegistry[task.TaskName]

		if !ok {
			if errorOnUnknownTask {
				return uapi.HttpResponse{
					Json: types.ApiError{
						Message: "Internal Error: Unknown task name",
					},
					Status: http.StatusInternalServerError,
				}
			} else {
				continue
			}
		}

		// Check permissions
		permLimits := api.PermLimits(d.Auth)
		resp, ok := api.HandlePermissionCheck(d.Auth.ID, guildId, baseTaskDef.CorrespondingBotCommand_View(), animusmagic.AmCheckCommandOptions{
			CustomResolvedKittycatPerms: permLimits,
		})

		if !ok {
			if errorIfNoPermissions {
				return resp
			}
			continue
		}

		checksDone[task.TaskName] = true
		parsedTasks = append(parsedTasks, task)
	}

	return uapi.HttpResponse{
		Status: http.StatusOK,
		Json:   types.TaskListResponse{Tasks: parsedTasks},
	}
}

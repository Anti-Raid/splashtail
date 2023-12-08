package ioauth_download_task

import (
	"encoding/json"
	"errors"
	"net/http"
	"splashtail/db"
	"splashtail/state"
	"splashtail/tasks"
	types "splashtail/types"
	"strings"
	"time"

	"github.com/bwmarrin/discordgo"
	"github.com/go-chi/chi/v5"
	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/uapi"
	"github.com/jackc/pgx/v5"
	"go.uber.org/zap"
)

var (
	taskColsArr = db.GetCols(types.Task{})
	taskColsStr = strings.Join(taskColsArr, ", ")

	// TODO: Change once we have a proper perms system
	//
	// At least one of these perms is required to download a task
	neededPerms = []int64{
		discordgo.PermissionManageServer,
		discordgo.PermissionAdministrator,
	}
)

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Get IOAuth Download Link",
		Description: "Gets the download link to a tasks output. Note that this endpoint, like most other IOAuth flows, is *not* meant to be used outside of the bot.",
		Params: []docs.Parameter{
			{
				Name:        "id",
				Description: "Task ID",
				Required:    true,
				In:          "path",
				Schema:      docs.IdSchema,
			},
			{
				Name:        "no_redirect",
				Description: "Whether or not to avoid the redirect and merely return the link",
				Required:    true,
				In:          "query",
				Schema:      docs.IdSchema,
			},
		},
		Resp: types.Task{},
	}
}

func Route(d uapi.RouteData, r *http.Request) uapi.HttpResponse {
	iotok := r.URL.Query().Get("ioauth")

	if iotok == "" {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json: types.ApiError{
				Message: "Missing IOAuth token",
			},
		}
	}

	taskId := chi.URLParam(r, "id")

	if taskId == "" {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json:   types.ApiError{Message: "task id is required"},
		}
	}

	// Get the ioauth token
	resp, err := state.Rueidis.Do(d.Context, state.Rueidis.B().Get().Key("ioauth:{"+iotok+"}").Build()).AsBytes()

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Internal Server Error [while checking ioauth token]: " + err.Error(),
			},
		}
	}

	if resp == nil {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json: types.ApiError{
				Message: "Invalid IOAuth token",
			},
		}
	}

	var iot types.IOAuthOutput

	err = json.Unmarshal(resp, &iot)

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Internal Server Error [while parsing ioauth token]: " + err.Error(),
			},
		}
	}

	// Fetch the task
	// Delete expired tasks first
	_, err = state.Pool.Exec(d.Context, "DELETE FROM tasks WHERE created_at + expiry < NOW()")

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

	if task.TaskInfo == nil {
		return uapi.HttpResponse{
			Status: http.StatusNotFound,
			Json:   types.ApiError{Message: "Task info not found"},
		}
	}

	if task.Output == nil {
		return uapi.HttpResponse{
			Status: http.StatusNotFound,
			Json:   types.ApiError{Message: "Task output not found"},
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

		if task.TaskFor.TargetType == types.TargetTypeUser {
			if iot.DiscordUser.ID != task.TaskFor.ID {
				return uapi.HttpResponse{
					Status: http.StatusForbidden,
					Json:   types.ApiError{Message: "You are not authorized to fetch this task [TargetType = User]!"},
				}
			}
		} else if task.TaskFor.TargetType == types.TargetTypeServer {
			if iot.UserGuilds == nil {
				return uapi.HttpResponse{
					Status: http.StatusForbidden,
					Json:   types.ApiError{Message: "You are not authorized to fetch this task [UserGuilds is nil]!"},
				}
			}

			found := false

			for _, guild := range iot.UserGuilds {
				if guild.ID == task.TaskFor.ID {
					var hasPerms bool
					for _, perm := range neededPerms {
						if guild.Permissions&perm == perm {
							hasPerms = true
							break
						}
					}

					if !hasPerms {
						return uapi.HttpResponse{
							Status: http.StatusForbidden,
							Json:   types.ApiError{Message: "You are not authorized to fetch this task [Missing perms]!"},
						}
					}

					found = true
					break
				}
			}

			if !found {
				return uapi.HttpResponse{
					Status: http.StatusForbidden,
					Json:   types.ApiError{Message: "You are not authorized to fetch this task [TargetType = Server, not in server]!"},
				}
			}
		} else {
			return uapi.HttpResponse{
				Status: http.StatusNotImplemented,
				Json:   types.ApiError{Message: "Downloading is not supported for this target type [TargetType = Server]"},
			}
		}
	}

	// Now get URL
	url, err := state.ObjectStorage.GetUrl(d.Context, tasks.GetPathFromOutput(task.TaskInfo, task.Output), task.Output.Filename, 10*time.Minute)

	if err != nil {
		state.Logger.Error("Failed to get url for task", zap.Error(err))
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json:   types.ApiError{Message: "Failed to get url for task: " + err.Error()},
		}
	}

	if r.URL.Query().Get("no_redirect") == "true" {
		return uapi.HttpResponse{
			Status: http.StatusOK,
			Json: types.ApiError{
				Message: url.String(),
			},
		}
	} else {
		return uapi.HttpResponse{
			Status:   http.StatusFound,
			Redirect: url.String(),
		}
	}
}

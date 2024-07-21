package ioauth_download_task

import (
	"bytes"
	"encoding/json"
	"errors"
	"html/template"
	"net/http"
	"strings"
	"time"

	jobs "github.com/anti-raid/splashtail/core/go.jobs"
	"github.com/anti-raid/splashtail/core/go.std/ext_types"
	"github.com/anti-raid/splashtail/core/go.std/splashcore"
	"github.com/anti-raid/splashtail/core/go.std/structparser/db"
	"github.com/anti-raid/splashtail/services/go.api/animusmagic_messages"
	"github.com/anti-raid/splashtail/services/go.api/api"
	"github.com/anti-raid/splashtail/services/go.api/state"
	types "github.com/anti-raid/splashtail/services/go.api/types"

	"github.com/go-chi/chi/v5"
	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/uapi"
	"github.com/jackc/pgx/v5"
	"go.uber.org/zap"
)

var (
	taskColsArr = db.GetCols(ext_types.Task{})
	taskColsStr = strings.Join(taskColsArr, ", ")
)

var downloadTemplate = template.Must(template.New("download").Parse(`<!DOCTYPE html>
<html>
	Your download should start in a moment. If not, <a href="{{.URL}}">click here</a>
	<script>
		if(window.opener) {
			window.opener.postMessage("dl:{{.URL}}", {{.Domain}});
		} else if(window.parent) {
			window.parent.postMessage("dl::{{.URL}}", {{.Domain}});
		}
		window.location.href = "{{.URL}}";
	</script>
</html>
`))

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Get IOAuth Download Link",
		Description: "Gets the download link to a tasks output.",
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
				Description: "Whether or not to avoid the redirect/text response and merely return the link",
				Required:    true,
				In:          "query",
				Schema:      docs.IdSchema,
			},
		},
		Resp: ext_types.Task{},
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

	task, err := pgx.CollectOneRow(row, pgx.RowToStructByName[ext_types.Task])

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

	if task.Output == nil {
		return uapi.HttpResponse{
			Status: http.StatusNotFound,
			Json:   types.ApiError{Message: "Task output not found"},
		}
	}

	taskDef, ok := jobs.TaskDefinitionRegistry[task.TaskName]

	if !ok {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json:   types.ApiError{Message: "Task definition not found"},
		}
	}

	if task.TaskForRaw != nil {
		task.TaskFor = jobs.ParseTaskFor(*task.TaskForRaw)

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

		if task.TaskFor.TargetType == splashcore.TargetTypeUser {
			if iot.DiscordUser.ID != task.TaskFor.ID {
				return uapi.HttpResponse{
					Status: http.StatusForbidden,
					Json:   types.ApiError{Message: "You are not authorized to fetch this task [TargetType = User]!"},
				}
			}
		} else if task.TaskFor.TargetType == splashcore.TargetTypeServer {
			// Check permissions
			resp, ok := api.HandlePermissionCheck(iot.DiscordUser.ID, task.TaskFor.ID, taskDef.CorrespondingBotCommand_Download(), animusmagic_messages.AmCheckCommandOptions{})

			if !ok {
				return resp
			}

		} else {
			return uapi.HttpResponse{
				Status: http.StatusNotImplemented,
				Json:   types.ApiError{Message: "Downloading is not supported for this target type [TargetType = " + task.TaskFor.TargetType + "]"},
			}
		}
	}

	// Now get URL
	url, err := state.ObjectStorage.GetUrl(d.Context, jobs.GetPathFromOutput(task.TaskID, taskDef, task.Output), task.Output.Filename, 10*time.Minute)

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
		var buf bytes.Buffer
		err := downloadTemplate.Execute(&buf, map[string]any{
			"URL":    url.String(),
			"Domain": state.Config.Sites.Frontend.Parse(),
		})

		if err != nil {
			state.Logger.Error("Failed to execute download template", zap.Error(err))
			return uapi.HttpResponse{
				Status: http.StatusInternalServerError,
				Json:   types.ApiError{Message: "Failed to execute download template: " + err.Error()},
			}
		}

		return uapi.HttpResponse{
			Status: http.StatusFound,
			Bytes:  buf.Bytes(),
			Headers: map[string]string{
				"Content-Type": "text/html, charset=utf-8",
			},
		}
	}
}

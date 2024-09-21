package ioauth_download_task

import (
	"bytes"
	"encoding/json"
	"errors"
	"html/template"
	"net/http"
	"strings"
	"time"

	"go.api/api"
	"go.api/rpc_messages"
	"go.api/state"
	"go.api/types"
	jobs "go.jobs"
	jobtypes "go.jobs/types"
	"go.std/splashcore"
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
		Resp: jobtypes.Task{},
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

	id := chi.URLParam(r, "id")

	if id == "" {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json:   types.ApiError{Message: "`id` is required"},
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

	row, err := state.Pool.Query(d.Context, "SELECT "+taskColsStr+" FROM tasks WHERE id = $1", id)

	if err != nil {
		state.Logger.Error("Failed to fetch task [db fetch]", zap.Error(err))
		return uapi.DefaultResponse(http.StatusInternalServerError)
	}

	job, err := pgx.CollectOneRow(row, pgx.RowToStructByName[jobtypes.Task])

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

	if job.Output == nil {
		return uapi.HttpResponse{
			Status: http.StatusNotFound,
			Json:   types.ApiError{Message: "Task output not found"},
		}
	}

	taskDef, ok := jobs.JobImplRegistry[job.Name]

	if !ok {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json:   types.ApiError{Message: "Task definition not found"},
		}
	}

	if job.OwnerRaw != nil {
		job.Owner = jobs.ParseOwner(*job.OwnerRaw)

		if job.Owner == nil {
			return uapi.HttpResponse{
				Status: http.StatusInternalServerError,
				Json:   types.ApiError{Message: "Invalid job.Owner. Parsing error occurred"},
			}
		}

		if job.Owner.ID == "" || job.Owner.TargetType == "" {
			return uapi.HttpResponse{
				Status: http.StatusInternalServerError,
				Json:   types.ApiError{Message: "Invalid job.Owner. Missing ID or TargetType"},
			}
		}

		if job.Owner.TargetType == splashcore.TargetTypeUser {
			if iot.DiscordUser.ID != job.Owner.ID {
				return uapi.HttpResponse{
					Status: http.StatusForbidden,
					Json:   types.ApiError{Message: "You are not authorized to fetch this task [TargetType = User]!"},
				}
			}
		} else if job.Owner.TargetType == splashcore.TargetTypeServer {
			// Check permissions
			resp, ok := api.HandlePermissionCheck(iot.DiscordUser.ID, job.Owner.ID, taskDef.CorrespondingBotCommand_Download(), rpc_messages.RpcCheckCommandOptions{})

			if !ok {
				return resp
			}

		} else {
			return uapi.HttpResponse{
				Status: http.StatusNotImplemented,
				Json:   types.ApiError{Message: "Downloading is not supported for this target type [TargetType = " + job.Owner.TargetType + "]"},
			}
		}
	}

	// Now get URL
	url, err := state.ObjectStorage.GetUrl(d.Context, jobs.GetPathFromOutput(job.ID, taskDef, job.Output), job.Output.Filename, 10*time.Minute)

	if err != nil {
		state.Logger.Error("Failed to get url for job", zap.Error(err))
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json:   types.ApiError{Message: "Failed to get url for job: " + err.Error()},
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

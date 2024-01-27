package create_task

import (
	"bytes"
	"io"
	"net/http"
	"time"

	"github.com/anti-raid/splashtail/tasks"
	"github.com/anti-raid/splashtail/types"
	"github.com/anti-raid/splashtail/webserver/state"
	jsoniter "github.com/json-iterator/go"

	"github.com/go-chi/chi/v5"
	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/ratelimit"
	"github.com/infinitybotlist/eureka/uapi"
	"go.uber.org/zap"
)

const clientName = "api"

var (
	clientSecret string
	json         = jsoniter.ConfigFastest
)

func Setup() {
	secrets := state.Config.Meta.JobserverSecrets.Parse()

	var ok bool
	clientSecret, ok = secrets[clientName]

	if !ok {
		panic("missing jobserver secret for client " + clientName)
	}
}

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Create Task",
		Description: "Creates a task. Returns the task data if this is successful",
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

	_, ok := tasks.TaskDefinitionRegistry[taskName]

	if !ok {
		return uapi.HttpResponse{
			Json: types.ApiError{
				Message: "Unknown task name",
			},
			Status: http.StatusBadRequest,
		}
	}

	body, err := io.ReadAll(r.Body)

	if err != nil {
		state.Logger.Error("Error reading body", zap.Error(err))
		return uapi.DefaultResponse(http.StatusInternalServerError)
	}

	var cmd = map[string]any{
		"args": map[string]any{
			"name":    taskName,
			"data":    string(body),
			"execute": true,
		},
	}

	cmdBytes, err := json.Marshal(cmd)

	if err != nil {
		state.Logger.Error("Error marshalling IPC create_task request", zap.Error(err))
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiErrorWith[types.TaskCreateResponse]{
				Message: "Error marshalling IPC create_task request: " + err.Error(),
			},
		}
	}

	// Use execute_task IPC in jobserver
	client := http.Client{
		Timeout: 5 * time.Second,
	}

	// FIXME: Use a better way of defining the job server url
	req, err := http.NewRequest("POST", state.Config.Meta.JobserverUrl.Parse()+"/ipc/create_task", bytes.NewBuffer(cmdBytes))

	if err != nil {
		state.Logger.Error("Error sending IPC create_task", zap.Error(err))
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiErrorWith[types.TaskCreateResponse]{
				Message: "Error sending IPC create_task: " + err.Error(),
			},
		}
	}

	req.Header.Set("Authorization", clientName+" "+clientSecret)

	resp, err := client.Do(req)

	if err != nil {
		state.Logger.Error("Error publishing IPC command", zap.Error(err))
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiErrorWith[types.TaskCreateResponse]{
				Message: "Error publishing IPC command: " + err.Error(),
			},
		}
	}

	if resp.StatusCode != http.StatusOK {
		resp, err := io.ReadAll(resp.Body)

		if err != nil {
			state.Logger.Error("Error publishing IPC command [recv error]", zap.Error(err))
			return uapi.HttpResponse{
				Status: http.StatusInternalServerError,
				Json: types.ApiErrorWith[types.TaskCreateResponse]{
					Message: "Error publishing IPC command [recv error]: " + err.Error(),
				},
			}
		}

		state.Logger.Error("Error publishing IPC command [got reply]:", zap.String("body", string(resp)))
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiErrorWith[types.TaskCreateResponse]{
				Message: "Error publishing IPC command [got reply]: " + string(resp),
			},
		}
	}

	var jsResp struct {
		Tcr types.TaskCreateResponse `json:"tcr"`
	}

	err = json.Unmarshal(body, &jsResp)

	if err != nil {
		state.Logger.Error("Error unmarshalling IPC create_task response", zap.Error(err))
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Error unmarshalling IPC create_task response: " + err.Error(),
			},
		}
	}

	return uapi.HttpResponse{
		Json: jsResp.Tcr,
	}
}

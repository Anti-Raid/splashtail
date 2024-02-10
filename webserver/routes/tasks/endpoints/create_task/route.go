package create_task

import (
	"io"
	"net/http"
	"time"

	"github.com/anti-raid/splashtail/splashcore/types"
	"github.com/anti-raid/splashtail/tasks"
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

	_, err = io.ReadAll(r.Body)

	if err != nil {
		state.Logger.Error("Error reading body", zap.Error(err))
		return uapi.DefaultResponse(http.StatusInternalServerError)
	}

	return uapi.DefaultResponse(http.StatusNotImplemented)
}

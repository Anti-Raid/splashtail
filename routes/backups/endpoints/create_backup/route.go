package create_backup

import (
	"net/http"
	"splashtail/state"
	"splashtail/tasks"
	"splashtail/tasks/backups"
	"splashtail/types"
	"time"

	"github.com/infinitybotlist/eureka/uapi/ratelimit"
	"go.uber.org/zap"

	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/uapi"
)

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Create Backup",
		Description: "Creates a backup for a server. Returns the task id if this is successful.",
		Params: []docs.Parameter{
			{
				Name:        "id",
				Description: "Server ID",
				Required:    true,
				In:          "path",
				Schema:      docs.IdSchema,
			},
		},
		Resp: types.TaskCreateResponse{},
	}
}

func Route(d uapi.RouteData, r *http.Request) uapi.HttpResponse {
	limit, err := ratelimit.Ratelimit{
		Expiry:      1 * time.Hour,
		MaxRequests: 50,
		Bucket:      "backup",
		Identifier: func(r *http.Request) string {
			return d.Auth.ID
		},
	}.Limit(d.Context, r)

	if err != nil {
		state.Logger.Error("Error while ratelimiting", zap.Error(err), zap.String("bucket", "backup"))
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

	task, tcr, err := tasks.CreateTask(d.Context, &backups.ServerBackupCreateTask{
		ServerID: d.Auth.ID,
	}, false)

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Error creating task:" + err.Error(),
			},
		}
	}

	go tasks.NewTask(task)

	return uapi.HttpResponse{
		Json: tcr,
	}
}

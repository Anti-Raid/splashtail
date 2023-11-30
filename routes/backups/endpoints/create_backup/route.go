package create_backup

import (
	"net/http"
	"splashtail/routes/backups/assets"
	"splashtail/state"
	"splashtail/types"
	"time"

	"github.com/infinitybotlist/eureka/crypto"
	"github.com/infinitybotlist/eureka/uapi/ratelimit"
	"github.com/jackc/pgx/v5/pgtype"
	"go.uber.org/zap"

	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/uapi"
)

const taskExpiryTime = time.Hour * 1
const taskName = "create_backup"

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

	taskKey := crypto.RandString(128)
	var taskId string

	err = state.Pool.QueryRow(d.Context, "INSERT INTO tasks (task_name, task_key, for_user, expiry, output, allow_unauthenticated) VALUES ($1, $2, $3, $4, $5, $6) RETURNING task_id",
		taskName,
		taskKey,
		"g/"+d.Auth.ID,
		taskExpiryTime,
		map[string]any{
			"meta": map[string]any{},
		},
		false,
	).Scan(&taskId)

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Error creating task:" + err.Error(),
			},
		}
	}

	go assets.CreateServerBackup(taskId, taskName, d.Auth.ID)

	return uapi.HttpResponse{
		Json: types.TaskCreateResponse{
			TaskID: taskId,
			TaskKey: pgtype.Text{
				Valid:  true,
				String: taskKey,
			},
			TaskName:             taskName,
			Expiry:               pgtype.Interval{Microseconds: int64(taskExpiryTime / time.Microsecond)},
			AllowUnauthenticated: false,
		},
	}
}

package create_guild_job

import (
	"io"
	"net/http"
	"time"

	"go.api/api"
	"go.api/rpc"
	"go.api/rpc_messages"
	"go.api/state"
	"go.api/types"
	jobs "go.jobs"
	jobtypes "go.jobs/types"

	"github.com/go-chi/chi/v5"
	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/jsonimpl"
	"github.com/infinitybotlist/eureka/ratelimit"
	"github.com/infinitybotlist/eureka/uapi"
	"go.uber.org/zap"
)

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Create Guild Job",
		Description: "Creates a job on a guild. Returns the job ID on success",
		Params: []docs.Parameter{
			{
				Name:        "guild_id",
				Description: "Guild ID",
				Required:    true,
				In:          "path",
				Schema:      docs.IdSchema,
			},
			{
				Name:        "name",
				Description: "The name of the job",
				Required:    true,
				In:          "path",
				Schema:      docs.IdSchema,
			},
		},
		Req:  "The fields to pass to the job create",
		Resp: jobtypes.JobCreateResponse{},
	}
}

func Route(d uapi.RouteData, r *http.Request) uapi.HttpResponse {
	limit, err := ratelimit.Ratelimit{
		Expiry:      1 * time.Hour,
		MaxRequests: 50,
		Bucket:      "create_job",
		Identifier: func(r *http.Request) string {
			return d.Auth.ID
		},
	}.Limit(d.Context, r)

	if err != nil {
		state.Logger.Error("Error while ratelimiting", zap.Error(err), zap.String("bucket", "create_job"))
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

	name := chi.URLParam(r, "name")

	if name == "" {
		return uapi.HttpResponse{
			Json: types.ApiError{
				Message: "Missing name",
			},
			Status: http.StatusBadRequest,
		}
	}

	guildId := chi.URLParam(r, "guild_id")

	if guildId == "" {
		return uapi.HttpResponse{
			Json: types.ApiError{
				Message: "Missing guild_id",
			},
			Status: http.StatusBadRequest,
		}
	}

	baseJobImpl, ok := jobs.JobImplRegistry[name]

	if !ok {
		return uapi.HttpResponse{
			Json: types.ApiError{
				Message: "Unknown job name",
			},
			Status: http.StatusBadRequest,
		}
	}

	b, err := io.ReadAll(r.Body)

	if err != nil {
		state.Logger.Error("Error reading body", zap.Error(err))
		return uapi.DefaultResponse(http.StatusInternalServerError)
	}

	job := baseJobImpl // Copy job

	err = jsonimpl.Unmarshal(b, &job)

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json: types.ApiError{
				Message: "Invalid job data: " + err.Error(),
			},
		}
	}

	// Check permissions
	permLimits := api.PermLimits(d.Auth)
	resp, ok := api.HandlePermissionCheck(d.Auth.ID, guildId, job.CorrespondingBotCommand_Create(), rpc_messages.RpcCheckCommandOptions{
		CustomResolvedKittycatPerms: permLimits,
	})

	if !ok {
		return resp
	}

	var data map[string]any

	err = jsonimpl.Unmarshal(b, &data)

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json: types.ApiError{
				Message: "Invalid job data: " + err.Error(),
			},
		}
	}

	str, err := rpc.JobserverSpawnTask(d.Context, &rpc_messages.JobserverSpawn{
		Name:    name,
		Data:    data,
		Create:  true,
		Execute: true,
		UserID:  d.Auth.ID,
	})

	if err != nil {
		state.Logger.Error("Error while spawning job on jobserver", zap.Error(err))
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Error while spawning job on jobserver: " + err.Error(),
			},
		}
	}

	return uapi.HttpResponse{
		Json: jobtypes.JobCreateResponse{
			ID: str.ID,
		},
	}
}

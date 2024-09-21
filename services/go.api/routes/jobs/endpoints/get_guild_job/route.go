package get_guild_job

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
	jobsColsArr = db.GetCols(jobtypes.Job{})
	jobsColsStr = strings.Join(jobsColsArr, ", ")
)

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Get Guild Job",
		Description: "Gets a job owned by a guild",
		Params: []docs.Parameter{
			{
				Name:        "guild_id",
				Description: "Guild ID",
				Required:    true,
				In:          "path",
				Schema:      docs.IdSchema,
			},
			{
				Name:        "id",
				Description: "The job ID",
				Required:    true,
				In:          "path",
				Schema:      docs.IdSchema,
			},
		},
		Resp: jobtypes.Job{},
	}
}

func Route(d uapi.RouteData, r *http.Request) uapi.HttpResponse {
	id := chi.URLParam(r, "id")
	guildId := chi.URLParam(r, "guild_id")

	if id == "" || guildId == "" {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json:   types.ApiError{Message: "id/guild_id is required"},
		}
	}

	// Delete expired jobs first
	_, err := state.Pool.Exec(d.Context, "DELETE FROM jobs WHERE created_at + expiry < NOW()")

	if err != nil {
		state.Logger.Error("Failed to delete expired jobs [db delete]", zap.Error(err))
		return uapi.DefaultResponse(http.StatusInternalServerError)
	}

	var row pgx.Rows
	row, err = state.Pool.Query(d.Context, "SELECT "+jobsColsStr+" FROM jobs WHERE id = $1 AND guild_id = $2", id, guildId)

	if err != nil {
		state.Logger.Error("Failed to fetch job [db fetch]", zap.Error(err))
		return uapi.DefaultResponse(http.StatusInternalServerError)
	}

	job, err := pgx.CollectOneRow(row, pgx.RowToStructByName[jobtypes.Job])

	if errors.Is(err, pgx.ErrNoRows) {
		return uapi.HttpResponse{
			Status: http.StatusNotFound,
			Json:   types.ApiError{Message: "Job not found"},
		}
	}

	if err != nil {
		state.Logger.Error("Failed to fetch job [db fetch]", zap.Error(err))
		return uapi.DefaultResponse(http.StatusInternalServerError)
	}

	baseJobImpl, ok := jobs.JobImplRegistry[job.Name]

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
		Json:   job,
	}
}

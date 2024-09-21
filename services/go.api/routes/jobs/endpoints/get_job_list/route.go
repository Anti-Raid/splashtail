package get_job_list

import (
	"errors"
	"net/http"
	"strings"

	"github.com/go-chi/chi/v5"
	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/uapi"
	"github.com/jackc/pgx/v5"
	"go.api/api"
	"go.api/rpc_messages"
	"go.api/state"
	"go.api/types"
	jobs "go.jobs"
	jobtypes "go.jobs/types"
	"go.std/structparser/db"
	"go.uber.org/zap"
)

var (
	jobColsArr = db.GetCols(jobtypes.PartialJob{})
	jobColsStr = strings.Join(jobColsArr, ", ")
)

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Get Job List",
		Description: "Gets the list of all jobs as PartialJob objects",
		Params: []docs.Parameter{
			{
				Name:        "guild_id",
				Description: "Guild ID",
				Required:    true,
				In:          "path",
				Schema:      docs.IdSchema,
			},
			{
				Name:        "error_if_no_permissions",
				Description: "Whether or not to return an error if the user does not have permission to view a job. Otherwise, the job will simply not be returned",
				Required:    true,
				In:          "query",
				Schema:      docs.IdSchema,
			},
			{
				Name:        "error_on_unknown_job",
				Description: "Whether or not to return an error if a job is unknown. Otherwise, the job will simply not be returned",
				Required:    true,
				In:          "query",
				Schema:      docs.IdSchema,
			},
		},
		Resp: jobtypes.JobListResponse{},
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
	errorOnUnknownJobs := r.URL.Query().Get("error_on_unknown_jobs") == "true"

	// Delete expired jobs first
	_, err := state.Pool.Exec(d.Context, "DELETE FROM jobs WHERE created_at + expiry < NOW()")

	if err != nil {
		state.Logger.Error("Failed to delete expired jobs [db delete]", zap.Error(err))
		return uapi.DefaultResponse(http.StatusInternalServerError)
	}

	row, err := state.Pool.Query(d.Context, "SELECT "+jobColsStr+" FROM jobs WHERE guild_id = $1", guildId)

	if err != nil {
		state.Logger.Error("Failed to fetch job [db fetch]", zap.Error(err))
		return uapi.DefaultResponse(http.StatusInternalServerError)
	}

	jobsFetched, err := pgx.CollectRows(row, pgx.RowToStructByName[jobtypes.PartialJob])

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

	var checksDone = map[string]bool{}
	var parsedJobs = []jobtypes.PartialJob{}
	for _, job := range jobsFetched {
		// NOTE/WARNING: This is a fastpath that depends on the assumption that the corresponding bot command
		// does not change for a job. If this assumption is broken, this code will break if the corresponding
		// command changes while in this loop
		if _, ok := checksDone[job.Name]; ok {
			parsedJobs = append(parsedJobs, job)
		}

		baseJobImpl, ok := jobs.JobImplRegistry[job.Name]

		if !ok {
			if errorOnUnknownJobs {
				return uapi.HttpResponse{
					Json: types.ApiError{
						Message: "Internal Error: Unknown job name",
					},
					Status: http.StatusInternalServerError,
				}
			} else {
				continue
			}
		}

		// Check permissions
		permLimits := api.PermLimits(d.Auth)
		resp, ok := api.HandlePermissionCheck(d.Auth.ID, guildId, baseJobImpl.CorrespondingBotCommand_View(), rpc_messages.RpcCheckCommandOptions{
			CustomResolvedKittycatPerms: permLimits,
		})

		if !ok {
			if errorIfNoPermissions {
				return resp
			}
			continue
		}

		checksDone[job.Name] = true
		parsedJobs = append(parsedJobs, job)
	}

	return uapi.HttpResponse{
		Status: http.StatusOK,
		Json:   jobtypes.JobListResponse{Jobs: parsedJobs},
	}
}

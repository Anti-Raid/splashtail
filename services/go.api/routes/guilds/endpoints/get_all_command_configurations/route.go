package get_all_command_configurations

import (
	"errors"
	"net/http"
	"strings"
	"time"

	"github.com/go-chi/chi/v5"
	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/ratelimit"
	"github.com/infinitybotlist/eureka/uapi"
	"github.com/jackc/pgx/v5"
	"go.api/state"
	"go.api/types"
	"go.std/silverpelt"
	"go.std/structparser/db"
	"go.uber.org/zap"
)

var (
	fullCommandConfigurationColsArr = db.GetCols(silverpelt.FullGuildCommandConfiguration{})
	fullCommandConfigurationCols    = strings.Join(fullCommandConfigurationColsArr, ", ")
)

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Get All Command Configurations",
		Description: "This endpoint returns all configurations for all commands in a guild",
		Resp:        []silverpelt.FullGuildCommandConfiguration{},
		Params: []docs.Parameter{
			{
				Name:        "guild_id",
				Description: "Whether to refresh the user's guilds from discord",
				In:          "path",
				Required:    true,
				Schema:      docs.IdSchema,
			},
		},
	}
}

func Route(d uapi.RouteData, r *http.Request) uapi.HttpResponse {
	limit, err := ratelimit.Ratelimit{
		Expiry:      1 * time.Minute,
		MaxRequests: 20,
		Bucket:      "command_configuration",
	}.Limit(d.Context, r)

	if err != nil {
		state.Logger.Error("Error while ratelimiting", zap.Error(err), zap.String("bucket", "command_configuration"))
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

	guildId := chi.URLParam(r, "guild_id")

	if guildId == "" {
		return uapi.DefaultResponse(http.StatusBadRequest)
	}

	rows, err := state.Pool.Query(
		d.Context,
		"SELECT "+fullCommandConfigurationCols+" FROM guild_command_configurations WHERE guild_id = $1",
		guildId,
	)

	if errors.Is(err, pgx.ErrNoRows) {
		return uapi.HttpResponse{
			Json: []silverpelt.FullGuildCommandConfiguration{},
		}
	}

	if err != nil {
		state.Logger.Error("Failed to query guild_command_configuration", zap.Error(err))
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Failed to query guild_command_configuration: " + err.Error(),
			},
		}
	}

	recs, err := pgx.CollectRows(rows, pgx.RowToStructByName[silverpelt.FullGuildCommandConfiguration])

	if errors.Is(err, pgx.ErrNoRows) {
		return uapi.HttpResponse{
			Json: []silverpelt.FullGuildCommandConfiguration{},
		}
	}

	if err != nil {
		state.Logger.Error("Failed to collect rows", zap.Error(err))
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Failed to collect rows: " + err.Error(),
			},
		}
	}

	return uapi.HttpResponse{
		Json: recs,
	}
}

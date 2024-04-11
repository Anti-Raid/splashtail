package get_module_configurations

import (
	"errors"
	"net/http"
	"strings"
	"time"

	"github.com/anti-raid/splashtail/splashcore/silverpelt"
	"github.com/anti-raid/splashtail/splashcore/structparser/db"
	"github.com/anti-raid/splashtail/splashcore/types"
	"github.com/anti-raid/splashtail/webserver/state"
	"github.com/go-chi/chi/v5"
	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/ratelimit"
	"github.com/infinitybotlist/eureka/uapi"
	"github.com/jackc/pgx/v5"
	"go.uber.org/zap"
)

var (
	gmcCols    = db.GetCols(silverpelt.GuildModuleConfiguration{})
	gmcColsStr = strings.Join(gmcCols, ", ")
)

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Get Module Configurations",
		Description: "This endpoint returns the configuration for all modules in a guild",
		Resp:        []*silverpelt.GuildModuleConfiguration{},
		Params: []docs.Parameter{
			{
				Name:        "user_id",
				Description: "The ID of the user to get information about",
				In:          "path",
				Required:    true,
				Schema:      docs.IdSchema,
			},
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
		Expiry:      2 * time.Minute,
		MaxRequests: 10,
		Bucket:      "module_configuration",
	}.Limit(d.Context, r)

	if err != nil {
		state.Logger.Error("Error while ratelimiting", zap.Error(err), zap.String("bucket", "module_configuration"))
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
	userId := chi.URLParam(r, "user_id")

	if guildId == "" || userId == "" {
		return uapi.DefaultResponse(http.StatusBadRequest)
	}

	// Fetch row from guild_module_configuration
	row, err := state.Pool.Query(d.Context, "SELECT "+gmcColsStr+" FROM guild_module_configurations WHERE guild_id = $1", guildId)

	if err != nil {
		state.Logger.Error("Failed to query guild_module_configuration", zap.Error(err))
		return uapi.DefaultResponse(http.StatusInternalServerError)
	}

	defer row.Close()

	gmc, err := pgx.CollectRows(row, pgx.RowToAddrOfStructByName[silverpelt.GuildModuleConfiguration])

	if err != nil && !errors.Is(err, pgx.ErrNoRows) {
		state.Logger.Error("Failed to collect guild_module_configuration", zap.Error(err))
		return uapi.DefaultResponse(http.StatusInternalServerError)
	}

	if len(gmc) == 0 {
		gmc = []*silverpelt.GuildModuleConfiguration{}
	}

	return uapi.HttpResponse{
		Json: gmc,
	}
}

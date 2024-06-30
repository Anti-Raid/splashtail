package get_command_configurations

import (
	"net/http"
	"time"

	"github.com/anti-raid/splashtail/core/go.std/silverpelt"
	"github.com/anti-raid/splashtail/core/go.std/types"
	"github.com/anti-raid/splashtail/services/go.api/state"
	"github.com/go-chi/chi/v5"
	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/ratelimit"
	"github.com/infinitybotlist/eureka/uapi"
	"go.uber.org/zap"
)

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Get Command Configurations",
		Description: "This endpoint returns all configurations for a specific command in a guild",
		Resp:        []silverpelt.GuildCommandConfiguration{},
		Params: []docs.Parameter{
			{
				Name:        "guild_id",
				Description: "Whether to refresh the user's guilds from discord",
				In:          "path",
				Required:    true,
				Schema:      docs.IdSchema,
			},
			{
				Name:        "command",
				Description: "The command to get the configuration of",
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
	command := chi.URLParam(r, "command")

	if guildId == "" || command == "" {
		return uapi.DefaultResponse(http.StatusBadRequest)
	}

	// Fetch row from guild_module_configuration
	gcc, err := silverpelt.GetAllCommandConfigurations(d.Context, state.Pool, guildId, command)

	if err != nil {
		state.Logger.Error("Failed to query guild_command_configuration", zap.Error(err))
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Failed to query guild_command_configuration: " + err.Error(),
			},
		}
	}

	return uapi.HttpResponse{
		Json: gcc,
	}
}

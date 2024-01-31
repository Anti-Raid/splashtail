package get_user_guilds

import (
	"encoding/json"
	"io"
	"net/http"
	"time"

	"github.com/anti-raid/splashtail/animusmagic"
	"github.com/anti-raid/splashtail/types"
	"github.com/anti-raid/splashtail/utils"
	"github.com/anti-raid/splashtail/utils/mewext"
	"github.com/anti-raid/splashtail/webserver/state"
	"github.com/bwmarrin/discordgo"
	"go.uber.org/zap"

	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/ratelimit"
	"github.com/infinitybotlist/eureka/uapi"
)

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Get User",
		Description: "This endpoint will return user information given their ID",
		Resp:        types.DashboardGuildData{},
		Params: []docs.Parameter{
			{
				Name:        "id",
				Description: "The ID of the user to get information about",
				In:          "path",
				Required:    true,
				Schema:      docs.IdSchema,
			},
			{
				Name:        "refresh",
				Description: "Whether to refresh the user's guilds from discord",
				In:          "query",
				Required:    false,
				Schema:      docs.BoolSchema,
			},
		},
	}
}

func Route(d uapi.RouteData, r *http.Request) uapi.HttpResponse {
	var limit *ratelimit.Limit
	if r.URL.Query().Get("refresh") == "true" {
		limit, err := ratelimit.Ratelimit{
			Expiry:      5 * time.Minute,
			MaxRequests: 3,
			Bucket:      "get_user_guilds_refresh",
		}.Limit(d.Context, r)

		if err != nil {
			state.Logger.Error("Error while ratelimiting", zap.Error(err), zap.String("bucket", "login"))
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
	} else {
		limit, err := ratelimit.Ratelimit{
			Expiry:      5 * time.Minute,
			MaxRequests: 5,
			Bucket:      "get_user_guilds_norefresh",
		}.Limit(d.Context, r)

		if err != nil {
			state.Logger.Error("Error while ratelimiting", zap.Error(err), zap.String("bucket", "login"))
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
	}

	refresh := r.URL.Query().Get("refresh") == "true"

	if !refresh {
		// Once case where we must refresh is if guilds_cache is NULL
		var count int64

		err := state.Pool.QueryRow(d.Context, "SELECT COUNT(*) FROM users WHERE user_id = $1 AND guilds_cache IS NULL", d.Auth.ID).Scan(&count)

		if err != nil {
			state.Logger.Error("Failed to query database", zap.Error(err))
			return uapi.DefaultResponse(http.StatusInternalServerError)
		}

		if count != 0 {
			refresh = true
		}
	}

	// Fetch guild list of user
	var dashguilds []*types.DashboardGuild
	if refresh {
		// Refresh guilds
		httpReq, err := http.NewRequestWithContext(d.Context, "GET", "https://discord.com/api/v10/users/@me/guilds", nil)

		if err != nil {
			state.Logger.Error("Failed to create oauth2 request to discord", zap.Error(err))
			return uapi.HttpResponse{
				Json: types.ApiError{
					Message: "Failed to create request to Discord to fetch guilds",
				},
				Status:  http.StatusInternalServerError,
				Headers: limit.Headers(),
			}
		}

		var accesstoken string

		err = state.Pool.QueryRow(d.Context, "SELECT access_token FROM users WHERE user_id = $1", d.Auth.ID).Scan(&accesstoken)

		if err != nil {
			state.Logger.Error("Failed to query database", zap.Error(err))
			return uapi.DefaultResponse(http.StatusInternalServerError)
		}

		httpReq.Header.Set("Authorization", "Bearer "+accesstoken)

		cli := &http.Client{}

		httpResp, err := cli.Do(httpReq)

		if err != nil {
			state.Logger.Error("Failed to send oauth2 request to discord", zap.Error(err))
			return uapi.HttpResponse{
				Json: types.ApiError{
					Message: "Failed to send oauth2 request to Discord [user guilds]",
				},
				Status:  http.StatusInternalServerError,
				Headers: limit.Headers(),
			}
		}

		defer httpResp.Body.Close()

		body, err := io.ReadAll(httpResp.Body)

		if err != nil {
			state.Logger.Error("Failed to read oauth2 response from discord", zap.Error(err))
			return uapi.HttpResponse{
				Json: types.ApiError{
					Message: "Failed to read oauth2 response from Discord [user guilds]",
				},
				Status:  http.StatusInternalServerError,
				Headers: limit.Headers(),
			}
		}

		var guilds []*discordgo.UserGuild

		err = json.Unmarshal(body, &guilds)

		if err != nil {
			state.Logger.Error("Failed to parse oauth2 response from discord", zap.Error(err))
			return uapi.HttpResponse{
				Json: types.ApiError{
					Message: "Failed to parse oauth2 response from Discord [user guilds]",
				},
				Status:  http.StatusInternalServerError,
				Headers: limit.Headers(),
			}
		}

		for _, guild := range guilds {
			dashguilds = append(dashguilds, &types.DashboardGuild{
				ID:   guild.ID,
				Name: guild.Name,
				Avatar: func() string {
					return utils.IconURL(guild.Icon, discordgo.EndpointGuildIcon(guild.ID, guild.Icon), discordgo.EndpointGuildIconAnimated(guild.ID, guild.Icon), "64")
				}(),
			})
		}
	} else {
		err := state.Pool.QueryRow(d.Context, "SELECT guilds_cache FROM users WHERE user_id = $1", d.Auth.ID).Scan(&dashguilds)

		if err != nil {
			state.Logger.Error("Failed to query database", zap.Error(err))
			return uapi.DefaultResponse(http.StatusInternalServerError)
		}
	}

	// Now use animus magic to determine which servers have the bot in them
	var clusterGuildReqs = map[int][]string{}

	for _, guild := range dashguilds {
		clusterId, err := mewext.GetClusterIDFromGuildID(guild.ID, state.MewldInstanceList.Map, int(state.MewldInstanceList.ShardCount))

		if err != nil {
			continue
		}

		if v, ok := clusterGuildReqs[clusterId]; ok {
			clusterGuildReqs[clusterId] = append(v, guild.ID)
		} else {
			clusterGuildReqs[clusterId] = []string{guild.ID}
		}
	}

	// Now send the requests
	var botInGuild []string
	for clusterId, guilds := range clusterGuildReqs {
		moduleListResp, err := state.AnimusMagicClient.Request(d.Context, state.Rueidis, &animusmagic.RequestData{
			ClusterID: utils.Pointer(uint16(clusterId)),
			Message: &animusmagic.AnimusMessage{
				GuildsExist: &struct {
					GuildsExist []string `json:"guilds_exist"`
				}{
					GuildsExist: guilds,
				},
			},
		})

		if err != nil {
			state.Logger.Error("Failed to send request to animus magic", zap.Error(err))
			return uapi.DefaultResponse(http.StatusInternalServerError)
		}

		if len(moduleListResp) != 1 {
			continue
		}

		for _, resp := range moduleListResp {
			for i, v := range resp.Resp.GuildsExist.GuildsExist {
				if v == 1 {
					botInGuild = append(botInGuild, guilds[i])
				}
			}
		}
	}

	return uapi.HttpResponse{
		Json: &types.DashboardGuildData{
			Guilds:      dashguilds,
			BotInGuilds: botInGuild,
		},
		Headers: limit.Headers(),
	}
}

package get_staff_team

import (
	"net/http"
	"time"

	"go.api/state"
	"go.api/types"
	"go.std/utils/syncmap"
	"go.uber.org/zap"

	"github.com/bwmarrin/discordgo"
	"github.com/go-chi/chi/v5"
	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/dovewing"
	"github.com/infinitybotlist/eureka/dovewing/dovetypes"
	"github.com/infinitybotlist/eureka/ratelimit"
	"github.com/infinitybotlist/eureka/uapi"
)

var StaffTeamCache = &StaffTeamCacher{}

type StaffTeamCacher struct {
	cache syncmap.Map[string, types.GuildStaffTeam]
}

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Get Staff Team",
		Description: "This endpoint will return a list of User objects representing the staff team of the server along with their position",
		Resp:        types.GuildStaffTeam{},
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
		Expiry:      3 * time.Minute,
		MaxRequests: 5,
		Bucket:      "get_staff_team",
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

	if guildId == "" {
		return uapi.DefaultResponse(http.StatusBadRequest)
	}

	// TODO: Allow this API to be used for any guild
	if guildId != state.Config.Servers.Main {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json: types.ApiError{
				Message: "This endpoint can currently only be used on the main server",
			},
		}
	}

	// Check if the staff team is cached
	if cached, ok := StaffTeamCache.cache.Load(guildId); ok {
		return uapi.HttpResponse{
			Json: cached,
		}
	}

	// Fetch the guild from sandwich
	if state.Config.Meta.SandwichHttpApi == "" {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json: types.ApiError{
				Message: "Sandwich API is not configured",
			},
		}
	}

	// Query Discord for the guild members
	after := ""
	guildMembers := []*discordgo.Member{}

	for {
		resp, err := state.Discord.GuildMembers(guildId, after, 1000)

		if err != nil {
			state.Logger.Error("Error while querying discord", zap.Error(err))
			return uapi.DefaultResponse(http.StatusInternalServerError)
		}

		guildMembers = append(guildMembers, resp...)

		if len(resp) < 1000 {
			break
		}

		after = resp[len(resp)-1].User.ID
	}

	// Get the staff team from the database
	rows, err := state.Pool.Query(d.Context, "SELECT user_id, roles, public FROM guild_members WHERE guild_id = $1 AND public = true", guildId)

	if err != nil {
		state.Logger.Error("Error while querying database", zap.Error(err))
		return uapi.DefaultResponse(http.StatusInternalServerError)
	}

	defer rows.Close()

	var collected []struct {
		UserId string
		Roles  []string
		Public bool
	}
	for rows.Next() {
		var userId string
		var roles []string
		var public bool

		err = rows.Scan(&userId, &roles, &public)

		if err != nil {
			state.Logger.Error("Error while scanning rows", zap.Error(err))
			return uapi.DefaultResponse(http.StatusInternalServerError)
		}

		collected = append(collected, struct {
			UserId string
			Roles  []string
			Public bool
		}{
			UserId: userId,
			Roles:  roles,
			Public: public,
		})
	}

	// Fetch guild staff roles from the database
	rows, err = state.Pool.Query(d.Context, "SELECT role_id, perms, index, display_name FROM guild_roles WHERE guild_id = $1 AND cardinality(perms) > 0", guildId)

	if err != nil {
		state.Logger.Error("Error while querying database", zap.Error(err))
		return uapi.DefaultResponse(http.StatusInternalServerError)
	}

	defer rows.Close()

	var roles = []types.GuildStaffRoles{}

	for rows.Next() {
		var roleId string
		var perms []string
		var index int
		var displayName *string

		err = rows.Scan(&roleId, &perms, &index, &displayName)

		if err != nil {
			state.Logger.Error("Error while scanning rows", zap.Error(err))
			return uapi.DefaultResponse(http.StatusInternalServerError)
		}

		roles = append(roles, types.GuildStaffRoles{
			RoleID:      roleId,
			Perms:       perms,
			Index:       index,
			DisplayName: displayName,
		})
	}

	var addedMembers = make(map[string]struct{})

	// First add cached guild members
	var members = []types.GuildStaffMember{}
	for _, member := range guildMembers {
		if member.User.Bot {
			continue
		}

		if len(member.Roles) == 0 {
			continue
		}

		properRoles := []string{}

		for _, role := range member.Roles {
			for _, r := range roles {
				if r.RoleID == role {
					properRoles = append(properRoles, role)
				}
			}
		}

		if len(properRoles) == 0 {
			continue // Skip if the user has no roles
		}

		members = append(members, types.GuildStaffMember{
			User: &dovetypes.PlatformUser{
				ID:          member.User.ID,
				Username:    member.User.Username,
				DisplayName: member.DisplayName(),
				Avatar:      member.AvatarURL(""),
				Bot:         member.User.Bot,
				Status:      dovetypes.PlatformStatusOffline,
			},
			Role:   properRoles,
			Public: true, // TOOD: Implement this
		})
		addedMembers[member.User.ID] = struct{}{}
	}

	// Then add database cached members
	for _, member := range collected {
		_, ok := addedMembers[member.UserId]

		if ok {
			if !member.Public {
				// Delete the member from the list
				for i, m := range members {
					if m.User.ID == member.UserId {
						members = append(members[:i], members[i+1:]...)
					}
				}
			}
			continue // Skip
		}

		if len(member.Roles) == 0 {
			continue
		}

		properRoles := []string{}

		for _, role := range member.Roles {
			for _, r := range roles {
				if r.RoleID == role {
					properRoles = append(properRoles, role)
				}
			}
		}

		if len(properRoles) == 0 {
			continue // Skip if the user has no roles
		}

		user, err := dovewing.GetUser(d.Context, member.UserId, state.DovewingPlatformDiscord)

		if err != nil {
			state.Logger.Error("Error while getting user", zap.Error(err))
			return uapi.DefaultResponse(http.StatusInternalServerError)
		}

		members = append(members, types.GuildStaffMember{
			User:   user,
			Role:   properRoles,
			Public: member.Public,
		})
		addedMembers[member.UserId] = struct{}{}
	}

	resp := types.GuildStaffTeam{
		Members: members,
		Roles:   roles,
	}

	StaffTeamCache.cache.Store(guildId, resp)

	return uapi.HttpResponse{
		Json: resp,
	}
}

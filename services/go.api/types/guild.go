package types

import (
	"github.com/Anti-Raid/corelib_go/silverpelt"
	"github.com/infinitybotlist/eureka/dovewing/dovetypes"
)

type DashboardGuild struct {
	ID          string `json:"id" description:"The ID of the guild"`
	Name        string `json:"name" description:"The name of the guild"`
	Avatar      string `json:"avatar" description:"The avatar url of the guild"`
	Permissions int64  `json:"permissions" description:"The permissions the user has in the guild"`
}

type DashboardGuildData struct {
	Guilds        []*DashboardGuild `json:"guilds" description:"The guilds the user is in"`
	BotInGuilds   []string          `json:"has_bot" description:"A list of guild IDs that the user has the bot in"`
	UnknownGuilds []string          `json:"unknown_guilds" description:"A list of guild IDs where the bot is in an outage etc. in"`
}

type GuildStaffTeam struct {
	Members []GuildStaffMember `json:"members" description:"The staff team of the guild"`
	Roles   []GuildStaffRoles  `json:"roles" description:"The staff roles of the guild"`
}

// Note: not all fields in `guild_members` are included
type GuildStaffRoles struct {
	RoleID      string   `json:"role_id" description:"The ID of the role"`
	Perms       []string `json:"perms" description:"The permissions of the role"`
	Index       int      `json:"index" description:"The index of the role"`
	DisplayName *string  `json:"display_name" description:"The display name of the role"`
}

// Note: not all fields in `guild_members` are included
type GuildStaffMember struct {
	User   *dovetypes.PlatformUser `json:"user" description:"The user object of the staff member"`
	Role   []string                `json:"role" description:"The role of the staff member"`
	Public bool                    `json:"public" description:"Whether the staff member is public"`
}

type ExecuteTemplateRequest struct {
	Args     any    `json:"args"`
	Template string `json:"template"`
}

type ExecuteTemplateResponse struct {
	Ok *struct {
		Result any `json:"result"`
	} `json:"Ok,omitempty"`
	ExecErr *struct {
		Error string `json:"error"`
	} `json:"ExecErr,omitempty"`
	PermissionError *struct {
		Res silverpelt.PermissionResult `json:"res"`
	} `json:"PermissionError,omitempty"`
}

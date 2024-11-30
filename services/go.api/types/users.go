package types

import (
	"time"

	"github.com/Anti-Raid/corelib_go/ext_types"
	"github.com/infinitybotlist/eureka/dovewing/dovetypes"
)

// Represents a user on Antiraid
type User struct {
	User       *dovetypes.PlatformUser `json:"user" description:"The user object of the user"`
	State      string                  `db:"state" json:"state" description:"The state of the user"`
	VoteBanned bool                    `db:"vote_banned" json:"vote_banned" description:"Whether or not the user is banned from recieving rewards from voting"`
	CreatedAt  time.Time               `db:"created_at" json:"created_at" description:"The time the user was created"`
	UpdatedAt  time.Time               `db:"updated_at" json:"updated_at" description:"The time the user was last updated"`
}

/*
owner_id: String,
name: String,
icon: Option<String>,
/// List of all roles in the server
roles: std::collections::HashMap<RoleId, Role>,
/// List of roles the user has
user_roles: Vec<RoleId>,
/// List of roles the bot has
bot_roles: Vec<RoleId>,
/// List of all channels in the server
channels: Vec<GuildChannel>,
*/

type UserGuildBaseData struct {
	OwnerID                 string                                  `json:"owner_id" description:"The ID of the guild owner"`
	Name                    string                                  `json:"name" description:"The name of the guild"`
	Icon                    *string                                 `json:"icon" description:"The icon of the guild"`
	Roles                   []ext_types.SerenityRole                `json:"roles" description:"The roles of the guild"`
	UserRoles               []string                                `json:"user_roles" description:"The role IDs the user has in the guild"`
	BotRoles                []string                                `json:"bot_roles" description:"The role IDs the bot has in the guild"`
	Channels                []ext_types.GuildChannelWithPermissions `json:"channels" description:"The channels of the guild with permission info"`
	GuildFinishedOnboarding bool                                    `json:"finished_onboarding" description:"Whether the user has finished onboarding"`
}

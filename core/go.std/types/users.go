package types

import (
	"time"

	"github.com/anti-raid/splashtail/core/go.std/types/ext"
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
*/

type UserGuildBaseData struct {
	OwnerID   string                       `json:"owner_id"`
	Name      string                       `json:"name"`
	Icon      *string                      `json:"icon"`
	Roles     map[string]*ext.SerenityRole `json:"roles"`
	UserRoles []string                     `json:"user_roles"`
	BotRoles  []string                     `json:"bot_roles"`
}

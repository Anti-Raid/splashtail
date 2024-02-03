package types

import (
	"time"

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

type PartialRole struct {
	ID       string `json:"id"`
	Name     string `json:"name"`
	Position int16  `json:"position"`
}

type UserGuildBaseData struct {
	OwnerID    string        `json:"owner_id"`
	Name       string        `json:"name"`
	Icon       *string       `json:"icon"`
	Roles      []PartialRole `json:"roles"`
	BotHighest PartialRole   `json:"bot_highest"`
}

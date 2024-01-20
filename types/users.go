package types

import (
	"time"

	"github.com/infinitybotlist/eureka/dovewing/dovetypes"
)

// Represents a user on Antiraid
type User struct {
	User      *dovetypes.PlatformUser `json:"user" description:"The user object of the user"`
	State     string                  `db:"state" json:"state" description:"The state of the user"`
	CreatedAt time.Time               `db:"created_at" json:"created_at" description:"The time the user was created"`
	UpdatedAt time.Time               `db:"updated_at" json:"updated_at" description:"The time the user was last updated"`
}

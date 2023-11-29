package types

import "time"

// Represents a user on Antiraid
type User struct {
	UserID string `db:"user_id" json:"user_id" description:"The user's ID"`

	CreatedAt time.Time `db:"created_at" json:"created_at" description:"The time the user was created"`
	UpdatedAt time.Time `db:"updated_at" json:"updated_at" description:"The time the user was last updated"`
}

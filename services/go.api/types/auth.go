package types

import (
	"time"
)

type AuthorizeRequest struct {
	Code        string `json:"code" validate:"required,min=5"`
	RedirectURI string `json:"redirect_uri" validate:"required"`
	Protocol    string `json:"protocol" validate:"required" description:"Should be 'a1'. This is to identify and block older clients that don't support newer protocols"`
	Scope       string `json:"scope" validate:"required,oneof=normal ban_exempt"`
}

type UserSession struct {
	ID         string    `db:"id" json:"id" description:"The ID of the session"`
	Name       *string   `db:"name" json:"name,omitempty" description:"The name of the session. Login sessions do not have any names by default"`
	UserID     string    `db:"user_id" json:"user_id" description:"The users ID"`
	CreatedAt  time.Time `db:"created_at" json:"created_at" description:"The time the session was created"`
	Type       string    `db:"type" json:"type" description:"The type of session token"`
	PermLimits []string  `db:"perm_limits" json:"perm_limits" description:"The permissions the session has"`
	Expiry     time.Time `db:"expiry" json:"expiry" description:"The time the session expires"`
}

type CreateUserSession struct {
	Name       string   `json:"name" validate:"required" description:"The name of the session"`
	Type       string   `json:"type" validate:"oneof=api" description:"The type of session token. Must be 'api'"`
	PermLimits []string `json:"perm_limits" description:"The permissions the session will have"`
	Expiry     int64    `json:"expiry" validate:"required" description:"The time in seconds the session will last"`
}

type CreateUserSessionResponse struct {
	UserID    string    `json:"user_id" description:"The ID of the user who created the session"`
	Token     string    `json:"token" description:"The token of the session"`
	SessionID string    `json:"session_id" description:"The ID of the session"`
	Expiry    time.Time `json:"expiry" description:"The time the session expires"`
}

type UserSessionList struct {
	Sessions []*UserSession `json:"sessions" description:"The list of sessions"`
}

type TestAuth struct {
	AuthType string `json:"auth_type"`
	TargetID string `json:"target_id"`
	Token    string `json:"token"`
}

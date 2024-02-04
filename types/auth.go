package types

import "time"

type AuthorizeRequest struct {
	Code        string `json:"code" validate:"required,min=5"`
	RedirectURI string `json:"redirect_uri" validate:"required"`
	Protocol    string `json:"protocol" validate:"required" description:"Should be 'a1'. This is to identify and block older clients that don't support newer protocols"`
	Scope       string `json:"scope" validate:"required,oneof=normal ban_exempt"`
}

type UserLogin struct {
	Token  string `json:"token" description:"The users token"`
	UserID string `json:"user_id" description:"The users ID"`
}

type UserSession struct {
	ID        string    `db:"id" json:"id" description:"The ID of the session"`
	UserID    string    `db:"user_id" json:"user_id" description:"The users ID"`
	CreatedAt time.Time `db:"created_at" json:"created_at" description:"The time the session was created"`
	Type      string    `db:"type" json:"type" description:"The type of session token"`
	Expiry    time.Time `db:"expiry" json:"expiry" description:"The time the session expires"`
}

type UserSessionList struct {
	Sessions []*UserSession `json:"sessions" description:"The list of sessions"`
}

type TestAuth struct {
	AuthType string `json:"auth_type"`
	TargetID string `json:"target_id"`
	Token    string `json:"token"`
}

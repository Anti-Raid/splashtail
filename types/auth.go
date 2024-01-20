package types

type AuthorizeRequest struct {
	ClientID    string `json:"client_id" validate:"required"`
	Code        string `json:"code" validate:"required,min=5"`
	RedirectURI string `json:"redirect_uri" validate:"required"`
	Protocol    string `json:"protocol" validate:"required" description:"Should be 'a1'. This is to identify and block older clients that don't support newer protocols"`
	Scope       string `json:"scope" validate:"required,oneof=normal ban_exempt external_auth"`
}

type UserLogin struct {
	Token  string `json:"token" description:"The users token"`
	UserID string `json:"user_id" description:"The users ID"`
}

type OauthMeta struct {
	ClientID string `json:"client_id" description:"The client ID"`
	URL      string `json:"url" description:"The URL to redirect the user to for discord oauth2"`
}

type TestAuth struct {
	AuthType string `json:"auth_type"`
	TargetID string `json:"target_id"`
	Token    string `json:"token"`
}

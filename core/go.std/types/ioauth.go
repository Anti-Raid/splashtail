package types

import "github.com/bwmarrin/discordgo"

type IOAuthRedirect struct {
	Dest   string   `json:"dest"`
	Scopes []string `json:"scopes"`
}

type IOAuthOutput struct {
	AccessToken  string `json:"access_token"`
	RefreshToken string `json:"refresh_token"`
	ExpiresIn    int    `json:"expires_in"` // Seconds
	Scope        string `json:"scope"`      // Scopes as returned by discord

	// Handled internally

	Scopes      []string               `json:"scopes"`                 // Scopes as a slice
	DiscordUser *discordgo.User        `json:"discord_user,omitempty"` // The discord user
	UserGuilds  []*discordgo.UserGuild `json:"user_guilds,omitempty"`  // The guilds the user is in if 'guilds' is in the scopes
}

type IOAuthDiscordError struct {
	Context map[string]any `json:"context" description:"The context of the error"`
	Message string         `json:"message" description:"The message of the error"`
}

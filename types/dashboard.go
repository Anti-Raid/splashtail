package types

type DashboardGuild struct {
	ID     string `json:"id" description:"The ID of the guild"`
	Name   string `json:"name" description:"The name of the guild"`
	Avatar string `json:"avatar" description:"The avatar url of the guild"`
}

type DashboardGuildData struct {
	Guilds      []*DashboardGuild `json:"guilds" description:"The guilds the user is in"`
	BotInGuilds []string          `json:"has_bot" description:"A list of guild IDs that the user has the bot in"`
}

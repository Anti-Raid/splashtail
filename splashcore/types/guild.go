package types

type DashboardGuild struct {
	ID          string `json:"id" description:"The ID of the guild"`
	Name        string `json:"name" description:"The name of the guild"`
	Avatar      string `json:"avatar" description:"The avatar url of the guild"`
	Permissions int64  `json:"permissions" description:"The permissions the user has in the guild"`
}

type DashboardGuildData struct {
	Guilds        []*DashboardGuild `json:"guilds" description:"The guilds the user is in"`
	BotInGuilds   []string          `json:"has_bot" description:"A list of guild IDs that the user has the bot in"`
	UnknownGuilds []string          `json:"unknown_guilds" description:"A list of guild IDs where the bot is in an outage etc. in"`
}

package ext

import (
	"github.com/bwmarrin/discordgo"
)

type GuildChannelWithPermissions struct {
	User    Permissions        `json:"user"`
	Bot     Permissions        `json:"bot"`
	Channel *discordgo.Channel `json:"channel"`
}

package state

import (
	"context"
	"errors"
	"strconv"

	"github.com/bwmarrin/discordgo"
	"github.com/infinitybotlist/eureka/dovewing"
	"github.com/infinitybotlist/eureka/dovewing/dovetypes"
)

var supportedBotFlags = map[string]int64{
	"BOT_HTTP_INTERACTIONS": 1 << 19, // BOT_HTTP_INTERACTIONS
	"VERIFIED_BOT":          1 << 16, // VERIFIED_BOT
}

func flagsToArray(u *discordgo.User) []string {
	var arr = []string{}
	if u.Bot {
		for flag, val := range supportedBotFlags {
			if int64(u.PublicFlags)&val == val {
				arr = append(arr, flag)
			}
		}
	}
	return arr
}
func discordPlatformStatus(status discordgo.Status) dovetypes.PlatformStatus {
	switch status {
	case discordgo.StatusOnline:
		return dovetypes.PlatformStatusOnline
	case discordgo.StatusIdle:
		return dovetypes.PlatformStatusIdle
	case discordgo.StatusDoNotDisturb:
		return dovetypes.PlatformStatusDoNotDisturb
	default:
		return dovetypes.PlatformStatusOffline
	}
}

type DiscordState struct {
	config      *DiscordStateConfig // Config for the discord state
	initialized bool                // Whether the platform has been initted or not
}
type DiscordStateConfig struct {
	Session        *discordgo.Session  // Discord session
	PreferredGuild string              // Which guilds should be checked first for users, good if theres one guild with the majority of users
	BaseState      *dovewing.BaseState // Base state
}

func (c DiscordStateConfig) New() (*DiscordState, error) {
	if c.Session == nil {
		return nil, errors.New("discord not enabled")
	}
	if c.BaseState == nil {
		return nil, errors.New("base state not provided")
	}
	return &DiscordState{
		config: &c,
	}, nil
}
func (d *DiscordState) PlatformName() string {
	return "discord"
}
func (d *DiscordState) Init() error {
	d.initialized = true
	return nil
}
func (d *DiscordState) Initted() bool {
	return d.initialized
}
func (d *DiscordState) GetState() *dovewing.BaseState {
	return d.config.BaseState
}
func (d *DiscordState) ValidateId(id string) (string, error) {
	// Before wasting time searching state, ensure the ID is actually a valid snowflake
	if _, err := strconv.ParseUint(id, 10, 64); err != nil {
		return "", err
	}
	// For all practical purposes, a simple length check can handle a lot of illegal IDs
	if len(id) <= 16 || len(id) > 20 {
		return "", errors.New("invalid snowflake")
	}
	return id, nil
}
func (d *DiscordState) PlatformSpecificCache(ctx context.Context, id string) (*dovetypes.PlatformUser, error) {
	// First try for main server
	if d.config.PreferredGuild != "" {
		member, err := d.config.Session.State.Member(d.config.PreferredGuild, id)
		if err == nil {
			p, pErr := d.config.Session.State.Presence(d.config.PreferredGuild, id)
			if pErr != nil {
				p = &discordgo.Presence{
					User:   member.User,
					Status: discordgo.StatusOffline,
				}
			}
			return &dovetypes.PlatformUser{
				ID:          id,
				Username:    member.User.Username,
				Avatar:      member.User.AvatarURL(""),
				DisplayName: member.User.GlobalName,
				Bot:         member.User.Bot,
				Flags:       flagsToArray(member.User),
				ExtraData: map[string]any{
					"nickname":        member.Nick,
					"mutual_guild":    d.config.PreferredGuild,
					"preferred_guild": true,
					"public_flags":    member.User.PublicFlags,
				},
				Status: discordPlatformStatus(p.Status),
			}, nil
		}
	}
	for _, guild := range d.config.Session.State.Guilds {
		if guild.ID == d.config.PreferredGuild {
			continue // Already checked
		}
		member, err := d.config.Session.State.Member(guild.ID, id)
		if err == nil {
			p, pErr := d.config.Session.State.Presence(guild.ID, id)
			if pErr != nil {
				p = &discordgo.Presence{
					User:   member.User,
					Status: discordgo.StatusOffline,
				}
			}
			return &dovetypes.PlatformUser{
				ID:          id,
				Username:    member.User.Username,
				Avatar:      member.User.AvatarURL(""),
				DisplayName: member.User.GlobalName,
				Bot:         member.User.Bot,
				Flags:       flagsToArray(member.User),
				ExtraData: map[string]any{
					"nickname":        member.Nick,
					"mutual_guild":    guild.ID,
					"preferred_guild": false,
					"public_flags":    member.User.PublicFlags,
				},
				Status: discordPlatformStatus(p.Status),
			}, nil
		}
	}
	return nil, nil
}
func (d *DiscordState) GetUser(ctx context.Context, id string) (*dovetypes.PlatformUser, error) {
	// Get from discord
	user, err := d.config.Session.User(id)
	if err != nil {
		return nil, err
	}
	return &dovetypes.PlatformUser{
		ID:          id,
		Username:    user.Username,
		Avatar:      user.AvatarURL(""),
		DisplayName: user.GlobalName,
		Bot:         user.Bot,
		Status:      dovetypes.PlatformStatusOffline,
		Flags:       flagsToArray(user),
	}, nil
}

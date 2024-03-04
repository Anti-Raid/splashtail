package common

import (
	"fmt"
	"slices"

	"github.com/anti-raid/splashtail/splashcore/utils"
	"github.com/bwmarrin/discordgo"
)

func CreateChannelAllocations(
	basePerms int64,
	g *discordgo.Guild,
	m *discordgo.Member,
	allowedChannelTypes []discordgo.ChannelType,
	channels []*discordgo.Channel,
	specialAllocs map[string]int,
	perChannel int,
	maxMessages int,
) (map[string]int, error) {
	// Create channel map to allow for easy channel lookup
	var channelMap map[string]*discordgo.Channel = make(map[string]*discordgo.Channel)

	for _, channel := range channels {
		channelMap[channel.ID] = channel
	}

	// Allocations per channel
	var perChannelMap = make(map[string]int)

	// First handle the special allocations
	for channelID, allocation := range specialAllocs {
		if c, ok := channelMap[channelID]; ok {
			// Error on bad channels for special allocations
			if !slices.Contains(allowedChannelTypes, c.Type) {
				return nil, fmt.Errorf("special allocation channel %s is not a valid channel type", c.ID)
			}

			perms := utils.MemberChannelPerms(basePerms, g, m, c)

			if !utils.CheckPermission(perms, discordgo.PermissionViewChannel) {
				return nil, fmt.Errorf("special allocation channel %s is not readable by the bot", c.ID)
			}

			if CountMap(perChannelMap) >= maxMessages {
				continue
			}

			perChannelMap[channelID] = allocation
		}
	}

	for _, channel := range channels {
		// Discard bad channels
		if !slices.Contains(allowedChannelTypes, channel.Type) {
			continue
		}

		perms := utils.MemberChannelPerms(basePerms, g, m, channel)

		if perms&discordgo.PermissionViewChannel != discordgo.PermissionViewChannel {
			continue
		}

		if CountMap(perChannelMap) >= maxMessages {
			perChannelMap[channel.ID] = 0 // We still need to include the channel in the allocations
			continue
		}

		if _, ok := perChannelMap[channel.ID]; !ok {
			perChannelMap[channel.ID] = perChannel
		}
	}

	return perChannelMap, nil
}

func ChannelAllocationStream(
	channelAllocs map[string]int,
	callback func(channelID string, allocation int) (collected int, err error),
	maxMessages int,
	rolloverLeftovers int, // Number of messages to rollover per future channel
) error {
	var totalHandledMessages int
	// Backup messages
	for channelID, allocation := range channelAllocs {
		if allocation == 0 {
			continue
		}

		collected, err := callback(channelID, allocation)

		if err != nil {
			return err
		}

		totalHandledMessages += collected
	}

	if rolloverLeftovers != 0 && totalHandledMessages < maxMessages {
		for channelID, allocation := range channelAllocs {
			if allocation == 0 {
				collected, err := callback(channelID, rolloverLeftovers)

				if err != nil {
					return err
				}

				totalHandledMessages += collected

				if totalHandledMessages >= maxMessages {
					break
				}
			}
		}
	}

	return nil
}

func CountMap(m map[string]int) int {
	var count int

	for _, v := range m {
		count += v
	}

	return count
}

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
	maxMessagesPerChannel int,
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

			if perms&discordgo.PermissionViewChannel != discordgo.PermissionViewChannel {
				return nil, fmt.Errorf("special allocation channel %s is not readable by the bot", c.ID)
			}

			if CountMap(perChannelMap) >= maxMessagesPerChannel {
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

		if CountMap(perChannelMap) >= maxMessagesPerChannel {
			perChannelMap[channel.ID] = 0 // We still need to include the channel in the allocations
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
	rolloverLeftovers bool,
) error {
	// Backup messages
	for channelID, allocation := range channelAllocs {
		if allocation == 0 {
			continue
		}

		collected, err := callback(channelID, allocation)

		if err != nil {
			return err
		}

		var leftovers int
		if collected > allocation {
			leftovers = 0
		} else {
			leftovers = allocation - collected
		}

		if leftovers > 0 && rolloverLeftovers {
			// Find a new channel with 0 allocation
			for channelID, allocation := range channelAllocs {
				if allocation == 0 {
					_, err := callback(channelID, leftovers)

					if err != nil {
						return err
					}
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

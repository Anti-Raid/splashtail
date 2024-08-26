package common

import (
	"fmt"
	"slices"

	"github.com/bwmarrin/discordgo"
	orderedmap "github.com/wk8/go-ordered-map/v2"
	"go.std/utils"
)

// Given a guild and a list of channels, return the channels that are in the list, removing duplicates
//
// This is useful for creating a set of channels to allocate messages counts to etc.
func GetChannelsFromList(g *discordgo.Guild, filterChans []string) []*discordgo.Channel {
	if len(filterChans) == 0 {
		return g.Channels
	}

	// Store all channels selected in a hashmap
	hasChannels := make(map[string]bool, len(filterChans))

	for _, c := range filterChans {
		hasChannels[c] = true
	}

	// Now filter out the channels
	chans := make([]*discordgo.Channel, 0, len(filterChans))

	for _, c := range g.Channels {
		if !hasChannels[c.ID] {
			continue
		}

		chans = append(chans, c)
	}

	return chans
}

type ChannelAllocationMap struct {
	*orderedmap.OrderedMap[string, int]
}

func (c ChannelAllocationMap) TotalAllocations() int {
	var count int

	for pair := c.Oldest(); pair != nil; pair = pair.Next() {
		count += pair.Value
	}

	return count
}

func CreateChannelAllocations(
	basePerms int64,
	g *discordgo.Guild,
	m *discordgo.Member,
	neededPerms []int64,
	allowedChannelTypes []discordgo.ChannelType,
	channels []*discordgo.Channel,
	specialAllocs map[string]int,
	perChannel int,
	maxMessages int,
) (*ChannelAllocationMap, error) {
	// Create channel map to allow for easy channel lookup
	var channelMap = orderedmap.New[string, *discordgo.Channel]()

	for _, channel := range channels {
		channelMap.Set(channel.ID, channel)
	}

	// Allocations per channel
	var perChannelMap = ChannelAllocationMap{
		OrderedMap: orderedmap.New[string, int](),
	}

	// First handle the special allocations
	for channelID, allocation := range specialAllocs {
		if c, ok := channelMap.Get(channelID); ok {
			// Error on bad channels for special allocations
			if !slices.Contains(allowedChannelTypes, c.Type) {
				return nil, fmt.Errorf("special allocation channel %s is not a valid channel type", c.ID)
			}

			perms := utils.MemberChannelPerms(basePerms, g, m, c)

			if !utils.CheckAllPermissions(perms, neededPerms) {
				return nil, fmt.Errorf("special allocation channel %s lacks needed perms: %d", c.ID, neededPerms)
			}

			if perChannelMap.TotalAllocations() >= maxMessages {
				continue
			}

			perChannelMap.Set(channelID, allocation)
		}
	}

	for _, channel := range channels {
		// Discard bad channels
		if !slices.Contains(allowedChannelTypes, channel.Type) {
			continue
		}

		perms := utils.MemberChannelPerms(basePerms, g, m, channel)

		if !utils.CheckAllPermissions(perms, neededPerms) {
			continue
		}

		if perChannelMap.TotalAllocations() >= maxMessages {
			perChannelMap.Set(channel.ID, 0) // We still need to include the channel in the allocations
			continue
		}

		if _, ok := perChannelMap.Get(channel.ID); !ok {
			perChannelMap.Set(channel.ID, perChannel)
		}
	}

	return &perChannelMap, nil
}

func ChannelAllocationStream(
	channelAllocs *ChannelAllocationMap,
	callback func(channelID string, allocation int) (collected int, err error),
	maxMessages int,
	rolloverLeftovers int, // Number of messages to rollover per future channel
) error {
	var totalHandledMessages int

	for pair := channelAllocs.Oldest(); pair != nil; pair = pair.Next() {
		channelID := pair.Key
		allocation := pair.Value

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
		for pair := channelAllocs.Oldest(); pair != nil; pair = pair.Next() {
			channelID := pair.Key
			allocation := pair.Value

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

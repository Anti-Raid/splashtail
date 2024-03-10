package moderation

import (
	"bytes"
	"fmt"
	"time"

	"github.com/anti-raid/splashtail/splashcore/types"
	"github.com/anti-raid/splashtail/splashcore/utils"
	"github.com/anti-raid/splashtail/tasks/common"
	"github.com/anti-raid/splashtail/tasks/taskdef"
	"github.com/anti-raid/splashtail/tasks/taskstate"
	"github.com/bwmarrin/discordgo"
	jsoniter "github.com/json-iterator/go"
	orderedmap "github.com/wk8/go-ordered-map/v2"
	"go.uber.org/zap"
)

var allowedMsgPruneChannelTypes = []discordgo.ChannelType{
	discordgo.ChannelTypeGuildText,
	discordgo.ChannelTypeGuildNews,
	discordgo.ChannelTypeGuildNewsThread,
	discordgo.ChannelTypeGuildPublicThread,
	discordgo.ChannelTypeGuildPrivateThread,
	discordgo.ChannelTypeGuildForum,
}

type MessagePruneTask struct {
	// The ID of the server
	ServerID string

	// Constraints, this is auto-set by the task in jobserver and hence not configurable in this mode.
	Constraints *ModerationConstraints

	// Backup options
	Options MessagePruneOpts

	valid bool
}

func (t *MessagePruneTask) Validate(state taskstate.TaskState) error {
	if t.ServerID == "" {
		return fmt.Errorf("server_id is required")
	}

	opMode := state.OperationMode()
	if opMode == "jobs" {
		t.Constraints = FreePlanModerationConstraints // TODO: Add other constraint types based on plans once we have them
	} else if opMode == "localjobs" {
		if t.Constraints == nil {
			return fmt.Errorf("constraints are required")
		}
	} else {
		return fmt.Errorf("invalid operation mode")
	}

	if t.Options.MaxMessages == 0 {
		t.Options.MaxMessages = t.Constraints.MessagePrune.TotalMaxMessages
	}

	if t.Options.PerChannel < t.Constraints.MessagePrune.MinPerChannel {
		return fmt.Errorf("per_channel cannot be less than %d", t.Constraints.MessagePrune.MinPerChannel)
	}

	if t.Options.MaxMessages > t.Constraints.MessagePrune.TotalMaxMessages {
		return fmt.Errorf("max_messages cannot be greater than %d", t.Constraints.MessagePrune.TotalMaxMessages)
	}

	if t.Options.PerChannel > t.Options.MaxMessages {
		return fmt.Errorf("per_channel cannot be greater than max_messages")
	}

	if len(t.Options.SpecialAllocations) == 0 {
		t.Options.SpecialAllocations = make(map[string]int)
	}

	// Check current moderation concurrency
	count, _ := concurrentModerationState.LoadOrStore(t.ServerID, 0)

	if count >= t.Constraints.MaxServerModerationTasks {
		return fmt.Errorf("you already have more than %d moderation tasks in progress, please wait for it to finish", t.Constraints.MaxServerModerationTasks)
	}

	t.valid = true

	return nil
}

func (t *MessagePruneTask) Exec(
	l *zap.Logger,
	tcr *types.TaskCreateResponse,
	state taskstate.TaskState,
	progstate taskstate.TaskProgressState,
) (*types.TaskOutput, error) {
	discord, botUser, _ := state.Discord()
	ctx := state.Context()

	// Check current moderation concurrency
	count, _ := concurrentModerationState.LoadOrStore(t.ServerID, 0)

	if count >= t.Constraints.MaxServerModerationTasks {
		return nil, fmt.Errorf("you already have more than %d moderation tasks in progress, please wait for it to finish", t.Constraints.MaxServerModerationTasks)
	}

	concurrentModerationState.Store(t.ServerID, count+1)

	// Decrement count when we're done
	defer func() {
		countNow, _ := concurrentModerationState.LoadOrStore(t.ServerID, 0)

		if countNow > 0 {
			concurrentModerationState.Store(t.ServerID, countNow-1)
		}
	}()

	l.Info("Fetching bots current state in server")
	m, err := discord.GuildMember(t.ServerID, botUser.ID, discordgo.WithContext(ctx))

	if err != nil {
		return nil, fmt.Errorf("error fetching bots member object: %w", err)
	}

	// Fetch guild
	g, err := discord.Guild(t.ServerID, discordgo.WithContext(ctx))

	if err != nil {
		return nil, fmt.Errorf("error fetching guild: %w", err)
	}

	// Fetch roles first before calculating base permissions
	if len(g.Roles) == 0 {
		roles, err := discord.GuildRoles(t.ServerID, discordgo.WithContext(ctx))

		if err != nil {
			return nil, fmt.Errorf("error fetching roles: %w", err)
		}

		g.Roles = roles
	}

	if len(g.Channels) == 0 {
		channels, err := discord.GuildChannels(t.ServerID, discordgo.WithContext(ctx))

		if err != nil {
			return nil, fmt.Errorf("error fetching channels: %w", err)
		}

		g.Channels = channels
	}

	// With servers now fully populated, get the base permissions now
	basePerms := utils.BasePermissions(g, m)

	if basePerms&discordgo.PermissionManageMessages != discordgo.PermissionManageMessages && basePerms&discordgo.PermissionAdministrator != discordgo.PermissionAdministrator {
		return nil, fmt.Errorf("bot does not have 'Manage Messages' permissions")
	}

	perChannelBackupMap, err := common.CreateChannelAllocations(
		basePerms,
		g,
		m,
		[]int64{discordgo.PermissionViewChannel, discordgo.PermissionReadMessageHistory, discordgo.PermissionManageMessages},
		allowedMsgPruneChannelTypes,
		func() []*discordgo.Channel {
			if len(t.Options.Channels) == 0 {
				return g.Channels
			}

			// Store all channels selected in a hashmap
			hasChannels := make(map[string]bool, len(t.Options.Channels))

			for _, c := range t.Options.Channels {
				hasChannels[c] = true
			}

			// Now filter out the channels
			chans := make([]*discordgo.Channel, 0, len(t.Options.Channels))

			for _, c := range g.Channels {
				if !hasChannels[c.ID] {
					continue
				}

				chans = append(chans, c)
			}

			return chans
		}(),
		t.Options.SpecialAllocations,
		t.Options.PerChannel,
		t.Options.MaxMessages,
	)

	if err != nil {
		return nil, fmt.Errorf("error creating channel allocations: %w", err)
	}

	l.Info("Created channel backup allocations", zap.Any("alloc", perChannelBackupMap), zap.Strings("botDisplayIgnore", []string{"alloc"}))

	// Now handle all the channel allocations
	var finalMessagesEnd = orderedmap.New[string, []*discordgo.Message]()
	err = common.ChannelAllocationStream(
		perChannelBackupMap,
		func(channelID string, allocation int) (collected int, err error) {
			// Fetch messages and bulk delete
			currentId := ""
			finalMsgs := make([]*discordgo.Message, 0, allocation)
			for {
				// Fetch messages
				if allocation < len(finalMsgs) {
					// We've gone over, break
					break
				}

				limit := min(100, allocation-len(finalMsgs))

				l.Info("Fetching messages", zap.String("channelID", channelID), zap.Int("limit", limit), zap.String("currentId", currentId))

				// Fetch messages
				messages, err := discord.ChannelMessages(
					channelID,
					limit,
					"",
					currentId,
					"",
					discordgo.WithContext(ctx),
				)

				if err != nil {
					return len(finalMsgs), fmt.Errorf("error fetching messages: %w", err)
				}

				if len(messages) == 0 {
					break
				}

				var messageList = make([]string, 0, len(messages))

				var twoWeeksAgo = time.Now().Add(-14 * 24 * time.Hour)
				for _, m := range messages {
					// Check that the message is under 14 days old
					if m.Timestamp.Before(twoWeeksAgo) {
						continue
					}

					messageList = append(messageList, m.ID)
					finalMsgs = append(finalMsgs, m)
				}

				if len(messageList) == 0 {
					break
				}

				// Bulk delete
				err = discord.ChannelMessagesBulkDelete(channelID, messageList, discordgo.WithContext(ctx))

				if err != nil {
					return len(finalMsgs), fmt.Errorf("error bulk deleting messages: %w", err)
				}

				if len(messages) < allocation {
					// We've reached the end
					break
				}
			}

			finalMessagesEnd.Set(channelID, finalMsgs)

			return 0, nil
		},
		t.Options.MaxMessages,
		func() int {
			if t.Options.RolloverLeftovers {
				return t.Options.PerChannel
			}

			return 0
		}(),
	)

	if err != nil {
		return nil, fmt.Errorf("error handling channel allocations: %w", err)
	}

	var outputBuf bytes.Buffer

	// Write to buffer
	err = jsoniter.ConfigFastest.NewEncoder(&outputBuf).Encode(finalMessagesEnd)

	if err != nil {
		return nil, fmt.Errorf("error encoding final messages: %w", err)
	}

	return &types.TaskOutput{
		Filename: "pruned-messages.txt",
		Buffer:   &outputBuf,
	}, nil
}

func (t *MessagePruneTask) Info() *types.TaskInfo {
	return &types.TaskInfo{
		Name: "message_prune",
		TaskFor: &types.TaskFor{
			ID:         t.ServerID,
			TargetType: types.TargetTypeServer,
		},
		TaskFields: t,
		Valid:      t.valid,
	}
}

func (t *MessagePruneTask) LocalPresets() *taskdef.PresetInfo {
	return &taskdef.PresetInfo{
		Runnable: true,
		Preset: &MessagePruneTask{
			ServerID: "{{.Args.ServerID}}",
			Constraints: &ModerationConstraints{
				MessagePrune: &MessagePruneConstraints{
					TotalMaxMessages: 1000,
					MinPerChannel:    10,
				},
				MaxServerModerationTasks: 1,
			},
			Options: MessagePruneOpts{
				PerChannel: 100,
			},
		},
		Comments: map[string]string{
			"Constraints.MaxServerModerationTasks":      "Only 1 mod task should be running at any given time locally",
			"Constraints.MessagePrune.TotalMaxMessages": "We can be more generous here with 1000 by default",
			"Constraints.MessagePrune.MinPerChannel":    "We can be more generous here with 10 by default",
		},
	}
}

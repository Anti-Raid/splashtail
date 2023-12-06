package backups

import (
	"encoding/json"
	"fmt"
	"os"
	"slices"
	"splashtail/state"
	"splashtail/tasks"
	"time"

	"github.com/bwmarrin/discordgo"
	"github.com/infinitybotlist/iblfile"
	"github.com/jackc/pgx/v5"
	"go.uber.org/zap"
)

func init() {
	tasks.RegisterTask(&ServerBackupCreateTask{})
}

func countMap(m map[string]int) int {
	var count int

	for _, v := range m {
		count += v
	}

	return count
}

func backupChannelMessages(channelID string, allocation int) ([]*discordgo.Message, error) {
	var finalMsgs []*discordgo.Message
	var currentId string
	for {
		// Fetch messages
		if allocation < len(finalMsgs) {
			// We've gone over, break
			break
		}

		limit := min(100, allocation-len(finalMsgs))

		messages, err := state.Discord.ChannelMessages(channelID, limit, "", currentId, "")

		if err != nil {
			return nil, fmt.Errorf("error fetching messages: %w", err)
		}

		finalMsgs = append(finalMsgs, messages...)

		if len(messages) < limit {
			// We've reached the end
			break
		}
	}

	return finalMsgs, nil
}

// A task to create backup a server
type ServerBackupCreateTask struct {
	// The ID of the task
	TaskID string `json:"task_id"`

	// The ID of the server
	ServerID string `json:"server_id"`

	// Backup options
	BackupOpts BackupOpts `json:"backup_opts"`
}

// $SecureStorage/guilds/$guildId/backups/$taskId
func (t *ServerBackupCreateTask) dir() string {
	return fmt.Sprintf("%s/guilds/%s/backups/%s", state.Config.Meta.SecureStorage, t.ServerID, t.TaskID)
}

// $SecureStorage/guilds/$guildId/backups/$taskId/backup.arbackup
func (t *ServerBackupCreateTask) path() string {
	return t.dir() + "/backup.arbackup"
}

func (t *ServerBackupCreateTask) Validate() error {
	if t.ServerID == "" {
		return fmt.Errorf("server_id is required")
	}

	return nil
}

func (t *ServerBackupCreateTask) Exec(l *zap.Logger, tx pgx.Tx) error {
	l.Info("Beginning backup")

	if t.BackupOpts.MaxMessages == 0 {
		t.BackupOpts.MaxMessages = totalMaxMessages
	}

	if t.BackupOpts.PerChannel == 0 {
		t.BackupOpts.PerChannel = 100
	}

	if t.BackupOpts.MaxMessages > totalMaxMessages {
		return fmt.Errorf("max_messages cannot be greater than %d", totalMaxMessages)
	}

	if t.BackupOpts.PerChannel > t.BackupOpts.MaxMessages {
		return fmt.Errorf("per_channel cannot be greater than max_messages")
	}

	if t.BackupOpts.BackupAttachments && !t.BackupOpts.BackupMessages {
		return fmt.Errorf("cannot backup attachments without messages")
	}

	if len(t.BackupOpts.SpecialAllocations) == 0 {
		t.BackupOpts.SpecialAllocations = make(map[string]int)
	}

	f, err := iblfile.NewAutoEncryptedFile("")

	if err != nil {
		return fmt.Errorf("error creating file: %w", err)
	}

	f.WriteJsonSection(t.BackupOpts, "backup_opts")

	l.Info("Backing up guild settings")

	// Fetch guild
	g, err := state.Discord.Guild(t.ServerID)

	if err != nil {
		return fmt.Errorf("error fetching guild: %w", err)
	}

	l.Info("Backing up guild channels")

	// Fetch channels of guild
	channels, err := state.Discord.GuildChannels(t.ServerID)

	if err != nil {
		return fmt.Errorf("error fetching channels: %w", err)
	}

	g.Channels = channels

	cb := CoreBackup{
		Guild: g,
	}

	if len(g.Roles) == 0 {
		l.Info("Backing up guild roles", zap.String("taskId", t.TaskID))

		// Fetch roles of guild
		roles, err := state.Discord.GuildRoles(t.ServerID)

		if err != nil {
			return fmt.Errorf("error fetching roles: %w", err)
		}

		g.Roles = roles
	}

	if len(g.Stickers) == 0 {
		l.Info("Backing up guild stickers", zap.String("taskId", t.TaskID))

		// Fetch stickers of guild
		stickers, err := state.Discord.Request("GET", discordgo.EndpointGuildStickers(t.ServerID), nil)

		if err != nil {
			return fmt.Errorf("error fetching stickers: %w", err)
		}

		var s []*discordgo.Sticker

		err = json.Unmarshal(stickers, &s)

		if err != nil {
			return fmt.Errorf("error unmarshalling stickers: %w", err)
		}

		g.Stickers = s
	}

	f.WriteJsonSection(cb, "core")

	// Backup messages
	if t.BackupOpts.BackupMessages {
		l.Info("Calculating message backup allocations", zap.String("taskId", t.TaskID))

		// Create channel map to allow for easy channel lookup
		var channelMap map[string]*discordgo.Channel = make(map[string]*discordgo.Channel)

		for _, channel := range channels {
			channelMap[channel.ID] = channel
		}

		// Allocations per channel
		var perChannelBackupMap = make(map[string]int)

		// First handle the special allocations
		for channelID, allocation := range t.BackupOpts.SpecialAllocations {
			if _, ok := channelMap[channelID]; ok {
				perChannelBackupMap[channelID] = allocation
			}
		}

		allowedChannelTypes := []discordgo.ChannelType{
			discordgo.ChannelTypeGuildText,
			discordgo.ChannelTypeGuildNews,
			discordgo.ChannelTypeGuildNewsThread,
			discordgo.ChannelTypeGuildPublicThread,
			discordgo.ChannelTypeGuildPrivateThread,
			discordgo.ChannelTypeGuildForum,
		}

		for _, channel := range channels {
			// Discard bad channels
			if !slices.Contains(allowedChannelTypes, channel.Type) {
				continue
			}

			if countMap(perChannelBackupMap) >= t.BackupOpts.MaxMessages {
				perChannelBackupMap[channel.ID] = 0 // We still need to include the channel in the allocations
			}

			if _, ok := perChannelBackupMap[channel.ID]; !ok {
				perChannelBackupMap[channel.ID] = t.BackupOpts.PerChannel
			}
		}

		l.Info("Created channel backup allocations", zap.Any("alloc", perChannelBackupMap), zap.Strings("botDisplayIgnore", []string{"alloc"}))

		// Backup messages
		for channelID, allocation := range perChannelBackupMap {
			if allocation == 0 {
				continue
			}

			l.Info("Backing up channel messages", zap.String("channelId", channelID))

			var leftovers int

			msgs, err := backupChannelMessages(channelID, allocation)

			if err != nil {
				if t.BackupOpts.IgnoreMessageBackupErrors {
					l.Error("error backing up channel messages", zap.Error(err))
					leftovers = allocation
				} else {
					return fmt.Errorf("error backing up channel messages: %w", err)
				}
			} else {
				if len(msgs) < allocation {
					leftovers = allocation - len(msgs)
				}

				// Write messages of this section
				f.WriteJsonSection(msgs, "messages/"+channelID)
			}

			if leftovers > 0 && t.BackupOpts.RolloverLeftovers {
				// Find a new channel with 0 allocation
				for channelID, allocation := range perChannelBackupMap {
					if allocation == 0 {
						msgs, err := backupChannelMessages(channelID, leftovers)

						if err != nil {
							if t.BackupOpts.IgnoreMessageBackupErrors {
								l.Error("error backing up channel messages [leftovers]", zap.Error(err))
								continue // Try again
							} else {
								return fmt.Errorf("error backing up channel messages [leftovers]: %w", err)
							}
						} else {
							// Write messages of this section
							f.WriteJsonSection(msgs, "messages/"+channelID)
							break
						}
					}
				}
			}
		}
	}

	metadata := iblfile.Meta{
		CreatedAt:      time.Now(),
		Protocol:       iblfile.Protocol,
		Type:           fileType,
		EncryptionData: f.EncDataMap,
	}

	ifmt, err := iblfile.GetFormat(fileType)

	if err != nil {
		l.Error("Error creating backup", zap.Error(err))
		return fmt.Errorf("error getting format: %w", err)
	}

	metadata.FormatVersion = ifmt.Version

	err = f.WriteJsonSection(metadata, "meta")

	if err != nil {
		l.Error("Error creating backup", zap.Error(err))
		return fmt.Errorf("error writing metadata: %w", err)
	}

	// Create dir
	err = os.MkdirAll(t.dir(), 0700)

	if err != nil {
		l.Error("Failed to create directory", zap.Error(err), zap.String("id", t.ServerID))
		return fmt.Errorf("error creating directory: %w", err)
	}

	// Write backup to path
	file, err := os.Create(t.path())

	if err != nil {
		l.Error("Failed to create file", zap.Error(err), zap.String("id", t.ServerID))
		return fmt.Errorf("error creating file: %w", err)
	}

	defer file.Close()

	err = f.WriteOutput(file)

	if err != nil {
		l.Error("Failed to write backup", zap.Error(err), zap.String("id", t.ServerID))
		return fmt.Errorf("error writing backup: %w", err)
	}

	l.Info("Successfully created backup", zap.String("id", t.ServerID))

	return nil
}

func (t *ServerBackupCreateTask) Info() *tasks.TaskInfo {
	return &tasks.TaskInfo{
		TaskID:     t.TaskID,
		Name:       "create_backup",
		For:        tasks.Pointer("g/" + t.ServerID),
		TaskFields: t,
		Expiry:     1 * time.Hour,
	}
}

func (t *ServerBackupCreateTask) Output() *tasks.TaskOutput {
	return &tasks.TaskOutput{
		Path: t.path(),
	}
}

func (t *ServerBackupCreateTask) Set(set *tasks.TaskSet) tasks.Task {
	t.TaskID = set.TaskID

	return t
}

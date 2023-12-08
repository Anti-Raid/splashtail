package backups

import (
	"bytes"
	"compress/gzip"
	"encoding/json"
	"fmt"
	"image"
	_ "image/gif"
	"image/jpeg"
	_ "image/png"
	"io"
	"net/http"
	"slices"
	"splashtail/state"
	"splashtail/tasks"
	"splashtail/types"
	"splashtail/utils"
	"time"

	_ "golang.org/x/image/webp"

	"github.com/bwmarrin/discordgo"
	"github.com/infinitybotlist/iblfile"
	"github.com/jackc/pgx/v5"
	"github.com/vmihailenco/msgpack/v5"
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

// Backs up messages of a channel
//
// Note that attachments are only backed up if withAttachments is true and f.Size() < fileSizeWarningThreshold
//
// Note that this function does not write the messages to the file, it only returns them
func backupChannelMessages(logger *zap.Logger, f *iblfile.AutoEncryptedFile, channelID string, allocation int, withAttachments bool) ([]*BackupMessage, error) {
	var finalMsgs []*BackupMessage
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

		for _, msg := range messages {
			im := BackupMessage{
				Message: msg,
			}

			if withAttachments && f.Size() < fileSizeWarningThreshold {
				am, bufs, err := createAttachmentBlob(logger, msg)

				if err != nil {
					return nil, fmt.Errorf("error creating attachment blob: %w", err)
				}

				im.AttachmentMetadata = am
				im.attachments = bufs
			}

			finalMsgs = append(finalMsgs, &im)
		}

		if len(messages) < limit {
			// We've reached the end
			break
		}
	}

	return finalMsgs, nil
}

func createAttachmentBlob(logger *zap.Logger, msg *discordgo.Message) ([]AttachmentMetadata, map[string]*bytes.Buffer, error) {
	var attachments []AttachmentMetadata
	var bufs = map[string]*bytes.Buffer{}
	for _, attachment := range msg.Attachments {
		if attachment.Size > maxAttachmentFileSize {
			attachments = append(attachments, AttachmentMetadata{
				ID:          attachment.ID,
				Name:        attachment.Filename,
				URL:         attachment.URL,
				ProxyURL:    attachment.ProxyURL,
				Size:        attachment.Size,
				ContentType: attachment.ContentType,
				Errors:      []string{"Attachment is too large to be saved."},
			})
			continue
		}

		// Download the attachment
		var url string

		if attachment.ProxyURL != "" {
			url = attachment.ProxyURL
		} else {
			url = attachment.URL
		}

		/*if !strings.HasPrefix(url, "https://cdn.discordapp.com") {
			attachments = append(attachments, AttachmentMetadata{
				ID:            attachment.ID,
				Name:          attachment.Filename,
				URL:           attachment.URL,
				ProxyURL:      attachment.ProxyURL,
				Size:          attachment.Size,
				ContentType:   attachment.ContentType,
				StorageFormat: AttachmentStorageFormatRemote,
				Errors:        []string{},
			})
			continue
		}*/

		resp, err := http.Get(url)

		if err != nil {
			logger.Error("Error downloading attachment", zap.Error(err), zap.String("url", url))
			return attachments, nil, fmt.Errorf("error downloading attachment: %w", err)
		}

		bt, err := io.ReadAll(resp.Body)

		if err != nil {
			logger.Error("Error reading attachment", zap.Error(err), zap.String("url", url))
			return attachments, nil, fmt.Errorf("error reading attachment: %w", err)
		}

		bufs[attachment.ID] = bytes.NewBuffer(bt)

		am := AttachmentMetadata{
			ID:            attachment.ID,
			Name:          attachment.Filename,
			URL:           attachment.URL,
			ProxyURL:      attachment.ProxyURL,
			Size:          attachment.Size,
			ContentType:   attachment.ContentType,
			StorageFormat: AttachmentStorageFormatUncompressed,
			Errors:        []string{},
		}

		switch attachment.ContentType {
		case "video/mp4", "video/webm":
			// We don't support compressing these yet, so just use uncompressed
			attachments = append(attachments, am)
		case "image/jpeg", "image/png", "image/gif", "image/webp":
			var img image.Image

			img, _, err = image.Decode(bytes.NewReader(bt))

			// We don't support compressing these yet, so just use uncompressed
			if err != nil {
				logger.Error("Error decoding attachment", zap.Error(err), zap.String("url", url))
				attachments = append(attachments, am)
				continue
			}

			var buf bytes.Buffer
			err := jpeg.Encode(&buf, img, &jpeg.Options{
				Quality: jpegReencodeQuality,
			})

			if err != nil {
				logger.Error("Error encoding attachment", zap.Error(err), zap.String("url", url))
				attachments = append(attachments, am)
				continue
			}

			am.StorageFormat = AttachmentStorageFormatJpegEncoded

			bufs[attachment.ID] = &buf
			attachments = append(attachments, am)
		case "text/plain", "text/html", "application/octet-stream":
			// Gzip compress
			am.StorageFormat = AttachmentStorageFormatGzip

			var buf bytes.Buffer
			gz := gzip.NewWriter(&buf)
			gz.Write(bt)

			err = gz.Close()

			if err != nil {
				logger.Error("Error gzipping attachment", zap.Error(err), zap.String("url", url))
				return attachments, nil, fmt.Errorf("error gzipping attachment: %w", err)
			}

			bufs[attachment.ID] = &buf

			attachments = append(attachments, am)
		default:
			attachments = append(attachments, am)
		}
	}

	return attachments, bufs, nil
}

func writeMsgpack(f *iblfile.AutoEncryptedFile, section string, data any) error {
	var buf bytes.Buffer
	enc := msgpack.NewEncoder(&buf)
	enc.SetCustomStructTag("json")
	enc.UseCompactInts(true)
	enc.UseCompactFloats(true)
	enc.UseInternedStrings(true)
	err := enc.Encode(data)

	if err != nil {
		return fmt.Errorf("error marshalling data: %w", err)
	}

	return f.WriteSection(&buf, section)
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

func (t *ServerBackupCreateTask) Validate() error {
	if t.ServerID == "" {
		return fmt.Errorf("server_id is required")
	}

	return nil
}

func (t *ServerBackupCreateTask) Exec(l *zap.Logger, tx pgx.Tx) (*tasks.TaskOutput, error) {
	l.Info("Beginning backup")

	if t.BackupOpts.MaxMessages == 0 {
		t.BackupOpts.MaxMessages = totalMaxMessages
	}

	if t.BackupOpts.PerChannel == 0 {
		t.BackupOpts.PerChannel = defaultPerChannel
	}

	if t.BackupOpts.PerChannel < minPerChannel {
		return nil, fmt.Errorf("per_channel cannot be less than %d", minPerChannel)
	}

	if t.BackupOpts.MaxMessages > totalMaxMessages {
		return nil, fmt.Errorf("max_messages cannot be greater than %d", totalMaxMessages)
	}

	if t.BackupOpts.PerChannel > t.BackupOpts.MaxMessages {
		return nil, fmt.Errorf("per_channel cannot be greater than max_messages")
	}

	if t.BackupOpts.BackupAttachments && !t.BackupOpts.BackupMessages {
		return nil, fmt.Errorf("cannot backup attachments without messages")
	}

	if len(t.BackupOpts.SpecialAllocations) == 0 {
		t.BackupOpts.SpecialAllocations = make(map[string]int)
	}

	f, err := iblfile.NewAutoEncryptedFile("")

	if err != nil {
		return nil, fmt.Errorf("error creating file: %w", err)
	}

	err = writeMsgpack(f, "backup_opts", t.BackupOpts)

	if err != nil {
		return nil, fmt.Errorf("error writing backup options: %w", err)
	}

	// Fetch the bots member object in the guild
	l.Info("Fetching bots current state in server")
	m, err := state.Discord.GuildMember(t.ServerID, state.BotUser.ID)

	if err != nil {
		return nil, fmt.Errorf("error fetching bots member object: %w", err)
	}

	err = writeMsgpack(f, "dbg/bot", m) // Write bot member object to debug section

	if err != nil {
		return nil, fmt.Errorf("error writing bot member object: %w", err)
	}

	l.Info("Backing up server settings")

	// Fetch guild
	g, err := state.Discord.Guild(t.ServerID)

	if err != nil {
		return nil, fmt.Errorf("error fetching guild: %w", err)
	}

	// With servers now backed up, get the base permissions now
	basePerms := utils.BasePermissions(g, m)

	// Write base permissions to debug section
	err = writeMsgpack(f, "dbg/basePerms", basePerms)

	if err != nil {
		return nil, fmt.Errorf("error writing base permissions: %w", err)
	}

	l.Info("Backing up guild channels")

	// Fetch channels of guild
	channels, err := state.Discord.GuildChannels(t.ServerID)

	if err != nil {
		return nil, fmt.Errorf("error fetching channels: %w", err)
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
			return nil, fmt.Errorf("error fetching roles: %w", err)
		}

		g.Roles = roles
	}

	if len(g.Stickers) == 0 {
		l.Info("Backing up guild stickers", zap.String("taskId", t.TaskID))

		// Fetch stickers of guild
		stickers, err := state.Discord.Request("GET", discordgo.EndpointGuildStickers(t.ServerID), nil)

		if err != nil {
			return nil, fmt.Errorf("error fetching stickers: %w", err)
		}

		var s []*discordgo.Sticker

		err = json.Unmarshal(stickers, &s)

		if err != nil {
			return nil, fmt.Errorf("error unmarshalling stickers: %w", err)
		}

		g.Stickers = s
	}

	// Write core backup
	err = writeMsgpack(f, "core", cb)

	if err != nil {
		return nil, fmt.Errorf("error writing core backup: %w", err)
	}

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
			if c, ok := channelMap[channelID]; ok {
				// Error on bad channels for special allocations
				if !slices.Contains(allowedChannelTypes, c.Type) {
					return nil, fmt.Errorf("special allocation channel %s is not a valid channel type", c.ID)
				}

				perms := utils.MemberChannelPerms(basePerms, g, m, c)

				if perms&discordgo.PermissionViewChannel != discordgo.PermissionViewChannel {
					return nil, fmt.Errorf("special allocation channel %s is not readable by the bot", c.ID)
				}

				if countMap(perChannelBackupMap) >= t.BackupOpts.MaxMessages {
					continue
				}

				perChannelBackupMap[channelID] = allocation
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

			if countMap(perChannelBackupMap) >= t.BackupOpts.MaxMessages {
				perChannelBackupMap[channel.ID] = 0 // We still need to include the channel in the allocations
			}

			if _, ok := perChannelBackupMap[channel.ID]; !ok {
				perChannelBackupMap[channel.ID] = t.BackupOpts.PerChannel
			}
		}

		l.Info("Created channel backup allocations", zap.Any("alloc", perChannelBackupMap), zap.Strings("botDisplayIgnore", []string{"alloc"}))

		err = writeMsgpack(f, "dbg/chanAlloc", perChannelBackupMap)

		if err != nil {
			return nil, fmt.Errorf("error writing channel allocations: %w", err)
		}

		// Backup messages
		for channelID, allocation := range perChannelBackupMap {
			if allocation == 0 {
				continue
			}

			l.Info("Backing up channel messages", zap.String("channelId", channelID))

			var leftovers int

			msgs, err := backupChannelMessages(l, f, channelID, allocation, t.BackupOpts.BackupAttachments)

			if err != nil {
				if t.BackupOpts.IgnoreMessageBackupErrors {
					l.Error("error backing up channel messages", zap.Error(err))
					leftovers = allocation
				} else {
					return nil, fmt.Errorf("error backing up channel messages: %w", err)
				}
			} else {
				if len(msgs) < allocation {
					leftovers = allocation - len(msgs)
				}

				// Write messages of this section
				err = writeMsgpack(f, "messages/"+channelID, msgs)

				if err != nil {
					return nil, fmt.Errorf("error writing messages: %w", err)
				}

				for _, msg := range msgs {
					if len(msg.attachments) > 0 {
						for id, buf := range msg.attachments {
							f.WriteSection(buf, "attachments/"+id)
						}
					}
				}
			}

			if leftovers > 0 && t.BackupOpts.RolloverLeftovers {
				// Find a new channel with 0 allocation
				for channelID, allocation := range perChannelBackupMap {
					if allocation == 0 {
						msgs, err := backupChannelMessages(l, f, channelID, leftovers, t.BackupOpts.BackupAttachments)

						if err != nil {
							if t.BackupOpts.IgnoreMessageBackupErrors {
								l.Error("error backing up channel messages [leftovers]", zap.Error(err))
								continue // Try again
							} else {
								return nil, fmt.Errorf("error backing up channel messages [leftovers]: %w", err)
							}
						} else {
							// Write messages of this section
							err = writeMsgpack(f, "messages/"+channelID, msgs)

							if err != nil {
								return nil, fmt.Errorf("error writing messages [leftovers]: %w", err)
							}

							for _, msg := range msgs {
								if len(msg.attachments) > 0 {
									for id, buf := range msg.attachments {
										f.WriteSection(buf, "attachments/"+id)
									}
								}
							}
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
		return nil, fmt.Errorf("error getting format: %w", err)
	}

	metadata.FormatVersion = ifmt.Version

	err = f.WriteJsonSection(metadata, "meta")

	if err != nil {
		l.Error("Error creating backup", zap.Error(err))
		return nil, fmt.Errorf("error writing metadata: %w", err)
	}

	// Save file
	var outputBuf bytes.Buffer

	err = f.WriteOutput(&outputBuf)

	if err != nil {
		l.Error("Failed to write backup to temporary buffer", zap.Error(err))
		return nil, fmt.Errorf("error writing backup: %w", err)
	}

	return &tasks.TaskOutput{
		Filename: "antiraid-backup.iblfile",
	}, nil
}

func (t *ServerBackupCreateTask) Info() *tasks.TaskInfo {
	return &tasks.TaskInfo{
		TaskID: t.TaskID,
		Name:   "create_backup",
		TaskFor: &types.TaskFor{
			ID:         t.ServerID,
			TargetType: types.TargetTypeServer,
		},
		TaskFields: t,
		Expiry:     1 * time.Hour,
	}
}

func (t *ServerBackupCreateTask) Set(set *tasks.TaskSet) tasks.Task {
	t.TaskID = set.TaskID

	return t
}

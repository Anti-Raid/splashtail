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
	"time"

	"github.com/anti-raid/splashtail/core/go.jobs/common"
	"github.com/anti-raid/splashtail/core/go.jobs/taskdef"
	"github.com/anti-raid/splashtail/core/go.jobs/taskstate"
	"github.com/anti-raid/splashtail/core/go.std/types"
	"github.com/anti-raid/splashtail/core/go.std/utils"

	_ "golang.org/x/image/webp"

	"github.com/bwmarrin/discordgo"
	"github.com/infinitybotlist/iblfile"
	"github.com/infinitybotlist/iblfile/encryptors/aes256"
	"github.com/infinitybotlist/iblfile/encryptors/noencryption"
	"github.com/vmihailenco/msgpack/v5"
	"go.uber.org/zap"
)

// Backs up image data to a file
func backupGuildAsset(state taskstate.TaskState, constraints *BackupConstraints, l *zap.Logger, f *iblfile.AutoEncryptedFile_FullFile, name, url string) error {
	l.Info("Backing up guild asset", zap.String("name", name))
	ctx := state.Context()
	client := http.Client{
		Timeout:   10 * time.Second,
		Transport: state.Transport(),
	}

	req, err := http.NewRequestWithContext(ctx, "GET", url, nil)

	if err != nil {
		return fmt.Errorf("error creating guild asset request: %w", err)
	}

	resp, err := client.Do(req)

	if err != nil {
		return fmt.Errorf("error fetching guild asset: %w", err)
	}

	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("error fetching guild asset: %w", fmt.Errorf("status code %d", resp.StatusCode))
	}

	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)

	if err != nil {
		return fmt.Errorf("error reading guild asset: %w", err)
	}

	// Re-encode to jpeg
	img, _, err := image.Decode(bytes.NewReader(body))

	if err != nil {
		return fmt.Errorf("error decoding guild asset: %w", err)
	}

	var buf bytes.Buffer

	err = jpeg.Encode(&buf, img, &jpeg.Options{
		Quality: constraints.Create.GuildAssetReencodeQuality,
	})

	if err != nil {
		return fmt.Errorf("error re-encoding guild asset: %w", err)
	}

	select {
	case <-ctx.Done():
		return ctx.Err()
	default:
	}

	f.WriteSection(&buf, "assets/"+name)
	return nil
}

// Backs up messages of a channel
//
// Note that attachments are only backed up if withAttachments is true and f.Size() < fileSizeWarningThreshold
//
// Note that this function does not write the messages to the file, it only returns them
func backupChannelMessages(state taskstate.TaskState, constraints *BackupConstraints, logger *zap.Logger, f *iblfile.AutoEncryptedFile_FullFile, channelID string, allocation int, withAttachments bool) ([]*BackupMessage, error) {
	discord, _, _ := state.Discord()
	ctx := state.Context()

	var finalMsgs []*BackupMessage
	var currentId string
	for {
		// Fetch messages
		if allocation < len(finalMsgs) {
			// We've gone over, break
			break
		}

		limit := min(100, allocation-len(finalMsgs))

		messages, err := discord.ChannelMessages(channelID, limit, currentId, "", "", discordgo.WithContext(ctx))

		if err != nil {
			return nil, fmt.Errorf("error fetching messages: %w", err)
		}

		for _, msg := range messages {
			im := BackupMessage{
				Message: msg,
			}

			if withAttachments && f.Size() < constraints.Create.FileSizeWarningThreshold {
				am, bufs, err := createAttachmentBlob(state, constraints, logger, msg)

				if err != nil {
					finalMsgs = append(finalMsgs, &im)
					return finalMsgs, fmt.Errorf("error creating attachment blob: %w", err)
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

func createAttachmentBlob(state taskstate.TaskState, constraints *BackupConstraints, logger *zap.Logger, msg *discordgo.Message) ([]AttachmentMetadata, map[string]*bytes.Buffer, error) {
	ctx := state.Context()

	var attachments []AttachmentMetadata
	var bufs = map[string]*bytes.Buffer{}
	for _, attachment := range msg.Attachments {
		select {
		case <-ctx.Done():
			return nil, nil, ctx.Err()
		default:
		}

		if attachment.Size > constraints.Create.MaxAttachmentFileSize {
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

		client := http.Client{
			Timeout:   10 * time.Second,
			Transport: state.Transport(),
		}

		req, err := http.NewRequestWithContext(ctx, "GET", url, nil)

		if err != nil {
			attachments = append(attachments, AttachmentMetadata{
				ID:          attachment.ID,
				Name:        attachment.Filename,
				URL:         attachment.URL,
				ProxyURL:    attachment.ProxyURL,
				Size:        attachment.Size,
				ContentType: attachment.ContentType,
				Errors: []string{
					"Error creating attachment request.",
					"Got error: " + err.Error(),
				},
			})
			continue
		}

		resp, err := client.Do(req)

		if err != nil {
			attachments = append(attachments, AttachmentMetadata{
				ID:          attachment.ID,
				Name:        attachment.Filename,
				URL:         attachment.URL,
				ProxyURL:    attachment.ProxyURL,
				Size:        attachment.Size,
				ContentType: attachment.ContentType,
				Errors: []string{
					"Error downloading attachment.",
					"Got status code " + fmt.Sprintf("%d", resp.StatusCode),
				},
			})
			continue
		}

		if resp.StatusCode != http.StatusOK {
			logger.Warn("Attachment was not found", zap.String("url", url), zap.Int("status", resp.StatusCode))
			attachments = append(attachments, AttachmentMetadata{
				ID:          attachment.ID,
				Name:        attachment.Filename,
				URL:         attachment.URL,
				ProxyURL:    attachment.ProxyURL,
				Size:        attachment.Size,
				ContentType: attachment.ContentType,
				Errors: []string{
					"Attachment was not found.",
					"Got status code " + fmt.Sprintf("%d", resp.StatusCode),
				},
			})
			continue
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
				Quality: constraints.Create.JpegReencodeQuality,
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

func writeMsgpack(f *iblfile.AutoEncryptedFile_FullFile, section string, data any) error {
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
	// The ID of the server
	ServerID string

	// Constraints, this is auto-set by the task in jobserver and hence not configurable in this mode.
	Constraints *BackupConstraints

	// Backup options
	Options BackupCreateOpts
}

func (t *ServerBackupCreateTask) TaskFields() map[string]any {
	opts := t.Options
	opts.Encrypt = "" // Clear encryption key

	return map[string]any{
		"ServerID":    t.ServerID,
		"Constraints": t.Constraints,
		"Options":     opts,
	}
}

func (t *ServerBackupCreateTask) Expiry() *time.Duration {
	return nil
}

func (t *ServerBackupCreateTask) Resumable() bool {
	return false
}

func (t *ServerBackupCreateTask) Validate(state taskstate.TaskState) error {
	if t.ServerID == "" {
		return fmt.Errorf("server_id is required")
	}

	opMode := state.OperationMode()
	if opMode == "jobs" {
		t.Constraints = FreePlanBackupConstraints // TODO: Add other constraint types based on plans once we have them
	} else if opMode == "localjobs" {
		if t.Constraints == nil {
			return fmt.Errorf("constraints are required")
		}
	} else {
		return fmt.Errorf("invalid operation mode")
	}

	if t.Options.MaxMessages == 0 {
		t.Options.MaxMessages = t.Constraints.Create.TotalMaxMessages
	}

	if t.Options.PerChannel == 0 {
		t.Options.PerChannel = t.Constraints.Create.DefaultPerChannel
	}

	if t.Options.PerChannel < t.Constraints.Create.MinPerChannel {
		return fmt.Errorf("per_channel cannot be less than %d", t.Constraints.Create.MinPerChannel)
	}

	if t.Options.MaxMessages > t.Constraints.Create.TotalMaxMessages {
		return fmt.Errorf("max_messages cannot be greater than %d", t.Constraints.Create.TotalMaxMessages)
	}

	if t.Options.PerChannel > t.Options.MaxMessages {
		return fmt.Errorf("per_channel cannot be greater than max_messages")
	}

	if t.Options.BackupAttachments && !t.Options.BackupMessages {
		return fmt.Errorf("cannot backup attachments without messages")
	}

	if len(t.Options.SpecialAllocations) == 0 {
		t.Options.SpecialAllocations = make(map[string]int)
	}

	// Check current backup concurrency
	count, _ := concurrentBackupState.LoadOrStore(t.ServerID, 0)

	if count >= t.Constraints.MaxServerBackupTasks {
		return fmt.Errorf("you already have more than %d backup-related tasks in progress, please wait for it to finish", t.Constraints.MaxServerBackupTasks)
	}

	return nil
}

func (t *ServerBackupCreateTask) Exec(
	l *zap.Logger,
	state taskstate.TaskState,
	progstate taskstate.TaskProgressState,
) (*types.TaskOutput, error) {
	discord, botUser, _ := state.Discord()
	ctx := state.Context()

	// Check current backup concurrency
	count, _ := concurrentBackupState.LoadOrStore(t.ServerID, 0)

	if count >= t.Constraints.MaxServerBackupTasks {
		return nil, fmt.Errorf("you already have more than %d backup-related tasks in progress, please wait for it to finish", t.Constraints.MaxServerBackupTasks)
	}

	concurrentBackupState.Store(t.ServerID, count+1)

	// Decrement count when we're done
	defer func() {
		countNow, _ := concurrentBackupState.LoadOrStore(t.ServerID, 0)

		if countNow > 0 {
			concurrentBackupState.Store(t.ServerID, countNow-1)
		}
	}()

	t1 := time.Now()

	var aeSource iblfile.AutoEncryptor

	if t.Options.Encrypt == "" {
		aeSource = noencryption.NoEncryptionSource{}
	} else {
		aeSource = aes256.AES256Source{
			EncryptionKey: t.Options.Encrypt,
		}
	}

	t.Options.Encrypt = "" // Clear encryption key

	f := iblfile.NewAutoEncryptedFile_FullFile(aeSource)

	t2 := time.Now()

	l.Info("STATISTICS: newautoencryptedfile", zap.Float64("duration", t2.Sub(t1).Seconds()))

	err := writeMsgpack(f, "backup_opts", t.Options)

	if err != nil {
		return nil, fmt.Errorf("error writing backup options: %w", err)
	}

	// Fetch the bots member object in the guild
	l.Info("Fetching bots current state in server")
	m, err := discord.GuildMember(t.ServerID, botUser.ID, discordgo.WithContext(ctx))

	if err != nil {
		return nil, fmt.Errorf("error fetching bots member object: %w", err)
	}

	err = writeMsgpack(f, "dbg/bot", m) // Write bot member object to debug section

	if err != nil {
		return nil, fmt.Errorf("error writing bot member object: %w", err)
	}

	l.Info("Backing up server settings")

	// Fetch guild
	g, err := discord.Guild(t.ServerID, discordgo.WithContext(ctx))

	if err != nil {
		return nil, fmt.Errorf("error fetching guild: %w", err)
	}

	if len(g.Channels) == 0 {
		channels, err := discord.GuildChannels(t.ServerID, discordgo.WithContext(ctx))

		if err != nil {
			return nil, fmt.Errorf("error fetching channels: %w", err)
		}

		g.Channels = channels
	}

	if len(g.Roles) == 0 {
		l.Info("Backing up guild roles")

		// Fetch roles of guild
		roles, err := discord.GuildRoles(t.ServerID, discordgo.WithContext(ctx))

		if err != nil {
			return nil, fmt.Errorf("error fetching roles: %w", err)
		}

		g.Roles = roles
	}

	// With servers now backed up, get the base permissions now
	basePerms := utils.BasePermissions(g, m)

	// Write base permissions to debug section
	err = writeMsgpack(f, "dbg/basePerms", basePerms)

	if err != nil {
		return nil, fmt.Errorf("error writing base permissions: %w", err)
	}

	if len(g.Stickers) == 0 {
		l.Info("Backing up guild stickers")

		// Fetch stickers of guild
		stickers, err := discord.Request("GET", discordgo.EndpointGuildStickers(t.ServerID), nil, discordgo.WithContext(ctx))

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
	err = writeMsgpack(f, "core/guild", g)

	if err != nil {
		return nil, fmt.Errorf("error writing core backup: %w", err)
	}

	// Backup guild assets
	l.Info("Backing up guild assets", zap.Strings("assets", t.Options.BackupGuildAssets))

	for _, b := range t.Options.BackupGuildAssets {
		switch b {
		case "icon":
			if g.Icon == "" {
				continue
			}

			err := backupGuildAsset(state, t.Constraints, l, f, "guildIcon", discordgo.EndpointGuildIcon(g.ID, g.Icon))

			if err != nil {
				return nil, fmt.Errorf("error backing up guild icon: %w", err)
			}
		case "banner":
			if g.Banner == "" {
				continue
			}

			err := backupGuildAsset(state, t.Constraints, l, f, "guildBanner", discordgo.EndpointGuildBanner(g.ID, g.Banner))

			if err != nil {
				return nil, fmt.Errorf("error backing up guild banner: %w", err)
			}
		case "splash":
			if g.Splash == "" {
				continue
			}

			err := backupGuildAsset(state, t.Constraints, l, f, "guildSplash", discordgo.EndpointGuildSplash(g.ID, g.Splash))

			if err != nil {
				return nil, fmt.Errorf("error backing up guild splash: %w", err)
			}
		default:
			return nil, fmt.Errorf("unknown guild asset to backup: %s", b)
		}
	}

	// Backup messages
	if t.Options.BackupMessages {
		perChannelBackupMap, err := common.CreateChannelAllocations(
			basePerms,
			g,
			m,
			[]int64{discordgo.PermissionViewChannel},
			allowedChannelTypes,
			common.GetChannelsFromList(g, t.Options.Channels),
			t.Options.SpecialAllocations,
			t.Options.PerChannel,
			t.Options.MaxMessages,
		)

		if err != nil {
			return nil, fmt.Errorf("error creating channel allocations: %w", err)
		}

		l.Info("Created channel allocations", zap.Any("alloc", perChannelBackupMap), zap.Strings("botDisplayIgnore", []string{"alloc"}))

		err = writeMsgpack(f, "dbg/chanAlloc", perChannelBackupMap)

		if err != nil {
			return nil, fmt.Errorf("error writing channel allocations: %w", err)
		}

		// Backup messages
		err = common.ChannelAllocationStream(
			perChannelBackupMap,
			func(channelID string, allocation int) (collected int, err error) {
				l.Info("Backing up channel messages", zap.String("channelId", channelID))

				msgs, err := backupChannelMessages(state, t.Constraints, l, f, channelID, allocation, t.Options.BackupAttachments)

				// Write messages of this section regardless of error
				if len(msgs) > 0 {
					errMsg := writeMsgpack(f, "messages/"+channelID, msgs)

					if errMsg != nil {
						return len(msgs), fmt.Errorf("error writing messages: %w", err)
					}

					for _, msg := range msgs {
						if len(msg.attachments) > 0 {
							for id, buf := range msg.attachments {
								f.WriteSection(buf, "attachments/"+id)
							}
						}
					}
				}

				if err != nil {
					if t.Options.IgnoreMessageBackupErrors {
						l.Error("error backing up channel messages", zap.Error(err))
						return len(msgs), nil
					} else {
						return len(msgs), fmt.Errorf("error backing up channel messages: %w", err)
					}
				}

				return len(msgs), nil
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
			return nil, fmt.Errorf("error streaming channel allocations: %w", err)
		}
	}

	dbgInfo := state.DebugInfo()
	metadata := iblfile.Meta{
		CreatedAt: time.Now(),
		Protocol:  iblfile.Protocol,
		Type:      t.Constraints.FileType,
		ExtraMetadata: map[string]string{
			"OperationMode": state.OperationMode(),
			"GoVersion":     dbgInfo.GoVersion,
		},
	}

	ifmt, err := iblfile.GetFormat(t.Constraints.FileType)

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

	return &types.TaskOutput{
		Filename: fmt.Sprintf("antiraid-backup-%s.iblfile", time.Now().Format("2006-01-02-15-04-05")),
		Buffer:   &outputBuf,
	}, nil
}

func (t *ServerBackupCreateTask) CorrespondingBotCommand_Create() string {
	return "backups create"
}

func (t *ServerBackupCreateTask) CorrespondingBotCommand_View() string {
	return "backups list"
}

func (t *ServerBackupCreateTask) CorrespondingBotCommand_Download() string {
	return "backups list"
}

func (t *ServerBackupCreateTask) Name() string {
	return "guild_create_backup"
}

func (t *ServerBackupCreateTask) TaskFor() *types.TaskFor {
	return &types.TaskFor{
		ID:         t.ServerID,
		TargetType: types.TargetTypeServer,
	}
}

func (t *ServerBackupCreateTask) LocalPresets() *taskdef.PresetInfo {
	return &taskdef.PresetInfo{
		Runnable: true,
		Preset: &ServerBackupCreateTask{
			ServerID: "{{.Args.ServerID}}",
			Constraints: &BackupConstraints{
				Create: &BackupCreateConstraints{
					TotalMaxMessages:          1000,
					FileSizeWarningThreshold:  100000000,
					MinPerChannel:             50,
					DefaultPerChannel:         100,
					JpegReencodeQuality:       85,
					GuildAssetReencodeQuality: 85,
				},
				MaxServerBackupTasks: 1,
				FileType:             "backup.server",
			},
			Options: BackupCreateOpts{
				MaxMessages:               500,
				BackupMessages:            true,
				BackupAttachments:         true,
				BackupGuildAssets:         []string{"icon", "banner", "splash"},
				PerChannel:                100,
				RolloverLeftovers:         true,
				IgnoreMessageBackupErrors: false,
				Encrypt:                   "{{.Settings.BackupPassword}}",
			},
		},
		Comments: map[string]string{
			"Constraints.MaxServerBackupTasks":            "Only 1 backup task should be running at any given time locally",
			"Constraints.FileType":                        "The file type of the backup, you probably don't want to change this",
			"Constraints.Create.TotalMaxMessages":         "Since this is a local job, we can afford to be more generous",
			"Constraints.Create.FileSizeWarningThreshold": "100MB is used as default as we can be more generous with storage locally",
			"Options.BackupMessages":                      "This is a local job so backing up messages is likely faster and desired",
			"Options.BackupAttachments":                   "This is a local job so backing up attachments is likely faster and desired",
			"Options.BackupGuildAssets":                   "This is a local job so backing up guild assets is likely faster and desired",
			"Options.IgnoreMessageBackupErrors":           "We likely don't want errors ignored in local jobs",
		},
	}
}

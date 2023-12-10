package backups

import (
	"encoding/base64"
	"fmt"
	"io"
	"net/http"
	"slices"
	"splashtail/state"
	"splashtail/types"
	"splashtail/utils"
	"strings"
	"time"

	"github.com/bwmarrin/discordgo"
	"github.com/infinitybotlist/iblfile"
	"github.com/infinitybotlist/iblfile/autoencryptedencoders/aes256"
	"github.com/infinitybotlist/iblfile/autoencryptedencoders/noencryption"
	"github.com/jackc/pgx/v5"
	"github.com/vmihailenco/msgpack/v5"
	"go.uber.org/zap"
)

func getImageAsDataUri(l *zap.Logger, f *iblfile.AutoEncryptedFile, name, endpoint string, bo *BackupCreateOpts) (string, error) {
	if slices.Contains(bo.BackupGuildAssets, name) {
		l.Info("Fetching guild asset", zap.String("name", name))
		iconBytes, err := f.Get("assets/" + name)

		if err != nil {
			return "", fmt.Errorf("failed to get guild asset: %w", err)
		}

		return convertToDataUri("image/jpeg", iconBytes.Bytes.Bytes()), nil
	} else {
		// Try fetching still, it might work
		client := http.Client{
			Timeout: 10 * time.Second,
		}

		resp, err := client.Get(endpoint)

		if err != nil {
			return "", fmt.Errorf("error fetching guild asset: %w", err)
		}

		if resp.StatusCode != http.StatusOK {
			return "", fmt.Errorf("error fetching guild asset: %w", fmt.Errorf("status code %d", resp.StatusCode))
		}

		mime := resp.Header.Get("Content-Type")

		if mime == "" {
			return "", fmt.Errorf("error fetching guild asset: %w", fmt.Errorf("no mime type"))
		}

		defer resp.Body.Close()

		body, err := io.ReadAll(resp.Body)

		if err != nil {
			return "", fmt.Errorf("error reading guild icon: %w", err)
		}

		return convertToDataUri(mime, body), nil
	}
}

func readMsgpackSection[T any](f *iblfile.AutoEncryptedFile, name string) (*T, error) {
	section, err := f.Get(name)

	if err != nil {
		return nil, fmt.Errorf("failed to get section %s: %w", name, err)
	}

	dec := msgpack.NewDecoder(section.Bytes)
	dec.UseInternedStrings(true)
	dec.SetCustomStructTag("json")

	var outp T

	err = dec.Decode(&outp)

	if err != nil {
		return nil, fmt.Errorf("failed to decode section %s: %w", name, err)
	}

	return &outp, nil
}

func convertToDataUri(mimeType string, data []byte) string {
	// Base64 encode
	b64enc := base64.StdEncoding.EncodeToString(data)

	return fmt.Sprintf("data:%s;base64,%s", mimeType, b64enc)
}

// A task to restore a backup of a server
type ServerBackupRestoreTask struct {
	// The ID of the server
	ServerID string `json:"server_id"`

	// Backup options
	Options BackupRestoreOpts `json:"options"`

	valid bool `json:"-"`
}

// Validate validates the task and sets up state if needed
func (t *ServerBackupRestoreTask) Validate() error {
	if t.ServerID == "" {
		return fmt.Errorf("server_id is required")
	}

	if t.Options.BackupSource == "" {
		return fmt.Errorf("backup_source is required")
	}

	if !strings.HasPrefix(t.Options.BackupSource, "https://") {
		return fmt.Errorf("backup_source must be a valid URL")
	}

	switch t.Options.ChannelRestoreMode {
	case ChannelRestoreModeFull:
	case ChannelRestoreModeDiff:
	case ChannelRestoreModeIgnoreExisting:
	default:
		if string(t.Options.ChannelRestoreMode) == "" {
			t.Options.ChannelRestoreMode = ChannelRestoreModeFull
		} else {
			return fmt.Errorf("invalid channel_restore_mode")
		}
	}

	switch t.Options.RoleRestoreMode {
	case RoleRestoreModeFull:
	default:
		if string(t.Options.RoleRestoreMode) == "" {
			t.Options.RoleRestoreMode = RoleRestoreModeFull
		} else {
			return fmt.Errorf("invalid role_restore_mode")
		}
	}

	t.valid = true

	return nil
}

func (t *ServerBackupRestoreTask) Exec(l *zap.Logger, tx pgx.Tx, tcr *types.TaskCreateResponse) (*types.TaskOutput, error) {
	// Download backup
	l.Info("Downloading backup", zap.String("url", t.Options.BackupSource))
	client := http.Client{
		Timeout: 10 * time.Second,
	}

	resp, err := client.Get(t.Options.BackupSource)

	if err != nil {
		return nil, fmt.Errorf("failed to download backup: %w", err)
	}

	// Limit body size to 100mb
	if resp.ContentLength > restoreMaxBodySize {
		return nil, fmt.Errorf("backup too large, expected less than %d bytes, got %d bytes", restoreMaxBodySize, resp.ContentLength)
	}

	resp.Body = http.MaxBytesReader(nil, resp.Body, restoreMaxBodySize)

	defer resp.Body.Close()

	l.Info("Parsing backup", zap.String("url", t.Options.BackupSource))

	// Parse backup
	t1 := time.Now()

	var aeSource iblfile.AEDataSource

	if t.Options.Decrypt == "" {
		aeSource = noencryption.NoEncryptionSource{}
	} else {
		aeSource = aes256.AES256Source{
			EncryptionKey: t.Options.Decrypt,
		}
	}

	t.Options.Decrypt = "" // Clear encryption key

	f, err := iblfile.OpenAutoEncryptedFile(resp.Body, aeSource)

	if err != nil {
		return nil, fmt.Errorf("error creating file: %w", err)
	}

	t2 := time.Now()

	l.Info("STATISTICS: openautoencryptedfile", zap.Float64("duration", t2.Sub(t1).Seconds()))

	t1 = time.Now()

	sections := f.Source.Sections()

	keys := make([]string, 0, len(sections))
	for name := range sections {
		keys = append(keys, name)
	}

	t2 = time.Now()

	l.Info("STATISTICS: keys", zap.Float64("duration", t2.Sub(t1).Seconds()), zap.Strings("keys", keys))

	// Step 0. Fetch backup_opts
	t1 = time.Now()

	bo, err := readMsgpackSection[BackupCreateOpts](f, "backup_opts")

	if err != nil {
		return nil, fmt.Errorf("failed to get backup_opts: %w", err)
	}

	t2 = time.Now()

	l.Info("STATISTICS: backupopts", zap.Float64("duration", t2.Sub(t1).Seconds()))

	// Fetch the bots member object in the guild
	l.Info("Fetching bots current state in server")
	m, err := state.Discord.GuildMember(t.ServerID, state.BotUser.ID)

	if err != nil {
		return nil, fmt.Errorf("error fetching bots member object: %w", err)
	}

	l.Info("Fetching guild object")
	tgtGuild, err := state.Discord.Guild(t.ServerID)

	if err != nil {
		return nil, fmt.Errorf("error fetching guild: %w", err)
	}

	basePerms := utils.BasePermissions(tgtGuild, m)

	if basePerms&discordgo.PermissionManageChannels != discordgo.PermissionManageChannels && basePerms&discordgo.PermissionAdministrator != discordgo.PermissionAdministrator {
		return nil, fmt.Errorf("bot does not have 'Manage Channels' permissions")
	}

	if basePerms&discordgo.PermissionManageRoles != discordgo.PermissionManageRoles && basePerms&discordgo.PermissionAdministrator != discordgo.PermissionAdministrator {
		return nil, fmt.Errorf("bot does not have 'Manage Roles' permissions")
	}

	// Get highest role
	var tgtBotGuildHighestRole *discordgo.Role

	for _, role := range tgtGuild.Roles {
		if !slices.Contains(m.Roles, role.ID) {
			continue
		}

		if tgtBotGuildHighestRole == nil {
			tgtBotGuildHighestRole = role
			continue
		}

		if role.Position > tgtBotGuildHighestRole.Position {
			tgtBotGuildHighestRole = role
		}

		if role.Position == tgtBotGuildHighestRole.Position {
			// Check ID
			if role.ID > tgtBotGuildHighestRole.ID {
				tgtBotGuildHighestRole = role
			}
		}
	}

	if tgtBotGuildHighestRole == nil {
		return nil, fmt.Errorf("bot does not have any roles")
	}

	if tgtBotGuildHighestRole.Position == 0 {
		return nil, fmt.Errorf("bot role isnt high enough")
	}

	l.Info("Got bots highest role", zap.String("role_id", tgtBotGuildHighestRole.ID))

	// Step 1. Fetch guild data
	t1 = time.Now()

	srcGuild, err := readMsgpackSection[discordgo.Guild](f, "core/guild")

	if err != nil {
		return nil, fmt.Errorf("failed to get core data: %w", err)
	}

	if srcGuild.ID == "" {
		return nil, fmt.Errorf("guild data is invalid [id is empty], likely an internal decoding error")
	}

	t2 = time.Now()

	l.Info("STATISTICS: guildread", zap.Float64("duration", t2.Sub(t1).Seconds()))

	t1 = time.Now()

	// Edit basic guild guild. Note that settings related to ID's are changed later if needed
	// Notes:
	//
	// - Region is not restored
	// - Owner is not restored
	gp := &discordgo.GuildParams{
		Name:                        srcGuild.Name,
		Description:                 srcGuild.Description,
		VerificationLevel:           &srcGuild.VerificationLevel,
		DefaultMessageNotifications: int(srcGuild.DefaultMessageNotifications),
		ExplicitContentFilter:       int(srcGuild.ExplicitContentFilter),
		AfkTimeout:                  srcGuild.AfkTimeout,
	}

	// Icons
	canUseIcon := slices.Contains(srcGuild.Features, "ANIMATED_ICON") || !strings.HasPrefix(srcGuild.Icon, "a_")
	canUseBanner := (slices.Contains(srcGuild.Features, "BANNER") && !strings.HasPrefix(srcGuild.Banner, "a_")) || slices.Contains(srcGuild.Features, "ANIMATED_BANNER")
	canUseSplash := slices.Contains(srcGuild.Features, "INVITE_SPLASH") && !strings.HasPrefix(srcGuild.Splash, "a_")

	if srcGuild.Icon != "" {
		if !canUseIcon {
			l.Warn("Not restoring animated icon on unsupported server", zap.String("guild_id", srcGuild.ID))
		} else {
			icon, err := getImageAsDataUri(l, f, "guildIcon", srcGuild.IconURL("1024"), bo)

			if err != nil {
				return nil, fmt.Errorf("failed to get icon: %w", err)
			}

			gp.Icon = icon
		}
	}

	if srcGuild.Banner != "" {
		if !canUseBanner {
			l.Warn("Not restoring banner on unsupported server", zap.String("guild_id", srcGuild.ID))
		} else {
			banner, err := getImageAsDataUri(l, f, "guildBanner", srcGuild.BannerURL("1024"), bo)

			if err != nil {
				return nil, fmt.Errorf("failed to get banner: %w", err)
			}

			gp.Banner = banner
		}
	}

	if srcGuild.Splash != "" {
		if !canUseSplash {
			l.Warn("Not restoring splash on unsupported server", zap.String("guild_id", srcGuild.ID))
		} else {
			splash, err := getImageAsDataUri(l, f, "guildSplash", discordgo.EndpointGuildSplash(srcGuild.ID, srcGuild.Splash), bo)

			if err != nil {
				return nil, fmt.Errorf("failed to get splash: %w", err)
			}

			gp.Splash = splash
		}
	}

	if slices.Contains(tgtGuild.Features, discordgo.GuildFeatureCommunity) {
		// Also disable community for now
		if basePerms&discordgo.PermissionAdministrator != discordgo.PermissionAdministrator {
			return nil, fmt.Errorf("**server restore cannot continue unless the bot is given administrator to disable community feature or community server status is disabled**")
		}

		gp.Features = []discordgo.GuildFeature{}

		for _, feature := range tgtGuild.Features {
			if feature == discordgo.GuildFeatureCommunity {
				continue
			}

			gp.Features = append(gp.Features, feature)
		}
	}

	_, err = state.Discord.GuildEdit(t.ServerID, gp)

	if err != nil {
		return nil, fmt.Errorf("failed to edit guild: %w", err)
	}

	t2 = time.Now()

	l.Info("STATISTICS: guildedit", zap.Float64("duration", t2.Sub(t1).Seconds()))

	// Step 2. Restore roles
	t1 = time.Now()

	// Map of backed up role id to restored role id
	var restoredRolesMap = make(map[string]string)

	switch t.Options.RoleRestoreMode {
	case RoleRestoreModeFull:
		for _, r := range tgtGuild.Roles {
			if slices.Contains(t.Options.ProtectedRoles, r.ID) {
				continue
			}

			if r.Managed {
				continue
			}

			if r.ID == tgtGuild.ID {
				continue // @everyone
			}

			if r.ID == tgtBotGuildHighestRole.ID {
				continue // Bot role
			}

			if r.Position >= tgtBotGuildHighestRole.Position {
				continue // Higher than bot role
			}

			l.Info("Deleting role", zap.String("name", r.Name), zap.Int("position", r.Position), zap.String("id", r.ID))

			err := state.Discord.GuildRoleDelete(t.ServerID, r.ID)

			if err != nil {
				return nil, fmt.Errorf("failed to delete role: %w with position of %d", err, r.Position)
			}

			time.Sleep(1 * time.Second)
		}
	}

	// Sort in descending order
	slices.SortFunc(srcGuild.Roles, func(a, b *discordgo.Role) int {
		if a.Position == b.Position {
			if a.ID == b.ID {
				return 0
			} else {
				if a.ID > b.ID {
					return -1
				} else {
					return 1
				}
			}
		}

		if a.Position > b.Position {
			return -1
		} else {
			return 1
		}
	})

	for i := range srcGuild.Roles {
		if _, ok := restoredRolesMap[srcGuild.Roles[i].ID]; ok {
			continue
		}

		if srcGuild.Roles[i].Position >= tgtBotGuildHighestRole.Position {
			srcGuild.Roles[i].Position = tgtBotGuildHighestRole.Position - 1
		}

		if slices.Contains(t.Options.ProtectedRoles, srcGuild.Roles[i].ID) {
			continue
		}

		if srcGuild.Roles[i].Managed {
			continue
		}

		if srcGuild.Roles[i].ID == srcGuild.ID {
			continue // @everyone
		}

		l.Info("Creating role", zap.String("name", srcGuild.Roles[i].Name), zap.Int("position", srcGuild.Roles[i].Position), zap.String("id", srcGuild.Roles[i].ID))

		newRole, err := state.Discord.GuildRoleCreate(t.ServerID, &discordgo.RoleParams{
			Name: srcGuild.Roles[i].Name,
			Color: func() *int {
				if srcGuild.Roles[i].Color == 0 {
					return nil
				}

				return &srcGuild.Roles[i].Color
			}(),
			Hoist:       utils.Pointer(srcGuild.Roles[i].Hoist),
			Permissions: utils.Pointer(srcGuild.Roles[i].Permissions),
			Mentionable: utils.Pointer(srcGuild.Roles[i].Mentionable),
		})

		if err != nil {
			return nil, fmt.Errorf("failed to create role: %w", err)
		}

		restoredRolesMap[srcGuild.Roles[i].ID] = newRole.ID
	}

	t2 = time.Now()

	l.Info("STATISTICS: roles", zap.Float64("duration", t2.Sub(t1).Seconds()))

	// Step 3. Next restore channels

	// // Map of backed up channel id to restored channel id
	var restoredChannelsMap = make(map[string]string)

	var backupChannelMap = make(map[string]*discordgo.Channel) // Map of backed up channel id to channel object

	var currentChannelMap = make(map[string]*discordgo.Channel) // Map of current channel id to channel object
	for _, channel := range tgtGuild.Channels {
		currentChannelMap[channel.ID] = channel
	}

	for _, channel := range srcGuild.Channels {
		backupChannelMap[channel.ID] = channel
	}

	switch t.Options.ChannelRestoreMode {
	case ChannelRestoreModeFull:
		for _, c := range tgtGuild.Channels {
			if slices.Contains(t.Options.ProtectedChannels, c.ID) {
				continue
			}

			bp := utils.MemberChannelPerms(basePerms, tgtGuild, m, c)

			if bp&discordgo.PermissionManageChannels != discordgo.PermissionManageChannels {
				l.Warn("Not removing channel due to lack of 'Manage Channels' permissions", zap.String("channel_id", c.ID))
				continue
			}

			_, err := state.Discord.ChannelDelete(c.ID)

			if err != nil {
				return nil, fmt.Errorf("failed to delete channel: %w", err)
			}

			time.Sleep(1 * time.Second)
		}
	case ChannelRestoreModeDiff:
		// Remove channels that are not in the backup
		for _, channel := range tgtGuild.Channels {
			if slices.Contains(t.Options.ProtectedChannels, channel.ID) {
				continue
			}

			bp := utils.MemberChannelPerms(basePerms, tgtGuild, m, channel)

			if bp&discordgo.PermissionManageChannels != discordgo.PermissionManageChannels {
				continue
			}

			if c, ok := backupChannelMap[channel.ID]; !ok {
				_, err := state.Discord.ChannelDelete(channel.ID)

				if err != nil {
					return nil, fmt.Errorf("failed to delete channel: %w", err)
				}

				time.Sleep(1 * time.Second)
				continue
			} else {
				// Check if type is different, this should never happen but just in case
				if c.Type != channel.Type || !slices.Contains([]discordgo.ChannelType{
					discordgo.ChannelTypeGuildText,
					discordgo.ChannelTypeGuildVoice,
					discordgo.ChannelTypeGuildNews,
				}, c.Type) {
					_, err := state.Discord.ChannelDelete(channel.ID)

					if err != nil {
						return nil, fmt.Errorf("failed to delete channel: %w", err)
					}

					time.Sleep(1 * time.Second)
					continue
				}

				// If channel has a parent id, make sure it exists [for now, this may be improved later]
				if c.ParentID != "" {
					if _, ok := backupChannelMap[c.ParentID]; !ok {
						// Parent does not exist, fallback to channel delete
						_, err := state.Discord.ChannelDelete(channel.ID)

						if err != nil {
							return nil, fmt.Errorf("failed to delete channel: %w", err)
						}

						time.Sleep(1 * time.Second)
						continue
					}
				}

				// Edit the channel
				cp := &discordgo.ChannelEdit{
					Name:                 channel.Name,
					Topic:                channel.Topic,
					NSFW:                 utils.Pointer(channel.NSFW),
					Position:             utils.Pointer(channel.Position),
					Bitrate:              channel.Bitrate,
					PermissionOverwrites: channel.PermissionOverwrites,
					ParentID:             channel.ParentID,
					Flags:                &channel.Flags,
				}

				updated, err := state.Discord.ChannelEdit(channel.ID, cp)

				if err != nil {
					return nil, fmt.Errorf("failed to edit channel, consider full backup mode: %w", err)
				}

				// Update channel map
				backupChannelMap[channel.ID] = updated

				restoredChannelsMap[c.ID] = channel.ID
			}
		}
	case ChannelRestoreModeIgnoreExisting:
		for _, c := range tgtGuild.Channels {
			if slices.Contains(t.Options.ProtectedChannels, c.ID) {
				continue
			}

			// Ignore if channel exists
			if _, ok := backupChannelMap[c.ID]; ok {
				continue
			}

			bp := utils.MemberChannelPerms(basePerms, tgtGuild, m, c)

			if bp&discordgo.PermissionManageChannels != discordgo.PermissionManageChannels {
				l.Warn("Not removing channel due to lack of 'Manage Channels' permissions", zap.String("channel_id", c.ID))
				continue
			}

			_, err := state.Discord.ChannelDelete(c.ID)

			if err != nil {
				return nil, fmt.Errorf("failed to delete channel: %w", err)
			}

			time.Sleep(1 * time.Second)
		}
	}

	createChannel := func(channel *discordgo.Channel) (*discordgo.Channel, error) {
		return state.Discord.GuildChannelCreateComplex(t.ServerID, discordgo.GuildChannelCreateData{
			Name:                 channel.Name,
			Type:                 channel.Type,
			Topic:                channel.Topic,
			Bitrate:              channel.Bitrate,
			UserLimit:            channel.UserLimit,
			RateLimitPerUser:     channel.RateLimitPerUser,
			Position:             channel.Position,
			PermissionOverwrites: channel.PermissionOverwrites,
			ParentID:             channel.ParentID,
			NSFW:                 channel.NSFW,
		})
	}

	for _, newChan := range srcGuild.Channels {
		if _, ok := restoredChannelsMap[newChan.ID]; ok {
			continue
		}

		// Create corresponding category if needed
		if newChan.ParentID != "" {
			if slices.Contains([]discordgo.ChannelType{
				discordgo.ChannelTypeGuildText,
				discordgo.ChannelTypeGuildVoice,
				discordgo.ChannelTypeGuildNews,
				discordgo.ChannelTypeGuildForum,
			}, newChan.Type) {
				if _, ok := restoredChannelsMap[newChan.ParentID]; !ok {
					// Create parent
					bmc, ok := backupChannelMap[newChan.ParentID]

					if !ok {
						l.Warn("Parent channel does not exist, skipping", zap.String("channel_id", newChan.ParentID))
						newChan.ParentID = ""
					} else {
						parent, err := createChannel(backupChannelMap[newChan.ParentID])

						if err != nil {
							return nil, fmt.Errorf("failed to create parent channel: %w", err)
						}

						restoredChannelsMap[bmc.ID] = parent.ID
					}
				}
			}
		}

		nc, err := createChannel(newChan)

		if err != nil {
			return nil, fmt.Errorf("failed to create channel: %w", err)
		}

		restoredChannelsMap[newChan.ID] = nc.ID
	}

	gp = &discordgo.GuildParams{}

	// Features, only COMMUNITY is editable IIRC
	var features []discordgo.GuildFeature = tgtGuild.Features
	if basePerms&discordgo.PermissionAdministrator == discordgo.PermissionAdministrator {
		if slices.Contains(srcGuild.Features, discordgo.GuildFeatureCommunity) && !slices.Contains(features, discordgo.GuildFeatureCommunity) {
			var rulesChannelId string
			var publicUpdatesChannelId string

			for srcChannel, restoredChannel := range restoredChannelsMap {
				if srcChannel == srcGuild.RulesChannelID {
					rulesChannelId = restoredChannel
				}

				if srcChannel == srcGuild.PublicUpdatesChannelID {
					publicUpdatesChannelId = restoredChannel
				}

				if rulesChannelId != "" && publicUpdatesChannelId != "" {
					break
				}
			}

			gp.RulesChannelID = rulesChannelId
			gp.PublicUpdatesChannelID = publicUpdatesChannelId

			if gp.RulesChannelID != "" && gp.PublicUpdatesChannelID != "" {
				features = append(features, discordgo.GuildFeatureCommunity)
				if tgtGuild.VerificationLevel == discordgo.VerificationLevelNone || tgtGuild.VerificationLevel == discordgo.VerificationLevelLow {
					medium := discordgo.VerificationLevelMedium
					gp.VerificationLevel = &medium
				}
			}
		}
	} else {
		l.Warn("Not admin, certain features may not be editable", zap.Int64("basePerms", basePerms))
	}

	gp.Features = features

	_, err = state.Discord.GuildEdit(t.ServerID, gp)

	if err != nil {
		return nil, fmt.Errorf("failed to edit guild: %w", err)
	}

	l.Info("Successfully restored guild")

	return nil, nil
}

func (t *ServerBackupRestoreTask) Info() *types.TaskInfo {
	return &types.TaskInfo{
		Name: "restore_backup",
		TaskFor: &types.TaskFor{
			ID:         t.ServerID,
			TargetType: types.TargetTypeServer,
		},
		TaskFields: t,
		Valid:      t.valid,
	}
}

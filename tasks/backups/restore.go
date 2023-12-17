package backups

import (
	"encoding/base64"
	"fmt"
	"io"
	"net/http"
	"slices"
	"strings"
	"time"

	"github.com/anti-raid/splashtail/state"
	"github.com/anti-raid/splashtail/types"
	"github.com/anti-raid/splashtail/utils"

	"github.com/bwmarrin/discordgo"
	"github.com/infinitybotlist/iblfile"
	"github.com/infinitybotlist/iblfile/autoencryptedencoders/aes256"
	"github.com/infinitybotlist/iblfile/autoencryptedencoders/noencryption"
	"github.com/vmihailenco/msgpack/v5"
	"go.uber.org/zap"
)

func getImageAsDataUri(constraints *BackupConstraints, l *zap.Logger, f *iblfile.AutoEncryptedFile, name, endpoint string, bo *BackupCreateOpts) (string, error) {
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
			Timeout: time.Duration(constraints.Restore.HttpClientTimeout),
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
	ServerID string

	// Constraints, this is auto-set by the task in jobserver and hence not configurable in this mode.
	Constraints *BackupConstraints

	// Backup options
	Options BackupRestoreOpts

	valid bool
}

// Validate validates the task and sets up state if needed
func (t *ServerBackupRestoreTask) Validate() error {
	if t.ServerID == "" {
		return fmt.Errorf("server_id is required")
	}

	if t.Constraints == nil || state.CurrentOperationMode == "jobs" {
		t.Constraints = FreePlanBackupConstraints // TODO: Add other constraint types based on plans once we have them
	}

	if t.Options.BackupSource == "" {
		return fmt.Errorf("backup_source is required")
	}

	if state.CurrentOperationMode == "jobs" {
		if !strings.HasPrefix(t.Options.BackupSource, "https://") {
			return fmt.Errorf("backup_source must be a valid URL")
		}
	} else if state.CurrentOperationMode == "localjobs" {
		if !strings.HasPrefix(t.Options.BackupSource, "file://") && !strings.HasPrefix(t.Options.BackupSource, "http://") && !strings.HasPrefix(t.Options.BackupSource, "https://") {
			return fmt.Errorf("backup_source must be a valid URL or file path")
		}
	} else {
		return fmt.Errorf("invalid operation mode")
	}

	switch t.Options.ChannelRestoreMode {
	case ChannelRestoreModeFull:
	case ChannelRestoreModeDiff:
		return fmt.Errorf("channel_restore_mode 'diff' is not yet supported due to the complexity of the approach")
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

func (t *ServerBackupRestoreTask) Exec(l *zap.Logger, tcr *types.TaskCreateResponse) (*types.TaskOutput, error) {
	// Check current backup concurrency
	count, _ := concurrentBackupState.LoadOrStore(t.ServerID, 0)

	if count >= t.Constraints.MaxServerBackupTasks {
		return nil, fmt.Errorf("you already have more than %d backup-related task in progress, please wait for it to finish", t.Constraints.MaxServerBackupTasks)
	}

	concurrentBackupState.Store(t.ServerID, count+1)

	// Decrement count when we're done
	defer func() {
		countNow, _ := concurrentBackupState.LoadOrStore(t.ServerID, 0)

		if countNow > 0 {
			concurrentBackupState.Store(t.ServerID, countNow-1)
		}
	}()

	// Download backup
	l.Info("Downloading backup", zap.String("url", t.Options.BackupSource))
	client := http.Client{
		Timeout: time.Duration(t.Constraints.Restore.HttpClientTimeout),
	}

	resp, err := client.Get(t.Options.BackupSource)

	if err != nil {
		return nil, fmt.Errorf("failed to download backup: %w", err)
	}

	// Limit body size to 100mb
	if resp.ContentLength > t.Constraints.Restore.MaxBodySize {
		return nil, fmt.Errorf("backup too large, expected less than %d bytes, got %d bytes", t.Constraints.Restore.MaxBodySize, resp.ContentLength)
	}

	resp.Body = http.MaxBytesReader(nil, resp.Body, t.Constraints.Restore.MaxBodySize)

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

	if len(tgtGuild.Roles) == 0 {
		roles, err := state.Discord.GuildRoles(t.ServerID)

		if err != nil {
			return nil, fmt.Errorf("error fetching roles: %w", err)
		}

		tgtGuild.Roles = roles
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

	if tgtBotGuildHighestRole.Position <= 0 {
		return nil, fmt.Errorf("bot role isnt high enough")
	}

	// Fetch channels of guild
	channels, err := state.Discord.GuildChannels(t.ServerID)

	if err != nil {
		return nil, fmt.Errorf("error fetching channels: %w", err)
	}

	tgtGuild.Channels = channels

	l.Info("Got bots highest role", zap.String("role_id", tgtBotGuildHighestRole.ID), zap.Int("role_position", tgtBotGuildHighestRole.Position))

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

	var srcIsCommunity = slices.Contains(srcGuild.Features, discordgo.GuildFeatureCommunity)
	var tgtIsCommunity = slices.Contains(tgtGuild.Features, discordgo.GuildFeatureCommunity)

	if srcIsCommunity && !tgtIsCommunity {
		return nil, fmt.Errorf("cannot restore community server to non-community server")
	}

	// Edit basic guild guild. Note that settings related to ID's are changed later if needed
	// Notes:
	//
	// - Region is not restored
	// - Owner is not restored
	gp := &discordgo.GuildParams{
		Name:                        srcGuild.Name,
		Description:                 srcGuild.Description,
		DefaultMessageNotifications: int(srcGuild.DefaultMessageNotifications),
		AfkTimeout:                  srcGuild.AfkTimeout,
	}

	// If the src server is a community server or the target isn't, we can restore these settings
	if srcIsCommunity || !tgtIsCommunity {
		gp.ExplicitContentFilter = int(srcGuild.ExplicitContentFilter)
		gp.VerificationLevel = &srcGuild.VerificationLevel
	}

	// Icons
	canUseIcon := slices.Contains(srcGuild.Features, "ANIMATED_ICON") || !strings.HasPrefix(srcGuild.Icon, "a_")
	canUseBanner := (slices.Contains(srcGuild.Features, "BANNER") && !strings.HasPrefix(srcGuild.Banner, "a_")) || slices.Contains(srcGuild.Features, "ANIMATED_BANNER")
	canUseSplash := slices.Contains(srcGuild.Features, "INVITE_SPLASH") && !strings.HasPrefix(srcGuild.Splash, "a_")

	if srcGuild.Icon != "" {
		if !canUseIcon {
			l.Warn("Not restoring animated icon on unsupported server", zap.String("guild_id", srcGuild.ID))
		} else {
			icon, err := getImageAsDataUri(t.Constraints, l, f, "guildIcon", srcGuild.IconURL("1024"), bo)

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
			banner, err := getImageAsDataUri(t.Constraints, l, f, "guildBanner", srcGuild.BannerURL("1024"), bo)

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
			splash, err := getImageAsDataUri(t.Constraints, l, f, "guildSplash", discordgo.EndpointGuildSplash(srcGuild.ID, srcGuild.Splash), bo)

			if err != nil {
				return nil, fmt.Errorf("failed to get splash: %w", err)
			}

			gp.Splash = splash
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

			if r.Position > tgtBotGuildHighestRole.Position {
				continue // Higher than bot role
			}

			if r.Position == tgtBotGuildHighestRole.Position && tgtBotGuildHighestRole.ID > r.ID {
				continue
			}

			l.Info("Deleting role", zap.String("name", r.Name), zap.Int("position", r.Position), zap.String("id", r.ID))

			err := state.Discord.GuildRoleDelete(t.ServerID, r.ID)

			if err != nil {
				return nil, fmt.Errorf("failed to delete role: %w with position of %d", err, r.Position)
			}

			time.Sleep(time.Duration(t.Constraints.Restore.RoleDeleteSleep))
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
			Hoist:       &srcGuild.Roles[i].Hoist,
			Permissions: &srcGuild.Roles[i].Permissions,
			Mentionable: &srcGuild.Roles[i].Mentionable,
		}, discordgo.WithRetryOnRatelimit(true))

		if err != nil {
			return nil, fmt.Errorf("failed to create role: %w", err)
		}

		restoredRolesMap[srcGuild.Roles[i].ID] = newRole.ID

		time.Sleep(time.Duration(t.Constraints.Restore.RoleCreateSleep))
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
		for i := range tgtGuild.Channels {
			if slices.Contains(t.Options.ProtectedChannels, tgtGuild.Channels[i].ID) {
				continue
			}

			if tgtGuild.Channels[i].ID == tgtGuild.RulesChannelID || tgtGuild.Channels[i].ID == tgtGuild.PublicUpdatesChannelID {
				continue
			}

			if tgtGuild.Channels[i].ID == tgtGuild.PublicUpdatesChannelID {
				continue
			}

			bp := utils.MemberChannelPerms(basePerms, tgtGuild, m, tgtGuild.Channels[i])

			if bp&discordgo.PermissionManageChannels != discordgo.PermissionManageChannels && bp&discordgo.PermissionAdministrator != discordgo.PermissionAdministrator {
				l.Warn("Not removing channel due to lack of 'Manage Channels' permissions", zap.String("channel_id", tgtGuild.Channels[i].ID))
				continue
			}

			l.Info("Deleting channel", zap.String("name", tgtGuild.Channels[i].Name), zap.Int("position", tgtGuild.Channels[i].Position), zap.String("id", tgtGuild.Channels[i].ID))

			_, err := state.Discord.ChannelDelete(tgtGuild.Channels[i].ID)

			if err != nil {
				return nil, fmt.Errorf("failed to delete channel: %w", err)
			}

			time.Sleep(time.Duration(t.Constraints.Restore.ChannelDeleteSleep))
		}
	case ChannelRestoreModeIgnoreExisting:
	default:
		return nil, fmt.Errorf("invalid channel_restore_mode")
	}

	// Internal function. Given a channel, this fixes permission overwrites and then creates the channel from the old source channel
	var createChannel = func(channel *discordgo.Channel) (*discordgo.Channel, error) {
		l.Info("Creating channel", zap.String("name", channel.Name), zap.Int("position", channel.Position), zap.String("srcId", channel.ID), zap.String("parent_id", channel.ParentID), zap.Any("type", channel.Type))

		// fix permission overwrites
		var permOverwrites = []*discordgo.PermissionOverwrite{}

		for _, overwrite := range channel.PermissionOverwrites {
			if overwrite.Type == discordgo.PermissionOverwriteTypeRole {
				if rcid, ok := restoredRolesMap[overwrite.ID]; ok {
					permOverwrites = append(permOverwrites, &discordgo.PermissionOverwrite{
						ID:    rcid,
						Type:  overwrite.Type,
						Allow: overwrite.Allow,
						Deny:  overwrite.Deny,
					})
					continue
				}

				if overwrite.ID == srcGuild.ID {
					permOverwrites = append(permOverwrites, &discordgo.PermissionOverwrite{
						ID:    tgtGuild.ID,
						Type:  overwrite.Type,
						Allow: overwrite.Allow,
						Deny:  overwrite.Deny,
					})
				}
			} else {
				permOverwrites = append(permOverwrites, overwrite)
			}
		}

		c, err := state.Discord.GuildChannelCreateComplex(t.ServerID, discordgo.GuildChannelCreateData{
			Name:                 channel.Name,
			Type:                 channel.Type,
			Topic:                channel.Topic,
			Bitrate:              channel.Bitrate,
			UserLimit:            channel.UserLimit,
			RateLimitPerUser:     channel.RateLimitPerUser,
			Position:             channel.Position,
			PermissionOverwrites: permOverwrites,
			ParentID:             channel.ParentID,
			NSFW:                 channel.NSFW,
		})

		if err != nil {
			return nil, fmt.Errorf("failed to create channel: %w", err)
		}

		time.Sleep(time.Duration(t.Constraints.Restore.ChannelCreateSleep))

		return c, nil
	}

	// First restore categories
	for i := range srcGuild.Channels {
		if srcGuild.Channels[i].Type != discordgo.ChannelTypeGuildCategory {
			continue
		}

		if _, ok := restoredChannelsMap[srcGuild.Channels[i].ID]; ok {
			continue
		}

		nc, err := createChannel(srcGuild.Channels[i])

		if err != nil {
			return nil, fmt.Errorf("failed to create channel: %w", err)
		}

		restoredChannelsMap[srcGuild.Channels[i].ID] = nc.ID
	}

	for i := range srcGuild.Channels {
		if _, ok := restoredChannelsMap[srcGuild.Channels[i].ID]; ok {
			continue
		}

		// Create corresponding category if needed
		if srcGuild.Channels[i].ParentID != "" {
			if rcid, ok := restoredChannelsMap[srcGuild.Channels[i].ParentID]; ok {
				srcGuild.Channels[i].ParentID = rcid
			} else {
				if t.Options.IgnoreRestoreErrors {
					l.Warn("Parent channel does not exist, skipping", zap.String("channel_id", srcGuild.Channels[i].ParentID))
					srcGuild.Channels[i].ParentID = ""
				} else {
					return nil, fmt.Errorf("parent channel does not exist")
				}
			}
		}

		nc, err := createChannel(srcGuild.Channels[i])

		if err != nil {
			return nil, fmt.Errorf("failed to create channel: %w", err)
		}

		restoredChannelsMap[srcGuild.Channels[i].ID] = nc.ID
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

	l.Info("Waiting 5 seconds to avoid API issues")

	time.Sleep(5 * time.Second)

	if bo.BackupMessages {
		for backedUpChannelId, restoredChannelId := range restoredChannelsMap {
			if _, ok := sections["messages/"+backedUpChannelId]; !ok {
				continue
			}

			l.Info("Processing backed up channel messages", zap.String("backed_up_channel_id", backedUpChannelId), zap.String("restored_channel_id", restoredChannelId))

			perms := utils.MemberChannelPerms(basePerms, tgtGuild, m, currentChannelMap[restoredChannelId])
			//canManageWebhooks := perms&discordgo.PermissionManageWebhooks == discordgo.PermissionManageWebhooks

			// Fetch section
			bmPtr, err := readMsgpackSection[[]*BackupMessage](f, "messages/"+backedUpChannelId)

			if err != nil {
				if t.Options.IgnoreRestoreErrors {
					continue
				}
				return nil, fmt.Errorf("failed to get section: %w", err)
			}

			bm := *bmPtr

			// Before doing anything else, sort the messages by timestamp
			slices.SortFunc(bm, func(a, b *BackupMessage) int {
				dtA, err := discordgo.SnowflakeTimestamp(a.Message.ID)

				if err != nil {
					panic(err)
				}

				dtB, err := discordgo.SnowflakeTimestamp(b.Message.ID)

				if err != nil {
					panic(err)
				}

				return dtB.Compare(dtA)
			})

			// First batch messages to avoid spam
			var messages = make([]*RestoreMessage, 0)
			var msgIndex int
			var contentLength int64
			var currentMsgAuthor string

			// Grow messages if msgIndex > len(messages)
			growMsgs := func(author *discordgo.User) {
				for {
					if len(messages) > msgIndex {
						break
					}

					messages = append(messages, &RestoreMessage{
						MessageSend: &discordgo.MessageSend{},
						Author:      author,
					})
				}
			}

			for i := range bm {
				if bm[i].Message.Author.ID != currentMsgAuthor {
					currentMsgAuthor = bm[i].Message.Author.ID
					msgIndex++
				}

				if len(bm[i].Message.Content) > 1900 {
					// Upload as file
					content := bm[i].Message.Content

					bm[i].Message.Content = ""

					growMsgs(bm[i].Message.Author)

					messages[msgIndex].SmallFiles = append(messages[msgIndex].SmallFiles, &discordgo.File{
						Name:        "context.txt",
						ContentType: "text/plain",
						Reader:      strings.NewReader(content),
					})
				}

				if contentLength+int64(len(bm[i].Message.Content)) > 1900 {
					contentLength = 0
					msgIndex++
				}

				// Grow messages if msgIndex > len(messages)
				growMsgs(bm[i].Message.Author)

				l.Debug("Processing backed up message", zap.Int("index", i), zap.String("message_id", bm[i].Message.ID), zap.String("author_id", bm[i].Message.Author.ID))

				if len(bm[i].Message.Embeds)+len(messages[msgIndex].MessageSend.Embeds) > 10 {
					// Make the current message only have 10 embeds, then move to the next message and so on
					embeds := bm[i].Message.Embeds

					embedsPaged := [][]*discordgo.MessageEmbed{}

					for len(embeds) > 10 {
						embedsPaged = append(embedsPaged, embeds[:10])
						embeds = embeds[10:]
					}

					embedsPaged = append(embedsPaged, embeds)

					for mod, embeds := range embedsPaged {
						for {
							messages = append(messages, &RestoreMessage{
								MessageSend: &discordgo.MessageSend{},
								Author:      bm[i].Message.Author,
							})

							if len(messages) >= msgIndex+mod {
								break
							}
						}

						messages[msgIndex+mod].MessageSend.Embeds = embeds
					}
				} else {
					messages[msgIndex].MessageSend.Embeds = append(messages[msgIndex].MessageSend.Embeds, bm[i].Message.Embeds...)
				}

				messages[msgIndex].MessageSend.Content += bm[i].Message.Content + "\n"
				contentLength += int64(len(bm[i].Message.Content))

				if bm[i].Message.TTS && perms&discordgo.PermissionSendTTSMessages == discordgo.PermissionSendTTSMessages {
					messages[msgIndex].MessageSend.TTS = bm[i].Message.TTS
				}

				messages[msgIndex].MessageSend.AllowedMentions = &discordgo.MessageAllowedMentions{} // NOTE: We intentionally do not set allowed mentions to avoid spam
			}

			// Send messages
			//
			// NOTE/WIP: message_reference is not supported yet
			// NOTE/WIP: StickerIDs and Components and Attachments/Files are not restored yet
			l.Info("Sending backed up messages", zap.Int("message_count", len(messages)), zap.String("channel_id", restoredChannelId))
			for i := range messages {
				if messages[i].MessageSend.Content == "" &&
					len(messages[i].MessageSend.Embeds) == 0 {
					continue
				}

				messages[i].MessageSend.Content = fmt.Sprintf("**%s**\n%s", strings.ReplaceAll(messages[i].Author.Username+"("+messages[i].Author.ID+")", "*", ""), messages[i].MessageSend.Content)

				_, err := state.Discord.ChannelMessageSendComplex(restoredChannelId, messages[i].MessageSend)

				if err != nil {
					if t.Options.IgnoreRestoreErrors {
						continue
					}
					return nil, fmt.Errorf("failed to send message: %w", err)
				}

				time.Sleep(time.Duration(t.Constraints.Restore.SendMessageSleep))
			}
		}
	}

	l.Info("Successfully restored guild")

	return nil, nil
}

func (t *ServerBackupRestoreTask) Info() *types.TaskInfo {
	return &types.TaskInfo{
		Name: "guild_restore_backup",
		TaskFor: &types.TaskFor{
			ID:         t.ServerID,
			TargetType: types.TargetTypeServer,
		},
		TaskFields: t,
		Valid:      t.valid,
	}
}

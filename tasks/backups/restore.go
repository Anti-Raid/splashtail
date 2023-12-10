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

	// Step 1. Fetch guild data
	t1 = time.Now()

	g, err := readMsgpackSection[discordgo.Guild](f, "core/guild")

	if err != nil {
		return nil, fmt.Errorf("failed to get core data: %w", err)
	}

	if g.ID == "" {
		return nil, fmt.Errorf("guild data is invalid [id is empty], likely an internal decoding error")
	}

	t2 = time.Now()

	l.Info("STATISTICS: guildread", zap.Float64("duration", t2.Sub(t1).Seconds()))

	t1 = time.Now()

	// Edit guild. Note that settings related to ID's are changed later if needed
	// Notes:
	//
	// - Region is not restored
	// - Owner is not restored
	gp := &discordgo.GuildParams{
		Name:                        g.Name,
		Description:                 g.Description,
		VerificationLevel:           &g.VerificationLevel,
		DefaultMessageNotifications: int(g.DefaultMessageNotifications),
		ExplicitContentFilter:       int(g.ExplicitContentFilter),
		AfkTimeout:                  g.AfkTimeout,
	}

	// Icons
	canUseIcon := slices.Contains(g.Features, "ANIMATED_ICON") || !strings.HasPrefix(g.Icon, "a_")
	canUseBanner := (slices.Contains(g.Features, "BANNER") && !strings.HasPrefix(g.Banner, "a_")) || slices.Contains(g.Features, "ANIMATED_BANNER")
	canUseSplash := slices.Contains(g.Features, "INVITE_SPLASH") && !strings.HasPrefix(g.Splash, "a_")

	if g.Icon != "" {
		if !canUseIcon {
			l.Warn("Not restoring animated icon on unsupported server", zap.String("guild_id", g.ID))
		} else {
			icon, err := getImageAsDataUri(l, f, "guildIcon", g.IconURL("1024"), bo)

			if err != nil {
				return nil, fmt.Errorf("failed to get icon: %w", err)
			}

			gp.Icon = icon
		}
	}

	if g.Banner != "" {
		if !canUseBanner {
			l.Warn("Not restoring banner on unsupported server", zap.String("guild_id", g.ID))
		} else {
			banner, err := getImageAsDataUri(l, f, "guildBanner", g.BannerURL("1024"), bo)

			if err != nil {
				return nil, fmt.Errorf("failed to get banner: %w", err)
			}

			gp.Banner = banner
		}
	}

	if g.Splash != "" {
		if !canUseSplash {
			l.Warn("Not restoring splash on unsupported server", zap.String("guild_id", g.ID))
		} else {
			splash, err := getImageAsDataUri(l, f, "guildSplash", discordgo.EndpointGuildSplash(g.ID, g.Splash), bo)

			if err != nil {
				return nil, fmt.Errorf("failed to get splash: %w", err)
			}

			gp.Splash = splash
		}
	}

	// Features, only COMMUNITY is editable IIRC
	var features []discordgo.GuildFeature
	basePerms := utils.BasePermissions(tgtGuild, m)

	if basePerms&discordgo.PermissionAdministrator == discordgo.PermissionAdministrator {
		if slices.Contains(g.Features, discordgo.GuildFeatureCommunity) {
			features = append(features, discordgo.GuildFeatureCommunity)
		}
	} else {
		l.Warn("Not admin, certain features may not be editable", zap.String("guild_id", g.ID))
	}

	gp.Features = features

	_, err = state.Discord.GuildEdit(t.ServerID, gp)

	if err != nil {
		return nil, fmt.Errorf("failed to edit guild: %w", err)
	}

	t2 = time.Now()

	l.Info("STATISTICS: guildedit", zap.Float64("duration", t2.Sub(t1).Seconds()))

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

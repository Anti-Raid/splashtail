package backups

import (
	"fmt"
	"net/http"
	"splashtail/types"
	"strings"
	"time"

	"github.com/infinitybotlist/iblfile"
	"github.com/infinitybotlist/iblfile/autoencryptedencoders/aes256"
	"github.com/infinitybotlist/iblfile/autoencryptedencoders/noencryption"
	"github.com/jackc/pgx/v5"
	"go.uber.org/zap"
)

// A task to restore a backup of a server
type ServerBackupRestoreTask struct {
	// The ID of the server
	ServerID string `json:"server_id"`

	// Backup options
	Options BackupRestoreOpts `json:"backup_opts"`

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

	l.Info("STATISTICS: sections", zap.Float64("duration", t2.Sub(t1).Seconds()), zap.Strings("keys", keys))

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

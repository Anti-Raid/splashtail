package backups

import (
	"fmt"
	"splashtail/types"
	"strings"

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

package backups

import (
	"fmt"
	"os"
	"splashtail/state"
	"splashtail/tasks"
	"time"

	"github.com/infinitybotlist/iblfile"
	"github.com/jackc/pgx/v5"
	"go.uber.org/zap"
)

func init() {
	tasks.RegisterTask(&ServerBackupCreateTask{})
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
	l.Info("Backing up core data", zap.String("taskId", t.TaskID))

	if t.BackupOpts.MaxMessages == 0 {
		t.BackupOpts.MaxMessages = totalMaxMessages
	}

	if t.BackupOpts.MaxMessages > totalMaxMessages {
		return fmt.Errorf("max_messages cannot be greater than %d", totalMaxMessages)
	}

	f, err := iblfile.NewAutoEncryptedFile("")

	if err != nil {
		return fmt.Errorf("error creating file: %w", err)
	}

	// Fetch guild
	g, err := state.Discord.Guild(t.ServerID)

	if err != nil {
		return fmt.Errorf("error fetching guild: %w", err)
	}

	// Fetch channels of guild
	channels, err := state.Discord.GuildChannels(t.ServerID)

	if err != nil {
		return fmt.Errorf("error fetching channels: %w", err)
	}

	g.Channels = channels

	cb := CoreBackup{
		Guild: g,
	}

	f.WriteJsonSection(cb, "core")

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

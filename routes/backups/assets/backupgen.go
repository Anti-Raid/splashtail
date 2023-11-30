package assets

import (
	"fmt"
	"os"
	"splashtail/state"
	"splashtail/tasks"
	"time"

	"github.com/infinitybotlist/iblfile"
	"go.uber.org/zap"
)

func createServerBackupImpl(taskId, id string, l *zap.Logger) (*iblfile.AutoEncryptedFile, error) {
	l.Info("Backing up core data", zap.String("taskId", taskId))

	f, err := iblfile.NewAutoEncryptedFile("")

	if err != nil {
		return nil, fmt.Errorf("error creating file: %w", err)
	}

	// Fetch guild
	g, err := state.Discord.Guild(id)

	if err != nil {
		return nil, fmt.Errorf("error fetching guild: %w", err)
	}

	// Fetch channels of guild
	channels, err := state.Discord.GuildChannels(id)

	if err != nil {
		return nil, fmt.Errorf("error fetching channels: %w", err)
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
		return nil, fmt.Errorf("error getting format: %w", err)
	}

	metadata.FormatVersion = ifmt.Version

	err = f.WriteJsonSection(metadata, "meta")

	if err != nil {
		l.Error("Error creating backup", zap.Error(err))
		return nil, fmt.Errorf("error writing metadata: %w", err)
	}

	return f, nil
}

func CreateServerBackup(taskId, taskName, id string) {
	l, _ := tasks.NewTaskLogger(taskId)

	var done bool

	// Fail failed tasks
	defer func() {
		err := recover()

		if err != nil {
			l.Error("Panic", zap.Any("err", err), zap.String("id", id))

			_, err := state.Pool.Exec(state.Context, "UPDATE tasks SET state = $1 WHERE task_id = $2", "failed", taskId)

			if err != nil {
				l.Error("Failed to update task", zap.Error(err), zap.String("id", id))
			}
		}

		if !done {
			l.Error("Failed to complete task", zap.String("id", id))

			_, err := state.Pool.Exec(state.Context, "UPDATE tasks SET state = $1 WHERE task_id = $2", "failed", taskId)

			if err != nil {
				l.Error("Failed to update task", zap.Error(err), zap.String("id", id))
			}
		}
	}()

	l.Info("Creating server backup", zap.String("taskId", taskId))

	tx, err := state.Pool.Begin(state.Context)

	if err != nil {
		l.Error("Failed to begin transaction", zap.Error(err), zap.String("id", id))
		return
	}

	defer tx.Rollback(state.Context)

	_, err = tx.Exec(state.Context, "DELETE FROM tasks WHERE task_name = $1 AND task_id != $2 AND for_user = $3", taskName, taskId, "g/"+id)

	if err != nil {
		l.Error("Failed to delete old data tasks", zap.Error(err), zap.String("id", id))
		return
	}

	// Do stuff here
	backup, err := createServerBackupImpl(taskId, id, l)

	if err != nil {
		l.Error("ERROR:", zap.Error(err), zap.String("id", id))
		return
	}

	// Create $SecureStorage/guilds/$guildId/backups/$taskId
	err = os.MkdirAll(fmt.Sprintf("%s/guilds/%s/backups/%s", state.Config.Meta.SecureStorage, id, taskId), 0700)

	if err != nil {
		l.Error("Failed to create directory", zap.Error(err), zap.String("id", id))
		return
	}

	// Write backup to $SecureStorage/guilds/$guildId/backups/$taskId/backup.arbackup
	file, err := os.Create(fmt.Sprintf("%s/guilds/%s/backups/%s/backup.arbackup", state.Config.Meta.SecureStorage, id, taskId))

	if err != nil {
		l.Error("Failed to create file", zap.Error(err), zap.String("id", id))
		return
	}

	defer file.Close()

	err = backup.WriteOutput(file)

	if err != nil {
		l.Error("Failed to write backup", zap.Error(err), zap.String("id", id))
		return
	}

	l.Info("Successfully created backup", zap.String("id", id))

	// Commit tx
	_, err = tx.Exec(state.Context, "UPDATE tasks SET output = $1, state = $2 WHERE task_id = $3", map[string]string{}, "completed", taskId)

	if err != nil {
		l.Error("Failed to update task", zap.Error(err), zap.String("id", id))
		return
	}

	err = tx.Commit(state.Context)

	if err != nil {
		l.Error("Failed to commit transaction", zap.Error(err), zap.String("id", id))
		return
	}

	done = true
}

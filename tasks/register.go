package tasks

import (
	"github.com/anti-raid/splashtail/tasks/tasks/backups"
	"github.com/anti-raid/splashtail/tasks/tasks/moderation"
)

// Add all tasks here
func init() {
	RegisterTaskDefinition(&backups.ServerBackupCreateTask{})
	RegisterTaskDefinition(&backups.ServerBackupRestoreTask{})
	RegisterTaskDefinition(&moderation.MessagePruneTask{})
}

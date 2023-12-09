package tasks

import (
	"splashtail/tasks/backups"
)

// Add all tasks here
func init() {
	RegisterTaskDefinition(&backups.ServerBackupCreateTask{})
	RegisterTaskDefinition(&backups.ServerBackupRestoreTask{})
}

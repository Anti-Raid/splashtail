package tasks

import (
	"github.com/anti-raid/splashtail/tasks/backups"
)

// Add all tasks here
func init() {
	RegisterTaskDefinition(&backups.ServerBackupCreateTask{})
	RegisterTaskDefinition(&backups.ServerBackupRestoreTask{})
}

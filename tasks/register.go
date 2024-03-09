package tasks

import (
	"github.com/anti-raid/splashtail/tasks/taskdef"
	"github.com/anti-raid/splashtail/tasks/tasks/backups"
	"github.com/anti-raid/splashtail/tasks/tasks/moderation"
)

// Task management core
var TaskDefinitionRegistry = map[string]taskdef.TaskDefinition{}

func RegisterTaskDefinition(task taskdef.TaskDefinition) {
	TaskDefinitionRegistry[task.Info().Name] = task
}

// Add all tasks here
func init() {
	RegisterTaskDefinition(&backups.ServerBackupCreateTask{})
	RegisterTaskDefinition(&backups.ServerBackupRestoreTask{})
	RegisterTaskDefinition(&moderation.MessagePruneTask{})
}

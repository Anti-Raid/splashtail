package jobs

import (
	"go.jobs/taskdef"
	"go.jobs/tasks/backups"
	"go.jobs/tasks/moderation"
)

// Task management core
var TaskDefinitionRegistry = map[string]taskdef.TaskDefinition{}

func RegisterTaskDefinition(task taskdef.TaskDefinition) {
	TaskDefinitionRegistry[task.Name()] = task
}

// Add all tasks here
func init() {
	RegisterTaskDefinition(&backups.ServerBackupCreateTask{})
	RegisterTaskDefinition(&backups.ServerBackupRestoreTask{})
	RegisterTaskDefinition(&moderation.MessagePruneTask{})
}

package jobs

import (
	"github.com/anti-raid/splashtail/core/go.jobs/taskdef"
	"github.com/anti-raid/splashtail/core/go.jobs/tasks/backups"
	"github.com/anti-raid/splashtail/core/go.jobs/tasks/moderation"
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

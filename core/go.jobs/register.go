package jobs

import (
	"go.jobs/interfaces"
	"go.jobs/tasks/backups"
	"go.jobs/tasks/moderation"
)

// Job impl registry
var JobImplRegistry = map[string]interfaces.JobImpl{}

func RegisterJobImpl(task interfaces.JobImpl) {
	JobImplRegistry[task.Name()] = task
}

// Add all tasks here
func init() {
	RegisterJobImpl(&backups.ServerBackupCreateTask{})
	RegisterJobImpl(&backups.ServerBackupRestoreTask{})
	RegisterJobImpl(&moderation.MessagePruneTask{})
}

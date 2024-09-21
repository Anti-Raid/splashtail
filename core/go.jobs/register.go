package jobs

import (
	"go.jobs/interfaces"
	"go.jobs/jobs/backups"
	"go.jobs/jobs/moderation"
)

// Job impl registry
var JobImplRegistry = map[string]interfaces.JobImpl{}

func RegisterJobImpl(jobImpl interfaces.JobImpl) {
	JobImplRegistry[jobImpl.Name()] = jobImpl
}

// Add all jobs here
func init() {
	RegisterJobImpl(&backups.ServerBackupCreate{})
	RegisterJobImpl(&backups.ServerBackupRestore{})
	RegisterJobImpl(&moderation.MessagePrune{})
}

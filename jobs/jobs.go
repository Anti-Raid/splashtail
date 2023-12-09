package jobs

import "time"

var JobRegistry = map[string]Job{}

func RegisterJob(job Job) {
	JobRegistry[job.Name()] = job
}

// Inspired from https://github.com/InfinityBotList/Arcadia/blob/main/src/tasks/taskcat.rs
type Job interface {
	// Whether or not the task is enabled
	Enabled() bool

	// How often the task should run
	Duration() time.Duration

	// Name of the task
	Name() string

	// Description of the task
	Description() string

	// Function to run the task
	Run() error
}

func StartAllJobs() {}

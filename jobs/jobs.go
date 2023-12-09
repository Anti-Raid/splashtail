package jobs

import (
	"splashtail/state"
	"sync"
	"time"

	"go.uber.org/zap"
)

var taskMutex sync.Mutex

var JobRegistry = []Job{}

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

func StartAllJobs() {
	for _, job := range JobRegistry {
		if job.Enabled() {
			go runTask(job)
		}
	}
}

func runTask(job Job) {
	defer func() {
		if err := recover(); err != nil {
			state.Logger.Info("task crashed", zap.String("task", job.Name()), zap.Any("error", err))
			runTask(job)
		}

		panic("task crashed")
	}()

	duration := job.Duration()
	description := job.Description()
	name := job.Name()

	for {
		time.Sleep(duration)

		taskMutex.Lock()

		state.Logger.Info("Running task", zap.String("task", name), zap.Duration("duration", duration), zap.String("description", description))

		err := job.Run()

		if err != nil {
			state.Logger.Error("task failed", zap.String("task", name), zap.Error(err), zap.String("description", description))
		}

		taskMutex.Unlock()
	}
}

package bgtasks

import (
	"sync"
	"time"

	"github.com/anti-raid/splashtail/jobserver/state"

	"go.uber.org/zap"
)

var taskMutex sync.Mutex

var BgTaskRegistry = []BackgroundTask{}

// Inspired from https://github.com/InfinityBotList/Arcadia/blob/main/src/tasks/taskcat.rs
type BackgroundTask interface {
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

func StartAllTasks() {
	for _, bgTask := range BgTaskRegistry {
		if bgTask.Enabled() {
			go runTask(bgTask)
		}
	}
}

func runTask(bgTask BackgroundTask) {
	defer func() {
		if err := recover(); err != nil {
			state.Logger.Info("task crashed", zap.String("task", bgTask.Name()), zap.Any("error", err))
			runTask(bgTask)
		}

		panic("task crashed")
	}()

	duration := bgTask.Duration()
	description := bgTask.Description()
	name := bgTask.Name()

	for {
		time.Sleep(duration)

		taskMutex.Lock()

		state.Logger.Info("Running task", zap.String("task", name), zap.Duration("duration", duration), zap.String("description", description))

		err := bgTask.Run()

		if err != nil {
			state.Logger.Error("task failed", zap.String("task", name), zap.Error(err), zap.String("description", description))
		}

		taskMutex.Unlock()
	}
}

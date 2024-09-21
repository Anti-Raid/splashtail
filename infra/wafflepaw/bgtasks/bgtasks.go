package bgtasks

import (
	"sync"
	"time"

	"go.uber.org/zap"
)

var taskMutex sync.Mutex

var BgTaskRegistry = []BackgroundTask{}

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

func StartAllTasks(l *zap.Logger) {
	for _, bgTask := range BgTaskRegistry {
		if bgTask.Enabled() {
			go runTask(l, bgTask)
		}
	}
}

func runTask(l *zap.Logger, bgTask BackgroundTask) {
	defer func() {
		if err := recover(); err != nil {
			l.Info("task crashed", zap.String("task", bgTask.Name()), zap.Any("error", err))
			runTask(l, bgTask)
		}

		panic("task crashed")
	}()

	duration := bgTask.Duration()
	description := bgTask.Description()
	name := bgTask.Name()

	for {
		time.Sleep(duration)

		taskMutex.Lock()

		l.Info("Running task", zap.String("task", name), zap.Duration("duration", duration), zap.String("description", description))

		err := bgTask.Run()

		if err != nil {
			l.Error("task failed", zap.String("task", name), zap.Error(err), zap.String("description", description))
		}

		taskMutex.Unlock()
	}
}

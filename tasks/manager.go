package tasks

import (
	"github.com/anti-raid/splashtail/splashcore/types"
	"github.com/anti-raid/splashtail/tasks/taskstate"

	"go.uber.org/zap"
)

// Task management core
var TaskDefinitionRegistry = map[string]TaskDefinition{}

func RegisterTaskDefinition(task TaskDefinition) {
	TaskDefinitionRegistry[task.Info().Name] = task
}

// TaskDefinition is the definition for any task that can be executed on splashtail
type TaskDefinition interface {
	// Validate validates the task and sets up state if needed
	Validate(state taskstate.TaskState) error

	// Exec executes the task returning an output if any
	Exec(l *zap.Logger, tcr *types.TaskCreateResponse, state taskstate.TaskState, progstate taskstate.TaskProgressState) (*types.TaskOutput, error)

	// Returns the info on a task
	Info() *types.TaskInfo
}

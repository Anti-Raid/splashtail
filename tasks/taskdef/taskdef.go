// To avoid circular dependencies, taskdef contains the core TaskDefinition
package taskdef

import (
	"github.com/anti-raid/splashtail/splashcore/types"
	"github.com/anti-raid/splashtail/tasks/taskstate"
	"go.uber.org/zap"
)

// TaskDefinition is the definition for any task that can be executed on splashtail
type TaskDefinition interface {
	// Validate validates the task and sets up state if needed
	Validate(state taskstate.TaskState) error

	// Exec executes the task returning an output if any
	Exec(l *zap.Logger, tcr *types.TaskCreateResponse, state taskstate.TaskState, progstate taskstate.TaskProgressState) (*types.TaskOutput, error)

	// Returns the info on a task
	Info() *types.TaskInfo

	// LocalPresets returns the preset options of a task
	LocalPresets() *PresetInfo
}

type PresetInfo struct {
	// Whether or not this task should be runnable
	Runnable bool

	// The default options/data of the task
	Preset TaskDefinition

	// Any comments for specific fields
	Comments map[string]string
}

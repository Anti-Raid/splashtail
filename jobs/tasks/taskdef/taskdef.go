// To avoid circular dependencies, taskdef contains the core TaskDefinition
package taskdef

import (
	"time"

	"github.com/anti-raid/splashtail/splashcore/types"
	"github.com/anti-raid/splashtail/jobs/tasks/taskstate"
	"go.uber.org/zap"
)

// TaskDefinition is the definition for any task that can be executed on splashtail
type TaskDefinition interface {
	// Name returns the name of the task
	Name() string

	// TaskFor returns who the task is for
	TaskFor() *types.TaskFor

	// As tasks often deal with sensitive data such as secrets, the TaskFields method returns
	// a map of fields that can be stored in the database
	TaskFields() map[string]any

	// Validate validates the task and sets up state if needed
	Validate(state taskstate.TaskState) error

	// Exec executes the task returning an output if any
	Exec(l *zap.Logger, state taskstate.TaskState, progstate taskstate.TaskProgressState) (*types.TaskOutput, error)

	// Expiry returns when the task will expire (if any), setting this to nil will make the task not expire
	Expiry() *time.Duration

	// Resumable returns whether or not the task is resumable
	Resumable() bool

	// CorrespondingBotCommand_Create returns the bot command that should be checked for ACL purposes to create such a task
	CorrespondingBotCommand_Create() string

	// CorrespondingBotCommand_Download returns the bot command that should be checked for ACL purposes to download such a task
	CorrespondingBotCommand_Download() string

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

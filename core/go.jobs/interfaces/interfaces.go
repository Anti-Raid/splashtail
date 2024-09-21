// To avoid circular dependencies, interfaces contains the core implementation of the job interfaces
package interfaces

import (
	"time"

	jobstate "go.jobs/state"
	"go.jobs/types"
	"go.uber.org/zap"
)

// JobImpl provides the definition for any job that can be executed on splashtail
type JobImpl interface {
	// Name returns the name of the job
	Name() string

	// Owner returns who the job is for
	Owner() *types.Owner

	// As jobs often deal with sensitive data such as secrets, the Fields method returns
	// a map of fields that can be stored in the database
	Fields() map[string]any

	// Validate validates and sets up state if needed
	Validate(state jobstate.State) error

	// Exec executes the job returning an output if any
	Exec(l *zap.Logger, state jobstate.State, progstate jobstate.ProgressState) (*types.Output, error)

	// Expiry returns when the job will expire (if any), setting this to nil will make the job not expire
	Expiry() *time.Duration

	// Resumable returns whether or not the job is resumable
	Resumable() bool

	// CorrespondingBotCommand_View returns the bot command that should be checked for ACL purposes to list/view the job
	CorrespondingBotCommand_View() string

	// CorrespondingBotCommand_Create returns the bot command that should be checked for ACL purposes to create the job
	CorrespondingBotCommand_Create() string

	// CorrespondingBotCommand_Download returns the bot command that should be checked for ACL purposes to download the job
	CorrespondingBotCommand_Download() string

	// LocalPresets returns the preset options of a job
	LocalPresets() *PresetInfo
}

type PresetInfo struct {
	// Whether or not this job should be runnable
	Runnable bool

	// The default options/data
	Preset JobImpl

	// Any comments for specific fields
	Comments map[string]string
}

package types

import (
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"strings"
	"time"

	"go.std/splashcore"
	"go.std/utils"
)

type TaskCreateResponse struct {
	ID string `json:"id" description:"The id of the task"`
}

// @ci table=tasks
//
// Jobs are background processes that can be run on a coordinator server.
type Task struct {
	ID         string           `db:"id" json:"id" validate:"required" description:"The ID of the job."`
	Name       string           `db:"name" json:"name" validate:"required" description:"The name of the job."`
	Output     *TaskOutput      `db:"output" json:"output" description:"The output of the job."`
	Fields     map[string]any   `db:"fields" json:"fields" description:"The public fields of the job. Note that sensitive data may be omitted from storage entirely"`
	Statuses   []map[string]any `db:"statuses" json:"statuses" validate:"required" description:"The task statuses."`
	TaskForRaw *string          `db:"task_for" json:"-" description:"The entity this job is for." ci:"internal"`
	TaskFor    *TaskFor         `db:"-" json:"task_for" description:"The entity this job is for."`
	Expiry     *time.Duration   `db:"expiry" json:"expiry" validate:"required" description:"The task expiry."`
	State      string           `db:"state" json:"state" validate:"required" description:"The tasks current state (pending/completed etc)."`
	Resumable  bool             `db:"resumable" json:"resumable" description:"Whether the task is resumable."`
	CreatedAt  time.Time        `db:"created_at" json:"created_at" description:"The time the task was created."`
}

// @ci table=tasks unfilled=1
//
// A PartialTask represents a partial representation of a job.
type PartialTask struct {
	ID        string         `db:"id" json:"id" validate:"required" description:"The ID of the job."`
	Name      string         `db:"name" json:"name" validate:"required" description:"The name of the job."`
	Expiry    *time.Duration `db:"expiry" json:"expiry" validate:"required" description:"The job expiry."`
	State     string         `db:"state" json:"state" validate:"required" description:"The jobs' current state (pending/completed etc)."`
	CreatedAt time.Time      `db:"created_at" json:"created_at" description:"The time the job was created."`
}

type TaskListResponse struct {
	Tasks []PartialTask `json:"tasks" description:"The list of (partial) tasks"`
}

// TaskFor is a struct containing the internal representation of who a task is for
type TaskFor struct {
	ID         string `json:"id" description:"The ID of the entity the task is for"`
	TargetType string `json:"target_type" description:"The type of the entity the task is for"`
}

// Marshal TaskFor using canonical form always
func (t *TaskFor) MarshalJSON() ([]byte, error) {
	formattedTaskFor, err := FormatTaskFor(t)

	if err != nil {
		return nil, err
	}

	return json.Marshal(formattedTaskFor)
}

// Unmarshal TaskFor using canonical form always
func (t *TaskFor) UnmarshalJSON(data []byte) error {
	formattedTaskFor := ParseTaskFor(string(data))

	if formattedTaskFor == nil {
		return errors.New("invalid task for")
	}

	*t = *formattedTaskFor

	return nil
}

// Formats a TaskFor into a string under the 'normal' type. Returns nil if the TaskFor is nil or has an invalid target type
func FormatTaskFor(fu *TaskFor) (*string, error) {
	if fu == nil {
		return nil, errors.New("formattaskfor: task for is nil")
	}

	switch fu.TargetType {
	case splashcore.TargetTypeUser:
		return utils.Pointer("u/" + fu.ID), nil
	case splashcore.TargetTypeServer:
		return utils.Pointer("g/" + fu.ID), nil
	default:
		return nil, fmt.Errorf("formattaskfor: invalid target type: %s", fu.TargetType)
	}
}

// Parses a TaskFor from a string. Returns nil if the string is invalid.
//
// TaskFor must be in 'normal' (not simplex) form (e.g: u/1234567890).
func ParseTaskFor(fu string) *TaskFor {
	fuSplit := strings.SplitN(fu, "/", 2)

	if len(fuSplit) != 2 {
		return nil
	}

	switch fuSplit[0] {
	case "u":
		return &TaskFor{
			TargetType: splashcore.TargetTypeUser,
			ID:         fuSplit[1],
		}
	case "g":
		return &TaskFor{
			TargetType: splashcore.TargetTypeServer,
			ID:         fuSplit[1],
		}
	default:
		return nil
	}
}

// TaskOutput is the output of a task
type TaskOutput struct {
	Filename   string        `json:"filename"`
	Segregated bool          `json:"segregated"` // If this flag is set, then the stored output will be stored in $taskForSimplexFormat/$Name/$taskId/$filename instead of $taskId/$filename
	Buffer     *bytes.Buffer `json:"-"`
}

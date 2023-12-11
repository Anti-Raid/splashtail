package types

import (
	"bytes"
	"time"
)

// TaskCreateRequest is the response upon creating a task
type TaskCreateResponse struct {
	TaskID   string    `json:"task_id" description:"The ID of the newly created task"`
	TaskKey  *string   `json:"task_key" description:"The key of the newly created task"`
	TaskInfo *TaskInfo `json:"task_info" description:"The task info."`
}

type TaskCreateResponseWithWait struct {
	TaskCreateResponse *TaskCreateResponse `json:"task_create_response" description:"The task create response"`
	Output             any                 `json:"output" description:"The task output"`
}

// @ci table=tasks
//
// Tasks are background processes that can be run on a coordinator server.
type Task struct {
	TaskID               string           `db:"task_id" json:"task_id" validate:"required" description:"The task ID."`
	TaskKey              *string          `db:"task_key" json:"-" validate:"required" description:"The task key."`
	AllowUnauthenticated bool             `db:"allow_unauthenticated" json:"allow_unauthenticated" description:"Whether the task can be accessed without authentication"`
	TaskName             string           `db:"task_name" json:"task_name" validate:"required" description:"The task name."`
	Output               *TaskOutput      `db:"output" json:"output" description:"The task output."`
	TaskInfo             *TaskInfo        `db:"task_info" json:"task_info" description:"The task info."`
	Statuses             []map[string]any `db:"statuses" json:"statuses" validate:"required" description:"The task statuses."`
	TaskForRaw           *string          `db:"task_for" json:"-" description:"The entity this task is for." ci:"internal"`
	TaskFor              *TaskFor         `db:"-" json:"task_for" description:"The entity this task is for."`
	Expiry               *time.Duration   `db:"expiry" json:"expiry" validate:"required" description:"The task expiry."`
	State                string           `db:"state" json:"state" validate:"required" description:"The tasks current state (pending/completed etc)."`
	CreatedAt            time.Time        `db:"created_at" json:"created_at" description:"The time the task was created."`
}

// TaskFor is a struct containing the internal representation of who a task is for
type TaskFor struct {
	ID         string `json:"id" description:"The ID of the entity the task is for"`
	TargetType string `json:"target_type" description:"The type of the entity the task is for"`
}

// TaskOutput is the output of a task
type TaskOutput struct {
	Filename   string        `json:"filename"`
	Segregated bool          `json:"segregated"` // If this flag is set, then the stored output will be stored in $taskForSimplexFormat/$taskName/$taskId/$filename instead of $taskId/$filename
	Buffer     *bytes.Buffer `json:"-"`
}

// Information on a task
type TaskInfo struct {
	Name                 string        `json:"name" description:"The task name."`
	TaskFor              *TaskFor      `json:"task_for" description:"The entity this task is for."`
	AllowUnauthenticated bool          `json:"allow_unauthenticated" description:"Whether the task can be accessed without authentication"`
	TaskFields           any           `json:"task_fields" description:"The task fields."`
	Expiry               time.Duration `json:"expiry"`
	Valid                bool          `json:"valid"`
}

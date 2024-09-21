package types

import (
	"bytes"
	"time"
)

type TaskCreateResponse struct {
	ID string `json:"id" description:"The id of the task"`
}

// @ci table=jobs
//
// Jobs are background processes that can be run on a coordinator server.
type Task struct {
	ID        string           `db:"id" json:"id" validate:"required" description:"The ID of the job."`
	Name      string           `db:"name" json:"name" validate:"required" description:"The name of the job."`
	Output    *Output          `db:"output" json:"output" description:"The output of the job."`
	Fields    map[string]any   `db:"fields" json:"fields" description:"The public fields of the job. Note that sensitive data may be omitted from storage entirely"`
	Statuses  []map[string]any `db:"statuses" json:"statuses" validate:"required" description:"The job statuses."`
	OwnerRaw  *string          `db:"owner" json:"-" description:"The entity this job is for." ci:"internal"`
	Owner     *Owner           `db:"-" json:"owner" description:"The entity this job is for."`
	Expiry    *time.Duration   `db:"expiry" json:"expiry" validate:"required" description:"The job expiry."`
	State     string           `db:"state" json:"state" validate:"required" description:"The jobs' current state (pending/completed etc)."`
	Resumable bool             `db:"resumable" json:"resumable" description:"Whether the job is resumable."`
	CreatedAt time.Time        `db:"created_at" json:"created_at" description:"The time the job was created."`
}

// @ci table=jobs unfilled=1
//
// A PartialTask represents a partial representation of a job.
type PartialTask struct {
	ID        string         `db:"id" json:"id" validate:"required" description:"The ID of the job."`
	Name      string         `db:"name" json:"name" validate:"required" description:"The name of the job."`
	Expiry    *time.Duration `db:"expiry" json:"expiry" validate:"required" description:"The job expiry."`
	State     string         `db:"state" json:"state" validate:"required" description:"The jobs' current state (pending/completed etc)."`
	CreatedAt time.Time      `db:"created_at" json:"created_at" description:"The time the job was created."`
}

type JobListResponse struct {
	Jobs []PartialTask `json:"jobs" description:"The list of (partial) jobs"`
}

// Owner is a struct containing the internal representation of who a task is for
type Owner struct {
	ID         string `json:"id" description:"The ID of the entity the task is for"`
	TargetType string `json:"target_type" description:"The type of the entity the task is for"`
}

// Output is the output of a task
type Output struct {
	Filename   string        `json:"filename"`
	Segregated bool          `json:"segregated"` // If this flag is set, then the stored output will be stored in $taskForSimplexFormat/$Name/$taskId/$filename instead of $taskId/$filename
	Buffer     *bytes.Buffer `json:"-"`
}

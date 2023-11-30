package types

import (
	"github.com/jackc/pgx/v5/pgtype"
)

type TaskCreateResponse struct {
	TaskID               string          `json:"task_id" description:"The ID of the newly created task"`
	TaskKey              pgtype.Text         `json:"task_key" description:"The key of the newly created task"`
	AllowUnauthenticated bool            `json:"allow_unauthenticated" description:"Whether the task can be accessed without authentication"`
	TaskName             string          `db:"task_name" json:"task_name" validate:"required" description:"The task name."`
	Expiry               pgtype.Interval `db:"expiry" json:"expiry" validate:"required" description:"The task expiry."`
}

// @ci table=tasks
//
// Tasks are background processes that can be run on the server.
type Task struct {
	TaskId               string             `db:"task_id" json:"task_id" validate:"required" description:"The task ID."`
	TaskKey              pgtype.Text        `db:"task_key" json:"-" validate:"required" description:"The task key."`
	AllowUnauthenticated bool               `db:"allow_unauthenticated" json:"allow_unauthenticated" description:"Whether the task can be accessed without authentication"`
	TaskName             string             `db:"task_name" json:"task_name" validate:"required" description:"The task name."`
	Output               map[string]any     `db:"output" json:"output" description:"The task output."`
	Statuses             []map[string]any   `db:"statuses" json:"statuses" validate:"required" description:"The task statuses."`
	ForUser              pgtype.Text        `db:"for_user" json:"for_user" description:"The user this task is for."`
	Expiry               pgtype.Interval    `db:"expiry" json:"expiry" validate:"required" description:"The task expiry."`
	State                string             `db:"state" json:"state" validate:"required" description:"The tasks current state (pending/completed etc)."`
	CreatedAt            pgtype.Timestamptz `db:"created_at" json:"created_at" description:"The time the task was created."`
}

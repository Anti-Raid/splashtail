export type KV = { [key: string]: any }

/*type TaskCreateResponse struct {
	TaskID               string        `json:"task_id" description:"The ID of the newly created task"`
	TaskKey              *string       `json:"task_key" description:"The key of the newly created task"`
	AllowUnauthenticated bool          `json:"allow_unauthenticated" description:"Whether the task can be accessed without authentication"`
	TaskName             string        `db:"task_name" json:"task_name" validate:"required" description:"The task name."`
	Expiry               time.Duration `db:"expiry" json:"expiry" validate:"required" description:"The task expiry."`
}
*/
export interface TaskCreateResponse {
    task_id: string;
    task_key?: string;
    allow_unauthenticated: boolean;
    task_name: string;
    expiry: number;
}

/*
// Tasks are background processes that can be run on a coordinator server.
type Task struct {
	TaskID               string           `db:"task_id" json:"task_id" validate:"required" description:"The task ID."`
	TaskKey              *string          `db:"task_key" json:"-" validate:"required" description:"The task key."`
	AllowUnauthenticated bool             `db:"allow_unauthenticated" json:"allow_unauthenticated" description:"Whether the task can be accessed without authentication"`
	TaskName             string           `db:"task_name" json:"task_name" validate:"required" description:"The task name."`
	Output               map[string]any   `db:"output" json:"output" description:"The task output."`
	Statuses             []map[string]any `db:"statuses" json:"statuses" validate:"required" description:"The task statuses."`
	ForUser              *string          `db:"for_user" json:"for_user" description:"The user this task is for."`
	Expiry               time.Duration    `db:"expiry" json:"expiry" validate:"required" description:"The task expiry."`
	State                string           `db:"state" json:"state" validate:"required" description:"The tasks current state (pending/completed etc)."`
	CreatedAt            time.Time        `db:"created_at" json:"created_at" description:"The time the task was created."`
}
 */
export interface Task {
    task_id: string;
    task_key?: string;
    allow_unauthenticated: boolean;
    task_name: string;
    output: KV;
    statuses: KV[];
    for_user?: string;
    expiry: number;
    state: string;
    created_at: number;
}
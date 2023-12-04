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
package rpc_messages

import (
	_ "go.jobserver/state" // Avoid unsafe import
)

// Spawns a task and executes it if the execute argument is set.
type SpawnTask struct {
	Name    string                 `json:"name"`
	Data    map[string]interface{} `json:"data"`
	Create  bool                   `json:"create"`
	Execute bool                   `json:"execute"`

	// If create is false, then task id must be set
	TaskID string `json:"task_id"`

	// The User ID who initiated the action
	UserID string `json:"user_id"`
}

type SpawnTaskResponse struct {
	TaskID string `json:"task_id"`
}

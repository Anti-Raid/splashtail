package animusmagic_messages

import (
	"github.com/anti-raid/splashtail/core/go.std/animusmagic"
	_ "github.com/anti-raid/splashtail/services/go.jobserver/state" // Avoid unsafe import
)

type JobserverMessage struct {
	// spawns a task and executes it if the execute argument is set.
	// If you already have both a task and a task create response, consider execute_task
	SpawnTask *struct {
		Name    string                 `json:"name"`
		Data    map[string]interface{} `json:"data"`
		Create  bool                   `json:"create"`
		Execute bool                   `json:"execute"`

		// If create is false, then task id must be set
		TaskID string `json:"task_id"`

		// The User ID who initiated the action
		UserID string `json:"user_id"`
	} `json:"SpawnTask,omitempty"`
}

func (b JobserverMessage) Message() {}

func (b JobserverMessage) Target() animusmagic.AnimusTarget {
	return animusmagic.AnimusTargetBot
}

type JobserverResponse struct {
	SpawnTask *struct {
		TaskID string `json:"task_id"`
	} `json:"SpawnTask,omitempty"`
}

func (b JobserverResponse) Response() {}

func (b JobserverResponse) Target() animusmagic.AnimusTarget {
	return animusmagic.AnimusTargetJobserver
}

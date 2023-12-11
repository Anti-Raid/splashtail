package execute_task

import (
	"encoding/json"
	"fmt"
	"splashtail/ipc/core"
	"splashtail/tasks"

	mredis "github.com/cheesycod/mewld/redis"
)

var ExecuteTask = core.IPC{
	Description:    "This IPC executes a task given a task and a task id and returns ok if successful. If you do not have both, consider create_task",
	SupportedModes: []core.IPCMode{core.IPCModeBot, core.IPCModeAPI},
	Exec: func(c *mredis.LauncherCmd) (*mredis.LauncherCmd, error) {
		if len(c.Args) == 0 {
			return nil, fmt.Errorf("no args provided to create task")
		}

		taskName, ok := c.Args["name"].(string)

		if !ok {
			return nil, fmt.Errorf("invalid task name provided")
		}

		taskId, ok := c.Args["task_id"].(string)

		if !ok {
			return nil, fmt.Errorf("invalid task id provided")
		}

		baseTaskDef, ok := tasks.TaskDefinitionRegistry[taskName]

		if !ok {
			return nil, fmt.Errorf("task %s does not exist on registry", taskName)
		}

		tBytes, err := json.Marshal(c.Args)

		if err != nil {
			return nil, fmt.Errorf("error marshalling task args: %w", err)
		}

		task := baseTaskDef // Copy task

		err = json.Unmarshal(tBytes, &task)

		if err != nil {
			return nil, fmt.Errorf("error unmarshalling task args: %w", err)
		}

		go tasks.ExecuteTask(taskId, task)

		return &mredis.LauncherCmd{
			Output: "ok",
		}, nil
	},
}

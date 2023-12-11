package create_task

import (
	"encoding/json"
	"fmt"
	"splashtail/ipc/core"
	"splashtail/state"
	"splashtail/tasks"

	mredis "github.com/cheesycod/mewld/redis"
)

var CreateTask = core.IPC{
	Description:    "This IPC creates a task and executes it. If you already have both a task and a task create response, consider execute_task",
	SupportedModes: []core.IPCMode{core.IPCModeBot, core.IPCModeAPI},
	Exec: func(c *mredis.LauncherCmd) (*mredis.LauncherCmd, error) {
		if len(c.Data) == 0 {
			return nil, fmt.Errorf("no data provided to create task")
		}

		taskName, ok := c.Args["name"].(string)

		if !ok {
			return nil, fmt.Errorf("invalid task name provided")
		}

		baseTaskDef, ok := tasks.TaskDefinitionRegistry[taskName]

		if !ok {
			return nil, fmt.Errorf("task %s does not exist on registry", taskName)
		}

		tBytes, err := json.Marshal(c.Output)

		if err != nil {
			return nil, fmt.Errorf("error marshalling task args: %w", err)
		}

		task := baseTaskDef // Copy task

		err = json.Unmarshal(tBytes, &task)

		if err != nil {
			return nil, fmt.Errorf("error unmarshalling task args: %w", err)
		}

		tcr, err := tasks.CreateTask(state.Context, task)

		if err != nil {
			return nil, fmt.Errorf("error creating task: %w", err)
		}

		go tasks.ExecuteTask(tcr.TaskID, task)

		return &mredis.LauncherCmd{
			Output: tcr,
		}, nil
	},
}

package create_task

import (
	"encoding/json"
	"fmt"

	"github.com/anti-raid/splashtail/jobserver/core"
	"github.com/anti-raid/splashtail/jobserver/core/taskexecutor"
	"github.com/anti-raid/splashtail/state"
	"github.com/anti-raid/splashtail/tasks"
)

var CreateTask = core.IPC{
	Description: "This IPC creates a task and executes it if the execute argument is set. If you already have both a task and a task create response, consider execute_task",
	Exec: func(client string, args map[string]any) (map[string]any, error) {
		taskName, ok := args["name"].(string)

		if !ok {
			return nil, fmt.Errorf("invalid task name provided")
		}

		baseTaskDef, ok := tasks.TaskDefinitionRegistry[taskName]

		if !ok {
			return nil, fmt.Errorf("task %s does not exist on registry", taskName)
		}

		data, ok := args["data"]

		if !ok {
			return nil, fmt.Errorf("invalid task data provided")
		}

		tBytes, err := json.Marshal(data)

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

		execute, _ := args["execute"].(bool)

		if execute {
			go taskexecutor.ExecuteTask(tcr.TaskID, task)
		}

		return map[string]any{
			"tcr": tcr,
		}, nil
	},
}

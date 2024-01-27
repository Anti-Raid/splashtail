package create_task

import (
	"encoding/json"
	"fmt"

	"github.com/anti-raid/splashtail/jobserver/endpoints"
	"github.com/anti-raid/splashtail/jobserver/jobrunner"
	"github.com/anti-raid/splashtail/jobserver/state"
	"github.com/anti-raid/splashtail/tasks"
)

var CreateTask = endpoints.IPC{
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

		// Validate task
		err = task.Validate(jobrunner.TaskState{})

		if err != nil {
			return nil, fmt.Errorf("failed to validate task: %w", err)
		}

		// Create task
		tcr, err := jobrunner.CreateTask(state.Context, state.Pool, task)

		if err != nil {
			return nil, fmt.Errorf("error creating task: %w", err)
		}

		execute, _ := args["execute"].(bool)

		if execute {
			go jobrunner.ExecuteTask(tcr.TaskID, task)
		}

		return map[string]any{
			"tcr": tcr,
		}, nil
	},
}

package execute_task

import (
	"encoding/json"
	"fmt"

	"github.com/anti-raid/splashtail/jobserver/core"
	"github.com/anti-raid/splashtail/jobserver/core/taskexecutor"
	"github.com/anti-raid/splashtail/tasks"
)

var ExecuteTask = core.IPC{
	Description: "This IPC executes a task given a task and a task id and returns ok if successful. If you do not have both, consider create_task",
	Exec: func(client string, args map[string]any) (map[string]any, error) {
		taskName, ok := args["name"].(string)

		if !ok {
			return nil, fmt.Errorf("invalid task name provided")
		}

		taskId, ok := args["task_id"].(string)

		if !ok {
			return nil, fmt.Errorf("invalid task id provided")
		}

		baseTaskDef, ok := tasks.TaskDefinitionRegistry[taskName]

		if !ok {
			return nil, fmt.Errorf("task %s does not exist on registry", taskName)
		}

		tBytes, err := json.Marshal(args)

		if err != nil {
			return nil, fmt.Errorf("error marshalling task args: %w", err)
		}

		task := baseTaskDef // Copy task

		err = json.Unmarshal(tBytes, &task)

		if err != nil {
			return nil, fmt.Errorf("error unmarshalling task args: %w", err)
		}

		err = task.Validate()

		if err != nil {
			return nil, fmt.Errorf("failed to validate task: %w", err)
		}
	
		tInfo := task.Info()
	
		if !tInfo.Valid {
			return nil, fmt.Errorf("invalid task info")
		}

		go taskexecutor.ExecuteTask(taskId, task)

		return nil, nil
	},
}

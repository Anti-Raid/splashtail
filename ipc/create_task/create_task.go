package create_task

import (
	"encoding/json"
	"fmt"
	"splashtail/state"
	"splashtail/tasks"

	mredis "github.com/cheesycod/mewld/redis"
)

func CreateTask(c *mredis.LauncherCmd) (*mredis.LauncherCmd, error) {
	if len(c.Data) == 0 {
		return nil, fmt.Errorf("no data provided to create task")
	}

	taskName, ok := c.Data["name"].(string)

	if !ok {
		return nil, fmt.Errorf("invalid task name provided")
	}

	task, ok := tasks.TaskRegistry[taskName]

	if !ok {
		return nil, fmt.Errorf("task %s does not exist on registry", taskName)
	}

	tBytes, err := json.Marshal(c.Args)

	if err != nil {
		return nil, fmt.Errorf("error marshalling task args: %w", err)
	}

	typ := task // Copy task

	err = json.Unmarshal(tBytes, &typ)

	if err != nil {
		return nil, fmt.Errorf("error unmarshalling task args: %w", err)
	}

	task, tcr, err := tasks.CreateTask(state.Context, typ, false)

	if err != nil {
		return nil, fmt.Errorf("error creating task: %w", err)
	}

	go tasks.NewTask(task)

	return &mredis.LauncherCmd{
		Output: tcr,
	}, nil
}

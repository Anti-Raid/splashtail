package get_task

import (
	"encoding/json"
	"errors"
	"fmt"
	"splashtail/api"
	"splashtail/db"
	"splashtail/state"
	"splashtail/types"
	"strings"

	mredis "github.com/cheesycod/mewld/redis"
	"github.com/jackc/pgx/v5"
)

var (
	taskColsArr = db.GetCols(types.Task{})
	taskColsStr = strings.Join(taskColsArr, ", ")
)

// @ci ipc=get_task
// @param target_id string - The target ID
// @param target_type string - The target type
// @param task string - JSON string of a TaskCreateResponse object returned from creating a task
// @param start_from int - The index to start from
func GetTask(c *mredis.LauncherCmd) (*mredis.LauncherCmd, error) {
	if len(c.Data) == 0 {
		return nil, fmt.Errorf("no data provided to get task")
	}

	targetId, ok := c.Data["target_id"].(string)

	if !ok {
		targetId = ""
	}

	targetType, ok := c.Data["target_type"].(string)

	if !ok {
		targetType = ""
	}

	taskStr, ok := c.Data["task"].(string)

	if !ok {
		return nil, fmt.Errorf("invalid task provided")
	}

	startFromF, ok := c.Data["start_from"].(float64)

	if !ok {
		startFromF = 0 // Start task
	}

	if startFromF < 0 {
		return nil, fmt.Errorf("invalid start_from provided")
	}

	startFrom := int(startFromF)

	var tcr *types.TaskCreateResponse

	err := json.Unmarshal([]byte(taskStr), &tcr)

	if err != nil {
		return nil, fmt.Errorf("error unmarshalling task: %w", err)
	}

	if tcr.TaskID == "" {
		return nil, fmt.Errorf("invalid task id provided")
	}

	// Delete expired tasks first
	_, err = state.Pool.Exec(state.Context, "DELETE FROM tasks WHERE created_at + expiry < NOW()")

	if err != nil {
		return nil, fmt.Errorf("error deleting expired tasks: %w", err)
	}

	row, err := state.Pool.Query(state.Context, "SELECT "+taskColsStr+" FROM tasks WHERE task_id = $1", tcr.TaskID)

	if err != nil {
		return nil, fmt.Errorf("error fetching task: %w", err)
	}

	task, err := pgx.CollectOneRow(row, pgx.RowToStructByName[types.Task])

	if errors.Is(err, pgx.ErrNoRows) {
		return nil, fmt.Errorf("task not found")
	}

	if err != nil {
		return nil, fmt.Errorf("error fetching task: %w", err)
	}

	if task.TaskKey != nil {
		if tcr.TaskKey == nil {
			return nil, fmt.Errorf("task key required")
		}

		if *task.TaskKey != *tcr.TaskKey {
			return nil, fmt.Errorf("invalid task key")
		}
	}

	if task.ForUser != nil {
		if targetId == "" || targetType == "" {
			return nil, fmt.Errorf("invalid target provided")
		}

		var forUserSplit = strings.Split(*task.ForUser, "/")

		if len(forUserSplit) != 2 {
			return nil, fmt.Errorf("invalid task.ForUser")
		}

		switch forUserSplit[0] {
		case "g":
			if targetType != api.TargetTypeServer {
				return nil, fmt.Errorf("this task is not owned by your server")
			}

			if forUserSplit[1] != targetId {
				return nil, fmt.Errorf("this task is not owned by your server")
			}
		case "u":
			if targetType != api.TargetTypeUser {
				return nil, fmt.Errorf("this task is not owned by your user account")
			}

			if forUserSplit[1] != targetId {
				return nil, fmt.Errorf("this task is not owned by your user account")
			}
		default:
			return nil, fmt.Errorf("invalid task.ForUser")
		}
	}

	if startFrom != 0 {
		if startFrom >= len(task.Statuses) {
			return nil, fmt.Errorf("invalid start_from provided")
		}

		// trim down statuses sent to only whats actually needed
		task.Statuses = task.Statuses[startFrom:]
	}

	return &mredis.LauncherCmd{
		Output: task,
	}, nil
}

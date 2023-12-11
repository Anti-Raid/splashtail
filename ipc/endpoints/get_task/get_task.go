package get_task

import (
	"encoding/json"
	"errors"
	"fmt"
	"splashtail/db"
	"splashtail/ipc/core"
	"splashtail/state"
	"splashtail/tasks"
	"splashtail/types"
	"strings"

	mredis "github.com/cheesycod/mewld/redis"
	"github.com/jackc/pgx/v5"
)

var (
	taskColsArr = db.GetCols(types.Task{})
	taskColsStr = strings.Join(taskColsArr, ", ")
)

var GetTask = core.IPC{
	Deprecated: `
get_task IPC should not be used as it is extremely costly and impedes on running tasks. 

Instead individual clients should manage this themselves. Mainly applicable to the bot as the API already does this.`,
	SupportedModes: []core.IPCMode{core.IPCModeBot},
	// @ci ipc=get_task
	// @param target_id string - The target ID
	// @param target_type string - The target type
	// @param task string - JSON string of a TaskCreateResponse object returned from creating a task
	// @param start_from int - The index to start from
	Exec: func(c *mredis.LauncherCmd) (*mredis.LauncherCmd, error) {
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
			return nil, fmt.Errorf("start_from must be greater than zero")
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

		if task.TaskForRaw != nil {
			task.TaskFor = tasks.ParseTaskFor(*task.TaskForRaw)

			if task.TaskFor == nil {
				return nil, fmt.Errorf("invalid task.TaskFor [task.TaskFor was parsed to nil]")
			}

			if task.TaskFor.ID == "" || task.TaskFor.TargetType == "" {
				return nil, fmt.Errorf("invalid task.TaskFor")
			}

			if task.TaskFor.TargetType != targetType {
				return nil, fmt.Errorf("this task is meant for '%s' but you are a '%s'", task.TaskFor.TargetType, targetType)
			}

			if task.TaskFor.ID != targetId {
				return nil, fmt.Errorf("you are not authorized to view this task")
			}
		}

		if startFrom != 0 {
			if startFrom > len(task.Statuses) {
				return nil, fmt.Errorf("start_from must be less than or equal to the length of the statuses array")
			}

			// trim down statuses sent to only whats actually needed
			task.Statuses = task.Statuses[startFrom:]
		}

		return &mredis.LauncherCmd{
			Output: task,
		}, nil
	},
}

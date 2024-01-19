package tasks

import (
	"context"
	"fmt"
	"time"

	"github.com/anti-raid/splashtail/state"
	"github.com/anti-raid/splashtail/types"

	"go.uber.org/zap"
)

// Task management core
var TaskDefinitionRegistry = map[string]TaskDefinition{}

func RegisterTaskDefinition(task TaskDefinition) {
	TaskDefinitionRegistry[task.Info().Name] = task
}

// TaskDefinition is the definition for any task that can be executed on splashtail
type TaskDefinition interface {
	// Validate validates the task and sets up state if needed
	Validate() error

	// Exec executes the task returning an output if any
	Exec(l *zap.Logger, tcr *types.TaskCreateResponse) (*types.TaskOutput, error)

	// Returns the info on a task
	Info() *types.TaskInfo
}

// Sets up a task
func CreateTask(ctx context.Context, task TaskDefinition) (*types.TaskCreateResponse, error) {
	err := task.Validate()

	if err != nil {
		return nil, fmt.Errorf("failed to validate task: %w", err)
	}

	tInfo := task.Info()

	if !tInfo.Valid {
		return nil, fmt.Errorf("invalid task info")
	}

	_, ok := TaskDefinitionRegistry[tInfo.Name]

	if !ok {
		return nil, fmt.Errorf("task %s does not exist on registry", tInfo.Name)
	}

	var taskId string

	var taskFor *string

	if tInfo.TaskFor != nil {
		taskFor, err = FormatTaskFor(tInfo.TaskFor)

		if err != nil {
			return nil, err
		}
	}

	err = state.Pool.QueryRow(ctx, "INSERT INTO tasks (task_name, task_for, expiry, output, task_info) VALUES ($1, $2, $3, $4, $5) RETURNING task_id",
		tInfo.Name,
		taskFor,
		func() *time.Duration {
			if tInfo.Expiry == 0 {
				return nil
			}

			return &tInfo.Expiry
		}(),
		nil,
		tInfo,
	).Scan(&taskId)

	if err != nil {
		return nil, fmt.Errorf("failed to create task: %w", err)
	}

	return &types.TaskCreateResponse{
		TaskID:   taskId,
		TaskInfo: tInfo,
	}, nil
}

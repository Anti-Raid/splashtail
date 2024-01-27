package jobrunner

import (
	"context"
	"fmt"
	"time"

	"github.com/anti-raid/splashtail/tasks"
	"github.com/anti-raid/splashtail/types"
	"github.com/jackc/pgx/v5/pgxpool"
)

// Sets up a task
func CreateTask(ctx context.Context, pool *pgxpool.Pool, task tasks.TaskDefinition) (*types.TaskCreateResponse, error) {
	tInfo := task.Info()

	if !tInfo.Valid {
		return nil, fmt.Errorf("invalid task info")
	}

	_, ok := tasks.TaskDefinitionRegistry[tInfo.Name]

	if !ok {
		return nil, fmt.Errorf("task %s does not exist on registry", tInfo.Name)
	}

	var taskId string
	var taskFor *string
	var err error

	if tInfo.TaskFor != nil {
		taskFor, err = tasks.FormatTaskFor(tInfo.TaskFor)

		if err != nil {
			return nil, err
		}
	}

	err = pool.QueryRow(ctx, "INSERT INTO tasks (task_name, task_for, expiry, output, task_info) VALUES ($1, $2, $3, $4, $5) RETURNING task_id",
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

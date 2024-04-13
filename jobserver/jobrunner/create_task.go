package jobrunner

import (
	"context"
	"fmt"

	"github.com/anti-raid/splashtail/tasks"
	"github.com/anti-raid/splashtail/tasks/taskdef"
	"github.com/jackc/pgx/v5/pgxpool"
)

// Sets up a task
func CreateTask(ctx context.Context, pool *pgxpool.Pool, task taskdef.TaskDefinition) (*string, error) {
	taskName := task.Name()
	taskFor := task.TaskFor()

	_, ok := tasks.TaskDefinitionRegistry[task.Name()]

	if !ok {
		return nil, fmt.Errorf("task %s does not exist on registry", task.Name())
	}

	var taskId string

	tx, err := pool.Begin(ctx)

	if err != nil {
		return nil, fmt.Errorf("failed to start transaction: %w", err)
	}

	defer tx.Rollback(ctx)

	taskForStr, err := tasks.FormatTaskFor(taskFor)

	if err != nil {
		return nil, fmt.Errorf("failed to format task_for: %w", err)
	}

	err = tx.QueryRow(ctx, "INSERT INTO tasks (task_name, task_for, expiry, output, task_fields, resumable) VALUES ($1, $2, $3, $4, $5, $6) RETURNING task_id",
		taskName,
		taskForStr,
		task.Expiry(),
		nil,
		task.TaskFields(),
		task.Resumable(),
	).Scan(&taskId)

	if err != nil {
		return nil, fmt.Errorf("failed to create task: %w", err)
	}

	// Add to ongoing_tasks
	_, err = tx.Exec(
		ctx,
		"INSERT INTO ongoing_tasks (task_id, data, initial_opts) VALUES ($1, $2, $3)",
		taskId,
		map[string]any{},
		task,
	)

	if err != nil {
		return nil, fmt.Errorf("failed to add task to ongoing_tasks: %w", err)
	}

	err = tx.Commit(ctx)

	if err != nil {
		return nil, fmt.Errorf("failed to commit transaction: %w", err)
	}

	return &taskId, nil
}

package jobrunner

import (
	"context"
	"fmt"

	"github.com/jackc/pgx/v5/pgxpool"
	jobs "go.jobs"
	"go.jobs/interfaces"
)

// Sets up a task
func CreateTask(ctx context.Context, pool *pgxpool.Pool, jobImpl interfaces.JobImpl) (*string, error) {
	name := jobImpl.Name()
	taskFor := jobImpl.TaskFor()

	_, ok := jobs.JobImplRegistry[jobImpl.Name()]

	if !ok {
		return nil, fmt.Errorf("job %s does not exist on registry", jobImpl.Name())
	}

	var taskId string

	tx, err := pool.Begin(ctx)

	if err != nil {
		return nil, fmt.Errorf("failed to start transaction: %w", err)
	}

	//nolint:errcheck
	defer tx.Rollback(ctx)

	taskForStr, err := jobs.FormatTaskFor(taskFor)

	if err != nil {
		return nil, fmt.Errorf("failed to format task_for: %w", err)
	}

	err = tx.QueryRow(ctx, "INSERT INTO tasks (name, task_for, expiry, output, fields, resumable) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
		name,
		taskForStr,
		jobImpl.Expiry(),
		nil,
		jobImpl.Fields(),
		jobImpl.Resumable(),
	).Scan(&taskId)

	if err != nil {
		return nil, fmt.Errorf("failed to create job: %w", err)
	}

	// Add to ongoing_jobs
	_, err = tx.Exec(
		ctx,
		"INSERT INTO ongoing_jobs (id, data, initial_opts) VALUES ($1, $2, $3)",
		taskId,
		map[string]any{},
		jobImpl,
	)

	if err != nil {
		return nil, fmt.Errorf("failed to add job to ongoing_jobs: %w", err)
	}

	err = tx.Commit(ctx)

	if err != nil {
		return nil, fmt.Errorf("failed to commit transaction: %w", err)
	}

	return &taskId, nil
}

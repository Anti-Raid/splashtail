package core

import (
	"context"
	"errors"
	"fmt"
	"strings"
	"time"

	"github.com/infinitybotlist/eureka/jsonimpl"
	"github.com/jackc/pgx/v5"
	jobs "go.jobs"
	"go.jobs/types"
	"go.jobserver/jobrunner"
	"go.jobserver/rpc_messages"
	"go.jobserver/state"
	"go.std/splashcore"
	"go.std/structparser/db"
	"go.std/utils/mewext"
	"go.uber.org/zap"
)

var DefaultTimeout = 30 * time.Minute
var ResumeOngoingTaskTimeoutSecs = 15 * 60
var DefaultValidationTimeout = 5 * time.Second

var (
	taskCols    = db.GetCols(types.Task{})
	taskColsStr = strings.Join(taskCols, ", ")
)

func SpawnTask(spawnTask rpc_messages.SpawnTask) (*rpc_messages.SpawnTaskResponse, error) {
	defer func() {
		if rvr := recover(); rvr != nil {
			fmt.Println("Recovered from panic:", rvr)
		}
	}()

	if !spawnTask.Create && !spawnTask.Execute {
		return nil, fmt.Errorf("either create or execute must be set")
	}

	if spawnTask.Name == "" {
		return nil, fmt.Errorf("invalid job name provided")
	}

	baseJobImpl, ok := jobs.JobImplRegistry[spawnTask.Name]

	if !ok {
		return nil, fmt.Errorf("job %s does not exist on registry", spawnTask.Name)
	}

	if len(spawnTask.Data) == 0 {
		return nil, fmt.Errorf("invalid task data provided")
	}

	tBytes, err := jsonimpl.Marshal(spawnTask.Data)

	if err != nil {
		return nil, fmt.Errorf("error marshalling task args: %w", err)
	}

	task := baseJobImpl // Copy task

	err = jsonimpl.Unmarshal(tBytes, &task)

	if err != nil {
		return nil, fmt.Errorf("error unmarshalling task args: %w", err)
	}

	taskFor := task.TaskFor()

	// Check if task pertains to this clusters shard
	if taskFor.TargetType == splashcore.TargetTypeUser && state.Shard != 0 {
		return nil, fmt.Errorf("task is not for this shard [user tasks must run on shard 0]")
	} else {
		taskShard, err := mewext.GetShardIDFromGuildID(taskFor.ID, int(state.ShardCount))

		if err != nil {
			state.Logger.Error("Failed to get shard id from guild id", zap.Error(err))
			return nil, fmt.Errorf("failed to get shard id from guild id: %w", err)
		}

		// This case should work until we reach 65 million servers
		if uint16(taskShard) != state.Shard {
			return nil, fmt.Errorf("task is not for this shard [task shard: %d, this shard: %d]", taskShard, state.Shard)
		}
	}

	// Validate task
	ctx, cancel := context.WithTimeout(state.Context, DefaultValidationTimeout)
	defer cancel()
	err = task.Validate(jobrunner.TaskState{
		Ctx: ctx,
	})

	if err != nil {
		return nil, fmt.Errorf("failed to validate task: %w", err)
	}

	// Create task
	var taskId string
	if spawnTask.Create {
		tid, err := jobrunner.CreateTask(state.Context, state.Pool, task)

		if err != nil {
			return nil, fmt.Errorf("error creating task: %w", err)
		}

		taskId = *tid
	} else {
		if spawnTask.ID == "" {
			return nil, fmt.Errorf("task id must be set if SpawnTask.Create is false")
		}

		taskId = spawnTask.ID
	}

	// Execute task
	if spawnTask.Execute {
		ctx, cancel := context.WithTimeout(state.Context, DefaultTimeout)
		go jobrunner.ExecuteTask(ctx, cancel, taskId, task, nil)
	}

	return &rpc_messages.SpawnTaskResponse{
		ID: taskId,
	}, nil
}

func Resume() {
	state.Logger.Info("Deleting ancient ongoing jobs older than ResumeOngoingTaskTimeout", zap.Int("timeout", ResumeOngoingTaskTimeoutSecs))

	_, err := state.Pool.Exec(state.Context, "DELETE FROM ongoing_jobs WHERE created_at < NOW() - make_interval(secs => $1)", ResumeOngoingTaskTimeoutSecs)

	if err != nil {
		state.Logger.Error("Failed to delete ancient ongoing_jobs", zap.Error(err))
		panic("Failed to delete ancient ongoing_jobs")
	}

	state.Logger.Info("Looking for tasks to resume")

	var taskId string
	var data map[string]any
	var initialOpts map[string]any
	var createdAt time.Time

	rows, err := state.Pool.Query(state.Context, "SELECT id, data, initial_opts, created_at FROM ongoing_jobs")

	if err != nil {
		state.Logger.Error("Failed to query tasks", zap.Error(err))
		panic("Failed to query tasks")
	}

	defer rows.Close()

	for rows.Next() {
		err = rows.Scan(&taskId, &data, &initialOpts, &createdAt)

		if err != nil {
			state.Logger.Error("Failed to scan task", zap.Error(err))
			panic("Failed to scan task")
		}

		// Select the task from the task db
		row, err := state.Pool.Query(state.Context, "SELECT "+taskColsStr+" FROM tasks WHERE id = $1", taskId)

		if errors.Is(err, pgx.ErrNoRows) {
			state.Logger.Error("Task not found", zap.String("id", taskId))
			continue
		}

		if err != nil {
			state.Logger.Error("Failed to query task", zap.Error(err))
			continue
		}

		defer row.Close()

		t, err := pgx.CollectOneRow(row, pgx.RowToAddrOfStructByName[types.Task])

		if errors.Is(err, pgx.ErrNoRows) {
			state.Logger.Error("Task not found", zap.String("id", taskId))
			continue
		}

		if err != nil {
			state.Logger.Error("Failed to collect task", zap.Error(err))
			continue
		}

		if !t.Resumable {
			continue
		}

		if t.State == "completed" || t.State == "failed" {
			continue
		}

		baseJobImpl, ok := jobs.JobImplRegistry[t.Name]

		if !ok {
			state.Logger.Error("Task not found in registry", zap.String("id", taskId))
			continue
		}

		tBytes, err := jsonimpl.Marshal(initialOpts)

		if err != nil {
			state.Logger.Error("Failed to marshal task create opts", zap.Error(err))
			continue
		}

		task := baseJobImpl // Copy task

		err = jsonimpl.Unmarshal(tBytes, &task)

		if err != nil {
			state.Logger.Error("Failed to unmarshal task create opts", zap.Error(err))
			continue
		}

		taskFor := task.TaskFor()

		// Check if task pertains to this clusters shard
		if taskFor.TargetType == splashcore.TargetTypeUser && state.Shard != 0 {
			continue
		} else {
			taskShard, err := mewext.GetShardIDFromGuildID(taskFor.ID, int(state.ShardCount))

			if err != nil {
				state.Logger.Error("Failed to get shard id from guild id", zap.Error(err))
				continue
			}

			// This case should work until we reach 65 million servers
			if uint16(taskShard) != state.Shard {
				continue
			}
		}

		// Validate task
		ctx, cancel := context.WithTimeout(state.Context, DefaultValidationTimeout)

		err = task.Validate(jobrunner.TaskState{
			Ctx: ctx,
		})

		cancel()

		if err != nil {
			state.Logger.Error("Failed to validate task", zap.Error(err))
			continue
		}

		// Execute task
		ctx, cancel = context.WithTimeout(state.Context, DefaultTimeout)

		go jobrunner.ExecuteTask(ctx, cancel, taskId, task, nil)
	}
}

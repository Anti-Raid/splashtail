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

func Spawn(spawn rpc_messages.SpawnTask) (*rpc_messages.SpawnTaskResponse, error) {
	defer func() {
		if rvr := recover(); rvr != nil {
			fmt.Println("Recovered from panic:", rvr)
		}
	}()

	if !spawn.Create && !spawn.Execute {
		return nil, fmt.Errorf("either create or execute must be set")
	}

	if spawn.Name == "" {
		return nil, fmt.Errorf("invalid job name provided")
	}

	baseJobImpl, ok := jobs.JobImplRegistry[spawn.Name]

	if !ok {
		return nil, fmt.Errorf("job %s does not exist on registry", spawn.Name)
	}

	if len(spawn.Data) == 0 {
		return nil, fmt.Errorf("invalid task data provided")
	}

	b, err := jsonimpl.Marshal(spawn.Data)

	if err != nil {
		return nil, fmt.Errorf("error marshalling task args: %w", err)
	}

	job := baseJobImpl // Copy task

	err = jsonimpl.Unmarshal(b, &job)

	if err != nil {
		return nil, fmt.Errorf("error unmarshalling task args: %w", err)
	}

	owner := job.Owner()

	// Check if task pertains to this clusters shard
	if owner.TargetType == splashcore.TargetTypeUser && state.Shard != 0 {
		return nil, fmt.Errorf("task is not for this shard [user tasks must run on shard 0]")
	} else {
		taskShard, err := mewext.GetShardIDFromGuildID(owner.ID, int(state.ShardCount))

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
	err = job.Validate(jobrunner.State{
		Ctx: ctx,
	})

	if err != nil {
		return nil, fmt.Errorf("failed to validate job: %w", err)
	}

	// Create task
	var id string
	if spawn.Create {
		tid, err := jobrunner.Create(state.Context, state.Pool, job)

		if err != nil {
			return nil, fmt.Errorf("error creating task: %w", err)
		}

		id = *tid
	} else {
		if spawn.ID == "" {
			return nil, fmt.Errorf("task id must be set if spawn.Create is false")
		}

		id = spawn.ID
	}

	// Execute task
	if spawn.Execute {
		ctx, cancel := context.WithTimeout(state.Context, DefaultTimeout)
		go jobrunner.Execute(ctx, cancel, id, job, nil)
	}

	return &rpc_messages.SpawnTaskResponse{
		ID: id,
	}, nil
}

func Resume() {
	state.Logger.Info("Deleting ancient ongoing jobs older than ResumeOngoingTaskTimeout", zap.Int("timeout", ResumeOngoingTaskTimeoutSecs))

	_, err := state.Pool.Exec(state.Context, "DELETE FROM ongoing_jobs WHERE created_at < NOW() - make_interval(secs => $1)", ResumeOngoingTaskTimeoutSecs)

	if err != nil {
		state.Logger.Error("Failed to delete ancient ongoing_jobs", zap.Error(err))
		panic("Failed to delete ancient ongoing_jobs")
	}

	state.Logger.Info("Looking for jobs to resume")

	var id string
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
		err = rows.Scan(&id, &data, &initialOpts, &createdAt)

		if err != nil {
			state.Logger.Error("Failed to scan task", zap.Error(err))
			panic("Failed to scan task")
		}

		// Select the task from the task db
		row, err := state.Pool.Query(state.Context, "SELECT "+taskColsStr+" FROM tasks WHERE id = $1", id)

		if errors.Is(err, pgx.ErrNoRows) {
			state.Logger.Error("Task not found", zap.String("id", id))
			continue
		}

		if err != nil {
			state.Logger.Error("Failed to query task", zap.Error(err))
			continue
		}

		defer row.Close()

		t, err := pgx.CollectOneRow(row, pgx.RowToAddrOfStructByName[types.Task])

		if errors.Is(err, pgx.ErrNoRows) {
			state.Logger.Error("Task not found", zap.String("id", id))
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
			state.Logger.Error("Task not found in registry", zap.String("id", id))
			continue
		}

		b, err := jsonimpl.Marshal(initialOpts)

		if err != nil {
			state.Logger.Error("Failed to marshal job create opts", zap.Error(err))
			continue
		}

		job := baseJobImpl // Copy job

		err = jsonimpl.Unmarshal(b, &job)

		if err != nil {
			state.Logger.Error("Failed to unmarshal job create opts", zap.Error(err))
			continue
		}

		owner := job.Owner()

		// Check if task pertains to this clusters shard
		if owner.TargetType == splashcore.TargetTypeUser && state.Shard != 0 {
			continue
		} else {
			taskShard, err := mewext.GetShardIDFromGuildID(owner.ID, int(state.ShardCount))

			if err != nil {
				state.Logger.Error("Failed to get shard id from guild id", zap.Error(err))
				continue
			}

			// This case should work until we reach 65 million servers
			if uint16(taskShard) != state.Shard {
				continue
			}
		}

		// Validate
		ctx, cancel := context.WithTimeout(state.Context, DefaultValidationTimeout)

		err = job.Validate(jobrunner.State{
			Ctx: ctx,
		})

		cancel()

		if err != nil {
			state.Logger.Error("Failed to validate task", zap.Error(err))
			continue
		}

		// Execute task
		ctx, cancel = context.WithTimeout(state.Context, DefaultTimeout)

		go jobrunner.Execute(ctx, cancel, id, job, nil)
	}
}

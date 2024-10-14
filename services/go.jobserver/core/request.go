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
	"go.std/structparser/db"
	"go.uber.org/zap"
)

var DefaultTimeout = 30 * time.Minute
var ResumeOngoingJobTimeoutSecs = 15 * 60
var DefaultValidationTimeout = 5 * time.Second

var (
	jobCols    = db.GetCols(types.Job{})
	jobColsStr = strings.Join(jobCols, ", ")
)

func Spawn(spawn rpc_messages.Spawn) (*rpc_messages.SpawnResponse, error) {
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
		return nil, fmt.Errorf("invalid job data provided")
	}

	b, err := jsonimpl.Marshal(spawn.Data)

	if err != nil {
		return nil, fmt.Errorf("error marshalling args: %w", err)
	}

	job := baseJobImpl // Copy job

	err = jsonimpl.Unmarshal(b, &job)

	if err != nil {
		return nil, fmt.Errorf("error unmarshalling args: %w", err)
	}

	// Validate
	ctx, cancel := context.WithTimeout(state.Context, DefaultValidationTimeout)
	defer cancel()
	err = job.Validate(jobrunner.State{
		Ctx: ctx,
	})

	if err != nil {
		return nil, fmt.Errorf("failed to validate job: %w", err)
	}

	// Create
	var id string
	if spawn.Create {
		tid, err := jobrunner.Create(state.Context, state.Pool, job)

		if err != nil {
			return nil, fmt.Errorf("error creating job: %w", err)
		}

		id = *tid
	} else {
		if spawn.ID == "" {
			return nil, fmt.Errorf("id must be set if spawn.Create is false")
		}

		id = spawn.ID
	}

	// Execute
	if spawn.Execute {
		ctx, cancel := context.WithTimeout(state.Context, DefaultTimeout)
		go jobrunner.Execute(ctx, cancel, id, job, nil)
	}

	return &rpc_messages.SpawnResponse{
		ID: id,
	}, nil
}

func Resume() {
	state.Logger.Info("Deleting ancient ongoing jobs older than ResumeOngoingJobTimeoutSecs", zap.Int("timeout", ResumeOngoingJobTimeoutSecs))

	_, err := state.Pool.Exec(state.Context, "DELETE FROM ongoing_jobs WHERE created_at < NOW() - make_interval(secs => $1)", ResumeOngoingJobTimeoutSecs)

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
		state.Logger.Error("Failed to query ongoing_jobs", zap.Error(err))
		panic("Failed to query ongoing_jobs")
	}

	defer rows.Close()

	for rows.Next() {
		err = rows.Scan(&id, &data, &initialOpts, &createdAt)

		if err != nil {
			state.Logger.Error("Failed to scan jobs", zap.Error(err))
			panic("Failed to scan jobs")
		}

		// Select the job from the job db
		row, err := state.Pool.Query(state.Context, "SELECT "+jobColsStr+" FROM jobs WHERE id = $1", id)

		if errors.Is(err, pgx.ErrNoRows) {
			state.Logger.Error("Job not found", zap.String("id", id))
			continue
		}

		if err != nil {
			state.Logger.Error("Failed to query job", zap.Error(err))
			continue
		}

		defer row.Close()

		t, err := pgx.CollectOneRow(row, pgx.RowToAddrOfStructByName[types.Job])

		if errors.Is(err, pgx.ErrNoRows) {
			state.Logger.Error("Job not found", zap.String("id", id))
			continue
		}

		if err != nil {
			state.Logger.Error("Failed to collect job", zap.Error(err))
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

		// Validate
		ctx, cancel := context.WithTimeout(state.Context, DefaultValidationTimeout)

		err = job.Validate(jobrunner.State{
			Ctx: ctx,
		})

		cancel()

		if err != nil {
			state.Logger.Error("Failed to validate job", zap.Error(err))
			continue
		}

		// Execute job
		ctx, cancel = context.WithTimeout(state.Context, DefaultTimeout)

		go jobrunner.Execute(ctx, cancel, id, job, nil)
	}
}

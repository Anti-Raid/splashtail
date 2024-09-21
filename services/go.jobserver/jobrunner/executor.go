// Package executor defines a "production-ready" executor for jobs.
//
// For local/non-production use, consider looking at cmd/localjobs's executor
package jobrunner

import (
	"context"
	"net/http"
	"runtime/debug"

	"github.com/bwmarrin/discordgo"
	"github.com/infinitybotlist/eureka/crypto"
	jobs "go.jobs"
	"go.jobs/interfaces"
	jobstate "go.jobs/state"
	"go.jobserver/state"
	"go.uber.org/zap"
)

// PersistState persists the state to redis temporarily
func PersistState(tc *Progress, prog *jobstate.Progress) error {
	_, err := state.Pool.Exec(
		tc.State.Context(),
		"UPDATE ongoing_jobs SET state = $2, data = $3 WHERE id = $1",
		tc.ID,
		prog.State,
		prog.Data,
	)

	if err != nil {
		return err
	}

	return nil
}

// GetPersistedState gets persisted state from redis
func GetPersistedState(tc *Progress) (*jobstate.Progress, error) {
	var s string
	var data map[string]any

	err := state.Pool.QueryRow(tc.State.Context(), "SELECT state, data FROM ongoing_jobs WHERE id = $1", tc.ID).Scan(&s, &data)

	if err != nil {
		return nil, err
	}

	return &jobstate.Progress{
		State: s,
		Data:  data,
	}, nil
}

// Implementor of jobs.State
type State struct {
	Ctx context.Context
}

func (State) Transport() *http.Transport {
	return state.Transport
}

func (State) OperationMode() string {
	return state.CurrentOperationMode
}

func (State) Discord() (*discordgo.Session, *discordgo.User, bool) {
	return state.Discord, state.BotUser, false
}

func (State) DebugInfo() *debug.BuildInfo {
	return state.BuildInfo
}

func (t State) Context() context.Context {
	return t.Ctx
}

type Progress struct {
	ID string

	State State

	// Used to cache the current progress in resumes
	//
	// When resuming, set this to the current progress
	CurrentProgress *jobstate.Progress

	// OnSetProgress is a callback that is called when SetProgress is called
	//
	// If unset, calls PersistState
	OnSetProgress func(tc *Progress, prog *jobstate.Progress) error
}

func (ts Progress) GetProgress() (*jobstate.Progress, error) {
	if ts.CurrentProgress == nil {
		return GetPersistedState(&ts)
	}

	return ts.CurrentProgress, nil
}

func (ts Progress) SetProgress(prog *jobstate.Progress) error {
	ts.CurrentProgress = prog

	if ts.OnSetProgress != nil {
		err := ts.OnSetProgress(&ts, prog)

		if err != nil {
			return err
		}
	} else {
		err := PersistState(&ts, prog)

		if err != nil {
			return err
		}
	}

	return nil
}

// Creates a new job on server and executes it
//
// If prog is set, it will be used to cache the progress, otherwise a blank one will be used
func Execute(
	ctx context.Context,
	ctxCancel context.CancelFunc,
	id string,
	jobImpl interfaces.JobImpl,
	prog *Progress,
) {
	if state.CurrentOperationMode != "jobs" {
		panic("cannot execute jobs outside of job server")
	}

	l, _ := NewTaskLogger(id, state.Pool, ctx, state.Logger)
	erl, _ := NewTaskLogger(id, state.Pool, state.Context, state.Logger)

	var done bool
	var bChan = make(chan int) // bChan is a channel thats used to control the canceller channel

	// Fail failed jobs
	defer func() {
		err := recover()

		if err != nil {
			erl.Error("Panic", zap.Any("err", err))
			state.Logger.Error("Panic", zap.Any("err", err))

			_, err := state.Pool.Exec(state.Context, "UPDATE jobs SET state = $1 WHERE id = $2", "failed", id)

			if err != nil {
				l.Error("Failed to update job", zap.Error(err))
			}
		}

		if !done {
			_, err := state.Pool.Exec(state.Context, "UPDATE jobs SET state = $1 WHERE id = $2", "failed", id)

			if err != nil {
				l.Error("Failed to update job", zap.Error(err))
			}
		}

		if ctxCancel != nil {
			defer ctxCancel()
		}

		_, err2 := state.Pool.Exec(state.Context, "DELETE FROM ongoing_jobs WHERE id = $1", id)

		if err != nil {
			l.Error("Failed to delete job from ongoing jobs", zap.Error(err2))
			return
		}

		bChan <- 1
	}()

	go func() {
		for {
			select {
			case <-bChan:
				return
			case <-ctx.Done():
				erl.Error("Context done, timeout?")
				done = true
				return
			}
		}
	}()

	// Set state to running
	_, err := state.Pool.Exec(state.Context, "UPDATE jobs SET state = $1 WHERE id = $2", "running", id)

	if err != nil {
		l.Error("Failed to update job", zap.Error(err))
		return
	}

	ts := State{
		Ctx: ctx,
	}
	if prog == nil {
		prog = &Progress{
			ID:    id,
			State: ts,
		}
	}

	var currState = "completed"

	outp, terr := jobImpl.Exec(l, ts, prog)

	if terr != nil {
		l.Error("Failed to execute job [terr != nil]", zap.Error(terr))
		currState = "failed"
	}

	// Save output to object storage
	if outp != nil {
		if outp.Filename == "" {
			outp.Filename = "unnamed." + crypto.RandString(16)
		}

		if outp.Buffer == nil {
			l.Error("Job output buffer is nil")
			currState = "failed"
		} else {
			l.Info("Saving job output", zap.String("filename", outp.Filename))

			err = state.ObjectStorage.Save(
				state.Context,
				jobs.GetPathFromOutput(id, jobImpl, outp),
				outp.Filename,
				outp.Buffer,
				0,
			)

			if err != nil {
				l.Error("Failed to save backup", zap.Error(err))
				return
			}
		}
	}

	_, err = state.Pool.Exec(state.Context, "UPDATE jobs SET output = $1, state = $2 WHERE id = $3", outp, currState, id)

	if err != nil {
		l.Error("Failed to update job", zap.Error(err))
		return
	}

	done = true
}

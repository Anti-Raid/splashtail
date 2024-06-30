// Package taskexecutor defines a "production-ready" task executor.
//
// For local/non-production use, consider looking at cmd/localjobs's task executor
package jobrunner

import (
	"context"
	"net/http"
	"runtime/debug"

	jobs "github.com/anti-raid/splashtail/core/go.jobs"
	"github.com/anti-raid/splashtail/core/go.jobs/taskdef"
	"github.com/anti-raid/splashtail/core/go.jobs/taskstate"
	"github.com/anti-raid/splashtail/services/go.jobserver/state"
	"github.com/bwmarrin/discordgo"
	"github.com/infinitybotlist/eureka/crypto"
	"go.uber.org/zap"
)

// PersistTaskState persists task state to redis temporarily
func PersistTaskState(tc *TaskProgress, prog *taskstate.Progress) error {
	_, err := state.Pool.Exec(
		tc.TaskState.Context(),
		"UPDATE ongoing_tasks SET state = $2, data = $3 WHERE task_id = $1",
		tc.TaskID,
		prog.State,
		prog.Data,
	)

	if err != nil {
		return err
	}

	return nil
}

// GetPersistedTaskState gets persisted task state from redis
func GetPersistedTaskState(tc *TaskProgress) (*taskstate.Progress, error) {
	var s string
	var data map[string]any

	err := state.Pool.QueryRow(tc.TaskState.Context(), "SELECT state, data FROM ongoing_tasks WHERE task_id = $1", tc.TaskID).Scan(&s, &data)

	if err != nil {
		return nil, err
	}

	return &taskstate.Progress{
		State: s,
		Data:  data,
	}, nil
}

// Implementor of jobs.TaskState
type TaskState struct {
	Ctx context.Context
}

func (TaskState) Transport() *http.Transport {
	return state.TaskTransport
}

func (TaskState) OperationMode() string {
	return state.CurrentOperationMode
}

func (TaskState) Discord() (*discordgo.Session, *discordgo.User, bool) {
	return state.Discord, state.BotUser, false
}

func (TaskState) DebugInfo() *debug.BuildInfo {
	return state.BuildInfo
}

func (t TaskState) Context() context.Context {
	return t.Ctx
}

type TaskProgress struct {
	TaskID string

	TaskState TaskState

	// Used to cache the current task progress in resumes
	//
	// When resuming, set this to the current progress
	CurrentTaskProgress *taskstate.Progress

	// OnSetProgress is a callback that is called when SetProgress is called
	//
	// If unset, calls PersistTaskState
	OnSetProgress func(tc *TaskProgress, prog *taskstate.Progress) error
}

func (ts TaskProgress) GetProgress() (*taskstate.Progress, error) {
	if ts.CurrentTaskProgress == nil {
		return GetPersistedTaskState(&ts)
	}

	return ts.CurrentTaskProgress, nil
}

func (ts TaskProgress) SetProgress(prog *taskstate.Progress) error {
	ts.CurrentTaskProgress = prog

	if ts.OnSetProgress != nil {
		err := ts.OnSetProgress(&ts, prog)

		if err != nil {
			return err
		}
	} else {
		err := PersistTaskState(&ts, prog)

		if err != nil {
			return err
		}
	}

	return nil
}

// Creates a new task on server and executes it
//
// If prog is set, it will be used to cache the task progress, otherwise a blank one will be used
func ExecuteTask(
	ctx context.Context,
	ctxCancel context.CancelFunc,
	taskId string,
	task taskdef.TaskDefinition,
	prog *TaskProgress,
) {
	if state.CurrentOperationMode != "jobs" {
		panic("cannot execute task outside of job server")
	}

	l, _ := NewTaskLogger(taskId, state.Pool, ctx, state.Logger)
	erl, _ := NewTaskLogger(taskId, state.Pool, state.Context, state.Logger)

	var done bool
	var bChan = make(chan int) // bChan is a channel thats used to control the canceller channel

	// Fail failed tasks
	defer func() {
		err := recover()

		if err != nil {
			erl.Error("Panic", zap.Any("err", err))
			state.Logger.Error("Panic", zap.Any("err", err))

			_, err := state.Pool.Exec(state.Context, "UPDATE tasks SET state = $1 WHERE task_id = $2", "failed", taskId)

			if err != nil {
				l.Error("Failed to update task", zap.Error(err))
			}
		}

		if !done {
			_, err := state.Pool.Exec(state.Context, "UPDATE tasks SET state = $1 WHERE task_id = $2", "failed", taskId)

			if err != nil {
				l.Error("Failed to update task", zap.Error(err))
			}
		}

		if ctxCancel != nil {
			defer ctxCancel()
		}

		_, err2 := state.Pool.Exec(state.Context, "DELETE FROM ongoing_tasks WHERE task_id = $1", taskId)

		if err != nil {
			l.Error("Failed to delete task from ongoing tasks", zap.Error(err2))
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

	// Set task state to running
	_, err := state.Pool.Exec(state.Context, "UPDATE tasks SET state = $1 WHERE task_id = $2", "running", taskId)

	if err != nil {
		l.Error("Failed to update task", zap.Error(err))
		return
	}

	ts := TaskState{
		Ctx: ctx,
	}
	if prog == nil {
		prog = &TaskProgress{
			TaskID:    taskId,
			TaskState: ts,
		}
	}

	var taskState = "completed"

	outp, terr := task.Exec(l, ts, prog)

	if terr != nil {
		l.Error("Failed to execute task [terr != nil]", zap.Error(terr))
		taskState = "failed"
	}

	// Save output to object storage
	if outp != nil {
		if outp.Filename == "" {
			outp.Filename = "unnamed." + crypto.RandString(16)
		}

		if outp.Buffer == nil {
			l.Error("Task output buffer is nil")
			taskState = "failed"
		} else {
			l.Info("Saving task output", zap.String("filename", outp.Filename))

			err = state.ObjectStorage.Save(
				state.Context,
				jobs.GetPathFromOutput(taskId, task, outp),
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

	_, err = state.Pool.Exec(state.Context, "UPDATE tasks SET output = $1, state = $2 WHERE task_id = $3", outp, taskState, taskId)

	if err != nil {
		l.Error("Failed to update task", zap.Error(err))
		return
	}

	done = true
}

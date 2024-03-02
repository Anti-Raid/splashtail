// Package taskexecutor defines a "production-ready" task executor.
//
// For local/non-production use, consider looking at cmd/localjobs's task executor
package jobrunner

import (
	"context"
	"net/http"
	"runtime/debug"

	"github.com/anti-raid/splashtail/jobserver/state"
	"github.com/anti-raid/splashtail/splashcore/types"
	"github.com/anti-raid/splashtail/tasks"
	"github.com/anti-raid/splashtail/tasks/taskstate"
	"github.com/bwmarrin/discordgo"
	"github.com/infinitybotlist/eureka/crypto"
	"go.uber.org/zap"
)

// PersistTaskState persists task state to redis temporarily
func PersistTaskState(tc *TaskProgress, prog *taskstate.Progress) error {
	_, err := state.Pool.Exec(
		tc.TaskState.Context(),
		"INSERT INTO ongoing_tasks (task_id, state, data) VALUES ($1, $2, $3) ON CONFLICT (task_id) DO UPDATE SET state = $2, data = $3",
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

// Implementor of tasks.TaskState
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
		return &taskstate.Progress{
			State: "",
			Data:  map[string]any{},
		}, nil
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
	task tasks.TaskDefinition,
	prog *TaskProgress,
) {
	if state.CurrentOperationMode != "jobs" {
		panic("cannot execute task outside of job server")
	}

	tInfo := task.Info()

	l, _ := NewTaskLogger(taskId, state.Pool, ctx, state.Logger)
	erl, _ := NewTaskLogger(taskId, state.Pool, state.Context, state.Logger)

	var done bool
	var bChan = make(chan int) // bChan is a channel thats used to control the canceller channel

	// Fail failed tasks
	defer func() {
		err := recover()

		if err != nil {
			erl.Error("Panic", zap.Any("err", err), zap.Any("data", tInfo.TaskFields))
			state.Logger.Error("Panic", zap.Any("err", err), zap.Any("data", tInfo.TaskFields))

			_, err := state.Pool.Exec(state.Context, "UPDATE tasks SET state = $1 WHERE task_id = $2", "failed", taskId)

			if err != nil {
				l.Error("Failed to update task", zap.Error(err), zap.Any("data", tInfo.TaskFields))
			}
		}

		if !done {
			_, err := state.Pool.Exec(state.Context, "UPDATE tasks SET state = $1 WHERE task_id = $2", "failed", taskId)

			if err != nil {
				l.Error("Failed to update task", zap.Error(err), zap.Any("data", tInfo.TaskFields))
			}
		}

		if ctxCancel != nil {
			defer ctxCancel()
		}

		bChan <- 1

		// Delete the task from ongoing tasks
		if prog.CurrentTaskProgress != nil {
			_, err2 := state.Pool.Exec(state.Context, "DELETE FROM ongoing_tasks WHERE task_id = $1", taskId)

			if err != nil {
				l.Error("Failed to delete task from ongoing tasks", zap.Error(err2), zap.Any("data", tInfo.TaskFields))
				return
			}
		}
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
		l.Error("Failed to update task", zap.Error(err), zap.Any("data", tInfo.TaskFields))
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
	var outp *types.TaskOutput

	outp, err = task.Exec(l, &types.TaskCreateResponse{
		TaskID:   taskId,
		TaskInfo: tInfo,
	}, ts, prog)

	if err != nil {
		l.Error("Failed to execute task", zap.Error(err))
		taskState = "failed"
	}

	// Save output to object storage
	if outp != nil {
		if outp.Filename == "" {
			outp.Filename = "unnamed." + crypto.RandString(16)
		}

		if outp.Buffer == nil {
			l.Error("Task output buffer is nil", zap.Any("data", tInfo.TaskFields))
			taskState = "failed"
		} else {
			l.Info("Saving task output", zap.String("filename", outp.Filename))

			err = state.ObjectStorage.Save(
				state.Context,
				tasks.GetPathFromOutput(taskId, tInfo, outp),
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
		l.Error("Failed to update task", zap.Error(err), zap.Any("data", tInfo.TaskFields))
		return
	}

	done = true
}

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
	"github.com/bwmarrin/discordgo"
	"github.com/infinitybotlist/eureka/crypto"
	"go.uber.org/zap"
)

// Implementor of tasks.TaskState
type TaskState struct{}

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

func (TaskState) Context() context.Context {
	return state.Context
}

// Creates a new task on server and executes it
func ExecuteTask(taskId string, task tasks.TaskDefinition) {
	if state.CurrentOperationMode != "jobs" {
		panic("cannot execute task outside of job server")
	}

	tInfo := task.Info()

	l, _ := NewTaskLogger(taskId, state.Pool, state.Context, state.Logger)

	var done bool

	// Fail failed tasks
	defer func() {
		err := recover()

		if err != nil {
			l.Error("Panic", zap.Any("err", err), zap.Any("data", tInfo.TaskFields))

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
	}()

	// Set task state to running
	_, err := state.Pool.Exec(state.Context, "UPDATE tasks SET state = $1 WHERE task_id = $2", "running", taskId)

	if err != nil {
		l.Error("Failed to update task", zap.Error(err), zap.Any("data", tInfo.TaskFields))
		return
	}

	var taskState = "completed"
	outp, err := task.Exec(l, &types.TaskCreateResponse{
		TaskID:   taskId,
		TaskInfo: tInfo,
	}, TaskState{})

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

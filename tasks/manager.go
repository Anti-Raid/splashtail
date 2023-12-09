package tasks

import (
	"context"
	"fmt"
	"splashtail/state"
	"splashtail/types"
	"time"

	"github.com/infinitybotlist/eureka/crypto"
	"github.com/jackc/pgx/v5"
	"go.uber.org/zap"
)

// Task management core
var TaskDefinitionRegistry = map[string]TaskDefinition{}

func RegisterTaskDefinition(task TaskDefinition) {
	TaskDefinitionRegistry[task.Info().Name] = task
}

func Pointer[T any](v T) *T {
	return &v
}

// TaskDefinition is the definition for any task that can be executed on splashtail
type TaskDefinition interface {
	// Validate validates the task
	Validate() error

	// Exec executes the task returning an output if any
	Exec(l *zap.Logger, tx pgx.Tx, tcr *types.TaskCreateResponse) (*types.TaskOutput, error)

	// Returns the info on a task
	Info() *types.TaskInfo
}

// Sets up a task
func CreateTask(ctx context.Context, task TaskDefinition) (*types.TaskCreateResponse, error) {
	tInfo := task.Info()

	_, ok := TaskDefinitionRegistry[tInfo.Name]

	if !ok {
		return nil, fmt.Errorf("task %s does not exist on registry", tInfo.Name)
	}

	taskKey := crypto.RandString(128)
	var taskId string

	err := state.Pool.QueryRow(ctx, "INSERT INTO tasks (task_name, task_key, task_for, expiry, output, task_info, allow_unauthenticated) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING task_id",
		tInfo.Name,
		taskKey,
		FormatTaskFor(tInfo.TaskFor),
		func() *time.Duration {
			if tInfo.Expiry == 0 {
				return nil
			}

			return &tInfo.Expiry
		}(),
		nil,
		tInfo,
		tInfo.AllowUnauthenticated,
	).Scan(&taskId)

	if err != nil {
		return nil, fmt.Errorf("failed to create task: %w", err)
	}

	return &types.TaskCreateResponse{
		TaskID:   taskId,
		TaskInfo: tInfo,
		TaskKey:  Pointer(taskKey),
	}, nil
}

// Creates a new task on server and executes it
func NewTask(tcr *types.TaskCreateResponse, task TaskDefinition) {
	tInfo := task.Info()

	l, _ := NewTaskLogger(tcr.TaskID)

	var done bool

	// Fail failed tasks
	defer func() {
		err := recover()

		if err != nil {
			l.Error("Panic", zap.Any("err", err), zap.Any("data", tInfo.TaskFields))

			_, err := state.Pool.Exec(state.Context, "UPDATE tasks SET state = $1 WHERE task_id = $2", "failed", tcr.TaskID)

			if err != nil {
				l.Error("Failed to update task", zap.Error(err), zap.Any("data", tInfo.TaskFields))
			}
		}

		if !done {
			_, err := state.Pool.Exec(state.Context, "UPDATE tasks SET state = $1 WHERE task_id = $2", "failed", tcr.TaskID)

			if err != nil {
				l.Error("Failed to update task", zap.Error(err), zap.Any("data", tInfo.TaskFields))
			}
		}
	}()

	l.Info("Creating task", zap.String("taskId", tcr.TaskID), zap.String("taskName", tInfo.Name), zap.Any("data", tInfo.TaskFields))

	// Set task state to running
	_, err := state.Pool.Exec(state.Context, "UPDATE tasks SET state = $1 WHERE task_id = $2", "running", tcr.TaskID)

	if err != nil {
		l.Error("Failed to update task", zap.Error(err), zap.Any("data", tInfo.TaskFields))
		return
	}

	tx, err := state.Pool.Begin(state.Context)

	if err != nil {
		l.Error("Failed to begin transaction", zap.Error(err), zap.Any("data", tInfo.TaskFields))
		return
	}

	defer tx.Rollback(state.Context)

	// Flush out old tasks
	tff := FormatTaskFor(tInfo.TaskFor)
	if tff != nil {
		_, err = tx.Exec(state.Context, "DELETE FROM tasks WHERE task_name = $1 AND task_id != $2 AND task_for = $3 AND state != 'completed'", tInfo.Name, tcr.TaskID, tff)
	} else {
		_, err = tx.Exec(state.Context, "DELETE FROM tasks WHERE task_name = $1 AND task_id != $2 AND task_for IS NULL AND state != 'completed'", tInfo.Name, tcr.TaskID)
	}

	if err != nil {
		l.Error("Failed to delete old tasks", zap.Error(err), zap.Any("data", tInfo.TaskFields))
		return
	}

	var taskState = "completed"
	outp, err := task.Exec(l, tx, tcr)

	if err != nil {
		l.Error("Failed to execute task", zap.Error(err), zap.Any("data", tInfo.TaskFields))
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
		}

		l.Info("Saving task output", zap.String("filename", outp.Filename))

		err = state.ObjectStorage.Save(
			state.Context,
			GetPathFromOutput(tcr.TaskID, tInfo, outp),
			outp.Filename,
			outp.Buffer,
			0,
		)

		if err != nil {
			l.Error("Failed to save backup", zap.Error(err))
			return
		}
	}

	_, err = tx.Exec(state.Context, "UPDATE tasks SET output = $1, state = $2 WHERE task_id = $3", outp, taskState, tcr.TaskID)

	if err != nil {
		l.Error("Failed to update task", zap.Error(err), zap.Any("data", tInfo.TaskFields))
		return
	}

	err = tx.Commit(state.Context)

	if err != nil {
		l.Error("Failed to commit transaction", zap.Error(err), zap.Any("data", tInfo.TaskFields))
		return
	}

	done = true
}

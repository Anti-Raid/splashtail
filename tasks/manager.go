package tasks

import (
	"context"
	"fmt"
	"splashtail/state"
	"splashtail/types"
	"splashtail/utils"
	"time"

	"github.com/infinitybotlist/eureka/crypto"
	"go.uber.org/zap"
)

// Task management core
var TaskDefinitionRegistry = map[string]TaskDefinition{}

func RegisterTaskDefinition(task TaskDefinition) {
	TaskDefinitionRegistry[task.Info().Name] = task
}

// TaskDefinition is the definition for any task that can be executed on splashtail
type TaskDefinition interface {
	// Validate validates the task and sets up state if needed
	Validate() error

	// Exec executes the task returning an output if any
	Exec(l *zap.Logger, tcr *types.TaskCreateResponse) (*types.TaskOutput, error)

	// Returns the info on a task
	Info() *types.TaskInfo
}

// Sets up a task
func CreateTask(ctx context.Context, task TaskDefinition) (*types.TaskCreateResponse, error) {
	err := task.Validate()

	if err != nil {
		return nil, fmt.Errorf("failed to validate task: %w", err)
	}

	tInfo := task.Info()

	if !tInfo.Valid {
		return nil, fmt.Errorf("invalid task info")
	}

	_, ok := TaskDefinitionRegistry[tInfo.Name]

	if !ok {
		return nil, fmt.Errorf("task %s does not exist on registry", tInfo.Name)
	}

	taskKey := crypto.RandString(128)
	var taskId string

	err = state.Pool.QueryRow(ctx, "INSERT INTO tasks (task_name, task_key, task_for, expiry, output, task_info, allow_unauthenticated) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING task_id",
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
		TaskKey:  utils.Pointer(taskKey),
	}, nil
}

// Creates a new task on server and executes it
func ExecuteTask(taskId string, task TaskDefinition) {
	if state.CurrentOperationMode != "jobs" {
		panic("cannot execute task outside of job server")
	}

	tInfo := task.Info()

	l, _ := NewTaskLogger(taskId)

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
	})

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
		}

		l.Info("Saving task output", zap.String("filename", outp.Filename))

		err = state.ObjectStorage.Save(
			state.Context,
			GetPathFromOutput(taskId, tInfo, outp),
			outp.Filename,
			outp.Buffer,
			0,
		)

		if err != nil {
			l.Error("Failed to save backup", zap.Error(err))
			return
		}
	}

	_, err = state.Pool.Exec(state.Context, "UPDATE tasks SET output = $1, state = $2 WHERE task_id = $3", outp, taskState, taskId)

	if err != nil {
		l.Error("Failed to update task", zap.Error(err), zap.Any("data", tInfo.TaskFields))
		return
	}

	done = true
}

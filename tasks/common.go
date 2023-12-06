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

var TaskRegistry = map[string]Task{}

func RegisterTask(task Task) {
	TaskRegistry[task.Info().Name] = task
}

func Pointer[T any](v T) *T {
	return &v
}

type TaskOutput struct {
	Path string `json:"path"` // If a path, this is the place where the file should be
}

type TaskInfo struct {
	TaskID     string        `json:"task_id"`
	Name       string        `json:"name"`
	For        *string       `json:"for"`
	TaskFields any           `json:"task_fields"`
	Expiry     time.Duration `json:"expiry"`
}

type TaskSet struct {
	TaskID string `json:"task_id"`
}

// Task is a task that can be executed on splashtail
type Task interface {
	// Validate validates the task
	Validate() error

	// Exec executes the task
	Exec(l *zap.Logger, tx pgx.Tx) error

	// Returns the info on a task
	Info() *TaskInfo

	// The output of the task
	Output() *TaskOutput

	// Set the output of the task
	Set(set *TaskSet) Task
}

// Sets up a task
func CreateTask(ctx context.Context, task Task, allowUnauthenticated bool) (Task, *types.TaskCreateResponse, error) {
	tInfo := task.Info()

	_, ok := TaskRegistry[tInfo.Name]

	if !ok {
		return nil, nil, fmt.Errorf("task %s does not exist on registry", tInfo.Name)
	}

	taskKey := crypto.RandString(128)
	var taskId string

	err := state.Pool.QueryRow(ctx, "INSERT INTO tasks (task_name, task_key, for_user, expiry, output, allow_unauthenticated) VALUES ($1, $2, $3, $4, $5, $6) RETURNING task_id",
		tInfo.Name,
		taskKey,
		tInfo.For,
		tInfo.Expiry,
		map[string]any{
			"meta": map[string]any{},
		},
		allowUnauthenticated,
	).Scan(&taskId)

	if err != nil {
		return nil, nil, fmt.Errorf("failed to create task: %w", err)
	}

	task = task.Set(&TaskSet{
		TaskID: taskId,
	})

	return task, &types.TaskCreateResponse{
		TaskID:               taskId,
		TaskName:             tInfo.Name,
		TaskKey:              Pointer(taskKey),
		AllowUnauthenticated: allowUnauthenticated,
		Expiry:               tInfo.Expiry,
	}, nil
}

// Creates a new task on server and executes it
func NewTask(task Task) {
	tInfo := task.Info()

	l, _ := NewTaskLogger(tInfo.TaskID)

	var done bool

	// Fail failed tasks
	defer func() {
		err := recover()

		if err != nil {
			l.Error("Panic", zap.Any("err", err), zap.Any("data", tInfo.TaskFields))

			_, err := state.Pool.Exec(state.Context, "UPDATE tasks SET state = $1 WHERE task_id = $2", "failed", tInfo.TaskID)

			if err != nil {
				l.Error("Failed to update task", zap.Error(err), zap.Any("data", tInfo.TaskFields))
			}
		}

		if !done {
			l.Error("Failed to complete task", zap.Any("data", tInfo.TaskFields))

			_, err := state.Pool.Exec(state.Context, "UPDATE tasks SET state = $1 WHERE task_id = $2", "failed", tInfo.TaskID)

			if err != nil {
				l.Error("Failed to update task", zap.Error(err), zap.Any("data", tInfo.TaskFields))
			}
		}
	}()

	l.Info("Creating task", zap.String("taskId", tInfo.TaskID), zap.String("taskName", tInfo.Name), zap.Any("data", tInfo.TaskFields))

	tx, err := state.Pool.Begin(state.Context)

	if err != nil {
		l.Error("Failed to begin transaction", zap.Error(err), zap.Any("data", tInfo.TaskFields))
		return
	}

	defer tx.Rollback(state.Context)

	if tInfo.For != nil {
		_, err = tx.Exec(state.Context, "DELETE FROM tasks WHERE task_name = $1 AND task_id != $2 AND for_user = $3 AND state != 'completed'", tInfo.Name, tInfo.TaskID, tInfo.For)
	} else {
		_, err = tx.Exec(state.Context, "DELETE FROM tasks WHERE task_name = $1 AND task_id != $2 AND for_user IS NULL AND state != 'completed'", tInfo.Name, tInfo.TaskID)
	}

	if err != nil {
		l.Error("Failed to delete old tasks", zap.Error(err), zap.Any("data", tInfo.TaskFields))
		return
	}

	// Execute the task here
	err = task.Exec(l, tx)

	if err != nil {
		l.Error("Failed to execute task", zap.Error(err), zap.Any("data", tInfo.TaskFields))

		// Set task output
		outp := task.Output()

		if outp == nil {
			outp = &TaskOutput{}
		}

		_, err := tx.Exec(state.Context, "UPDATE tasks SET state = $1, output = $2 WHERE task_id = $3", "failed", outp, tInfo.TaskID)

		if err != nil {
			l.Error("Failed to update task", zap.Error(err), zap.Any("data", tInfo.TaskFields))
		}

		return
	}

	// Set task output
	outp := task.Output()

	if outp == nil {
		outp = &TaskOutput{}
	}

	_, err = tx.Exec(state.Context, "UPDATE tasks SET output = $1, state = $2 WHERE task_id = $3", outp, "completed", tInfo.TaskID)

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

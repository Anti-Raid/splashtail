package lib

import (
	"fmt"
	"os"

	"github.com/anti-raid/splashtail/tasks"
	"github.com/anti-raid/splashtail/types"

	"github.com/infinitybotlist/eureka/crypto"
	"go.uber.org/zap"
)

type TaskLocalOpts struct {
	OnStateChange func(state string) error
}

// Executes a task locally
func ExecuteTaskLocal(taskId string, l *zap.Logger, task tasks.TaskDefinition, opts TaskLocalOpts) error {
	var currentTaskState = "pending"

	err := opts.OnStateChange(currentTaskState)

	if err != nil {
		return fmt.Errorf("failed to update task state: %w", err)
	}

	err = task.Validate()

	if err != nil {
		return fmt.Errorf("failed to validate task: %w", err)
	}

	tInfo := task.Info()

	if !tInfo.Valid {
		return fmt.Errorf("invalid task info")
	}

	_, ok := tasks.TaskDefinitionRegistry[tInfo.Name]

	if !ok {
		return fmt.Errorf("task %s does not exist on registry", tInfo.Name)
	}

	currentTaskState = "running"

	err = opts.OnStateChange(currentTaskState)

	if err != nil {
		return fmt.Errorf("failed to update task state: %w", err)
	}

	outp, err := task.Exec(l, &types.TaskCreateResponse{
		TaskID:   taskId,
		TaskInfo: tInfo,
	})

	if err != nil {
		return fmt.Errorf("failed to execute task: %w", err)
	}

	// Save output to object storage
	if outp != nil {
		if outp.Filename == "" {
			outp.Filename = "unnamed." + crypto.RandString(16)
		}

		if outp.Buffer == nil {
			l.Error("Task output buffer is nil", zap.Any("data", tInfo.TaskFields))
			currentTaskState = "failed"

			err = opts.OnStateChange(currentTaskState)

			if err != nil {
				return fmt.Errorf("failed to update task state: %w", err)
			}
		} else {
			// Write task output to tasks/$taskId/$output
			err = os.MkdirAll("tasks/"+taskId, 0755)

			if err != nil {
				return fmt.Errorf("failed to create task output directory: %w", err)
			}

			f, err := os.Create("tasks/" + taskId + "/" + outp.Filename)

			if err != nil {
				return fmt.Errorf("failed to create task output file: %w", err)
			}

			_, err = f.Write(outp.Buffer.Bytes())

			if err != nil {
				return fmt.Errorf("failed to write task output file: %w", err)
			}

			l.Info("Saved task output", zap.String("filename", outp.Filename), zap.String("task_id", taskId))
		}
	}

	return nil
}

package lib

import (
	"fmt"
	"os"

	jobs "go.jobs"
	"go.jobs/interfaces"
	jobstate "go.jobs/state"

	"github.com/infinitybotlist/eureka/crypto"
	"go.uber.org/zap"
)

type TaskLocalOpts struct {
	OnStateChange func(state string) error
}

// Executes a job locally
func ExecuteJobLocal(prefix, taskId string, l *zap.Logger, jobImpl interfaces.JobImpl, opts TaskLocalOpts, taskState jobstate.State) error {
	var currentTaskState = "pending"

	err := opts.OnStateChange(currentTaskState)

	if err != nil {
		return fmt.Errorf("failed to update job state: %w", err)
	}

	err = jobImpl.Validate(taskState)

	if err != nil {
		return fmt.Errorf("failed to validate job: %w", err)
	}

	_, ok := jobs.JobImplRegistry[jobImpl.Name()]

	if !ok {
		return fmt.Errorf("job %s does not exist on registry", jobImpl.Name())
	}

	currentTaskState = "running"

	err = opts.OnStateChange(currentTaskState)

	if err != nil {
		return fmt.Errorf("failed to update task state: %w", err)
	}

	outp, terr := jobImpl.Exec(l, taskState, TaskProgress{})

	// Save output to object storage
	if outp != nil {
		if outp.Filename == "" {
			outp.Filename = "unnamed." + crypto.RandString(16)
		}

		if outp.Buffer == nil {
			l.Error("Job output buffer is nil", zap.Any("data", jobImpl))
			currentTaskState = "failed"

			err = opts.OnStateChange(currentTaskState)

			if err != nil {
				return fmt.Errorf("failed to update task state: %w", err)
			}
		} else {
			// Write task output to jobs/$taskId/$output
			err = os.MkdirAll(prefix+"/jobs/"+taskId, 0755)

			if err != nil {
				return fmt.Errorf("failed to create job output directory: %w", err)
			}

			f, err := os.Create(prefix + "/jobs/" + taskId + "/" + outp.Filename)

			if err != nil {
				return fmt.Errorf("failed to create task output file: %w", err)
			}

			_, err = f.Write(outp.Buffer.Bytes())

			if err != nil {
				return fmt.Errorf("failed to write task output file: %w", err)
			}

			l.Info("Saved task output", zap.String("filename", outp.Filename), zap.String("id", taskId))
		}
	}

	if terr != nil {
		l.Error("Failed to execute job", zap.Error(err))
		currentTaskState = "failed"
		err = opts.OnStateChange(currentTaskState)

		if err != nil {
			return fmt.Errorf("failed to update task state: %w", err)
		}
		return fmt.Errorf("failed to execute task: %w", err)
	}

	return nil
}

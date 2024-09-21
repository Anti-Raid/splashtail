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
func ExecuteJobLocal(prefix, id string, l *zap.Logger, jobImpl interfaces.JobImpl, opts TaskLocalOpts, state jobstate.State) error {
	var currentState = "pending"

	err := opts.OnStateChange(currentState)

	if err != nil {
		return fmt.Errorf("failed to update job state: %w", err)
	}

	err = jobImpl.Validate(state)

	if err != nil {
		return fmt.Errorf("failed to validate job: %w", err)
	}

	_, ok := jobs.JobImplRegistry[jobImpl.Name()]

	if !ok {
		return fmt.Errorf("job %s does not exist on registry", jobImpl.Name())
	}

	currentState = "running"

	err = opts.OnStateChange(currentState)

	if err != nil {
		return fmt.Errorf("failed to update state: %w", err)
	}

	outp, terr := jobImpl.Exec(l, state, Progress{})

	// Save output to object storage
	if outp != nil {
		if outp.Filename == "" {
			outp.Filename = "unnamed." + crypto.RandString(16)
		}

		if outp.Buffer == nil {
			l.Error("Job output buffer is nil", zap.Any("data", jobImpl))
			currentState = "failed"

			err = opts.OnStateChange(currentState)

			if err != nil {
				return fmt.Errorf("failed to update state: %w", err)
			}
		} else {
			// Write task output to jobs/$id/$output
			err = os.MkdirAll(prefix+"/jobs/"+id, 0755)

			if err != nil {
				return fmt.Errorf("failed to create job output directory: %w", err)
			}

			f, err := os.Create(prefix + "/jobs/" + id + "/" + outp.Filename)

			if err != nil {
				return fmt.Errorf("failed to create output file: %w", err)
			}

			_, err = f.Write(outp.Buffer.Bytes())

			if err != nil {
				return fmt.Errorf("failed to write output file: %w", err)
			}

			l.Info("Saved output", zap.String("filename", outp.Filename), zap.String("id", id))
		}
	}

	if terr != nil {
		l.Error("Failed to execute job", zap.Error(err))
		currentState = "failed"
		err = opts.OnStateChange(currentState)

		if err != nil {
			return fmt.Errorf("failed to update state: %w", err)
		}
		return fmt.Errorf("failed to execute job: %w", err)
	}

	return nil
}

package jobserver

import (
	"context"
	"fmt"
	"time"

	"github.com/anti-raid/splashtail/jobserver/jobrunner"
	"github.com/anti-raid/splashtail/jobserver/state"
	"github.com/anti-raid/splashtail/splashcore/animusmagic"
	"github.com/anti-raid/splashtail/tasks"
	jsoniter "github.com/json-iterator/go"
	"go.uber.org/zap"
)

var DefaultTimeout = 1 * time.Microsecond
var DefaultValidationTimeout = 5 * time.Second

var json = jsoniter.ConfigFastest

func CreateJobServer() {
	state.AnimusMagicClient.OnRequest = func(c *animusmagic.ClientRequest) (animusmagic.AnimusResponse, error) {
		defer func() {
			if rvr := recover(); rvr != nil {
				fmt.Println("Recovered from panic:", rvr)
			}
		}()

		state.Logger.Info("Recieved request", zap.String("from", c.Meta.From.String()), zap.String("to", c.Meta.To.String()), zap.String("commandId", c.Meta.CommandID))

		data, err := animusmagic.ParseClientRequest[animusmagic.JobserverMessage](c)

		if err != nil {
			return nil, err
		}

		if data == nil {
			return nil, fmt.Errorf("nil data")
		}

		if data.SpawnTask != nil {
			if !data.SpawnTask.Create && !data.SpawnTask.Execute {
				return nil, fmt.Errorf("either create or execute must be set")
			}

			if data.SpawnTask.Name == "" {
				return nil, fmt.Errorf("invalid task name provided")
			}

			baseTaskDef, ok := tasks.TaskDefinitionRegistry[data.SpawnTask.Name]

			if !ok {
				return nil, fmt.Errorf("task %s does not exist on registry", data.SpawnTask.Name)
			}

			if len(data.SpawnTask.Data) == 0 {
				return nil, fmt.Errorf("invalid task data provided")
			}

			tBytes, err := json.Marshal(data.SpawnTask.Data)

			if err != nil {
				return nil, fmt.Errorf("error marshalling task args: %w", err)
			}

			task := baseTaskDef // Copy task

			err = json.Unmarshal(tBytes, &task)

			if err != nil {
				return nil, fmt.Errorf("error unmarshalling task args: %w", err)
			}

			// Validate task
			ctx, cancel := context.WithTimeout(state.Context, DefaultValidationTimeout)
			defer cancel()
			err = task.Validate(jobrunner.TaskState{
				Ctx: ctx,
			})

			if err != nil {
				return nil, fmt.Errorf("failed to validate task: %w", err)
			}

			// Create task
			var taskId string
			if data.SpawnTask.Create {
				tcr, err := jobrunner.CreateTask(state.Context, state.Pool, task)

				if err != nil {
					return nil, fmt.Errorf("error creating task: %w", err)
				}

				taskId = tcr.TaskID
			} else {
				if data.SpawnTask.TaskID == "" {
					return nil, fmt.Errorf("task id must be set if SpawnTask.Create is false")
				}

				taskId = data.SpawnTask.TaskID
			}

			// Execute task
			if data.SpawnTask.Execute {
				ctx, cancel := context.WithTimeout(state.Context, DefaultTimeout)
				go jobrunner.ExecuteTask(ctx, cancel, taskId, task, nil)
			}

			return &animusmagic.JobserverResponse{
				SpawnTask: &struct {
					TaskID string "json:\"task_id\""
				}{
					TaskID: taskId,
				},
			}, nil
		}

		return nil, fmt.Errorf("invalid request")
	}

	// Start listening
	go state.AnimusMagicClient.ListenOnce(
		state.Context,
		state.Rueidis,
		state.Logger,
	)
}

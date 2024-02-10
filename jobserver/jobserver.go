package jobserver

import (
	"fmt"

	"github.com/anti-raid/splashtail/jobserver/endpoints"
	"github.com/anti-raid/splashtail/jobserver/endpoints/create_task"
	"github.com/anti-raid/splashtail/jobserver/endpoints/execute_task"
	"github.com/anti-raid/splashtail/jobserver/jobrunner"
	"github.com/anti-raid/splashtail/jobserver/state"
	"github.com/anti-raid/splashtail/splashcore/animusmagic"
	"github.com/anti-raid/splashtail/tasks"

	jsoniter "github.com/json-iterator/go"
)

var expectedSecretMap map[string]string // Set during setup

var json = jsoniter.ConfigFastest

var ipcEvents = map[string]endpoints.IPC{
	"create_task":  create_task.CreateTask,
	"execute_task": execute_task.ExecuteTask,
}

type IpcRequest struct {
	// Arguments to pass to the ipc command
	Args map[string]any `json:"args"`
}

func CreateJobServer() {
	state.AnimusMagicClient.OnRequest = func(c *animusmagic.ClientRequest) (animusmagic.AnimusResponse, error) {
		defer func() {
			if rvr := recover(); rvr != nil {
				fmt.Println("Recovered from panic:", rvr)
			}
		}()

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

			tBytes, err := json.Marshal(data)

			if err != nil {
				return nil, fmt.Errorf("error marshalling task args: %w", err)
			}

			task := baseTaskDef // Copy task

			err = json.Unmarshal(tBytes, &task)

			if err != nil {
				return nil, fmt.Errorf("error unmarshalling task args: %w", err)
			}

			// Validate task
			err = task.Validate(jobrunner.TaskState{})

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
				go jobrunner.ExecuteTask(taskId, task)
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

	return
}

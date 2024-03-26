// In many cases such as restoring backups, tasks can be quite complex
// and should/can be broken down into smaller steps
//
// Step is an utility structure that allows breaking down tasks complete with persist support
package step

import (
	"fmt"
	"strconv"

	"github.com/anti-raid/splashtail/splashcore/types"
	"github.com/anti-raid/splashtail/tasks/taskstate"
	"go.uber.org/zap"
)

type Stepper[T any] struct {
	steps          []*Step[T]
	stepCache      map[string]*Step[T]
	stepIndexCache map[string]int
}

// NewStepper creates a new stepper
func NewStepper[T any](steps ...Step[T]) *Stepper[T] {
	var s = &Stepper[T]{
		steps:          []*Step[T]{},
		stepCache:      map[string]*Step[T]{},
		stepIndexCache: map[string]int{},
	}

	for i := range steps {
		step := steps[i]

		if step.State == "" {
			panic("step state cannot be empty")
		}

		// Ensure no duplicate steps
		if _, ok := s.stepCache[step.State]; ok {
			panic("duplicate step state")
		}

		if step.Index == 0 {
			step.Index = i
		}

		fmt.Println(i, step.Index, step.State)

		s.steps = append(s.steps, &step)
		s.stepCache[step.State] = &step
		s.stepIndexCache[step.State] = step.Index
	}

	return s
}

// Step returns a step based on its state
func (s *Stepper[T]) Step(state string) (*Step[T], bool) {
	if step, ok := s.stepCache[state]; ok {
		return step, true
	}

	return nil, false
}

// StepPosition returns the index of a step
func (s *Stepper[T]) StepIndex(state string) int {
	if pos, ok := s.stepIndexCache[state]; ok {
		return pos
	}

	return -1
}

// Exec executes all steps, skipping over steps with a lower index
func (s *Stepper[T]) Exec(
	task *T,
	l *zap.Logger,
	tcr *types.TaskCreateResponse,
	state taskstate.TaskState,
	progstate taskstate.TaskProgressState,
) (*types.TaskOutput, error) {
	curProg, err := progstate.GetProgress()

	if err != nil {
		return nil, err
	}

	if curProg == nil {
		curProg = &taskstate.Progress{
			State: "",
			Data:  map[string]any{},
		}
	}

	fmt.Println(curProg.State)

	for i := range s.steps {
		step := s.steps[i]

		select {
		case <-state.Context().Done():
			return nil, state.Context().Err()
		default:
			// Continue
		}

		fmt.Println(curProg.State)

		// Conditions to run a step:
		//
		// 1. curProg.State is empty means the step will be executed
		// 2. curProg.State is not empty and is equal to the step state
		// 3. curProg.State is not empty and is not equal to the step state but the step index is greater than or equal to the current step index
		if curProg.State == "" || curProg.State == step.State || step.Index >= s.StepIndex(curProg.State) {
			l.Info("[" + strconv.Itoa(step.Index) + "] Executing step '" + step.State + "'")

			outp, prog, err := step.Exec(task, l, tcr, state, progstate, curProg)

			if err != nil {
				return nil, err
			}

			if outp != nil {
				return outp, nil
			}

			if prog != nil {
				if prog.State == "" {
					// Get the next step and use that for state
					if len(s.steps) > i {
						prog.State = s.steps[i+1].State
					} else {
						prog.State = "completed"
					}
				} else {
					// Ensure the next step is valid
					if _, ok := s.Step(prog.State); !ok {
						return nil, fmt.Errorf("invalid step state")
					}
				}

				curProg.State = prog.State // Update state

				if prog.Data != nil {
					// Prog is additive, add in all the elements from prog to curProg
					for k, v := range prog.Data {
						if v == nil {
							// Delete from curProg
							delete(curProg.Data, k)
						} else {
							curProg.Data[k] = v
						}
					}
				}

				err = progstate.SetProgress(curProg)

				if err != nil {
					return nil, err
				}
			}
		} else {
			l.Info("[" + strconv.Itoa(step.Index) + "] Skipping step '" + step.State + "' [resuming task?]")
		}
	}

	return nil, nil
}

type Step[T any] struct {
	State string

	// By default, steps of a lower index are ignored
	// Steps may however have an equal index in which case the step that is first in the array is first executed
	Index int

	// Exec will either return the task output which ends the task
	// or a task progress telling the new progress of the task
	// or an error to quickly abort the stepping
	//
	// After finishing the task
	Exec func(
		t *T,
		l *zap.Logger,
		tcr *types.TaskCreateResponse,
		state taskstate.TaskState,
		progstate taskstate.TaskProgressState,
		progress *taskstate.Progress,
	) (*types.TaskOutput, *taskstate.Progress, error)
}

package common

import jobstate "go.jobs/state"

// SaveIntermediateResult is a helper method to save an intermediate
// result to database within the state
func SaveIntermediateResult(
	progstate jobstate.ProgressState,
	prog *jobstate.Progress,
	data map[string]any,
) error {
	// Prog is additive, add in all the elements from data to progress
	for k, v := range data {
		if v == nil {
			// Delete from curProg
			delete(data, k)
		} else {
			prog.Data[k] = v
		}
	}

	return progstate.SetProgress(prog)
}

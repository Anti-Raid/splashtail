# Go Jobserver

The Go Jobserver handles server backups, message prunes and other long running tasks on a server

# Go Jobserver Developer Docs

This folder documents the Go Jobserver component of AntiRaid. The jobserver is responsible for handling long-running and background tasks such as server backups, message pruning, and other asynchronous jobs.

The Job Server takes in jobs either via the Spawn internal RPC endpoint or from the database of ongoing jobs. These jobs are then processed with larger jobs making use of a special `Step` abstraction that allows the Jobserver to reach and mark checkpoints to continue/start from.

## Startup Tasks

1. First, the jobserver marks all `pending` tasks as `failed` as these are tasks that have been registered but not yet started.
2. Two tasks are then spawned, one for the jobservers RPC server and the second for resuming currently ongoing tasks
3. The RPC server is an HTTP server which waits for new jobs (via Spawn RPC endpoint) and then creates a new job. This job is then passed to the JobExecutor abstraction which then runs the job to execution
4. For resuming, the jobserver first removes stale tasks that are too old to be resumed. All ``ongoing_jobs`` are then created and scheduled for execution via JobExecutor.

## JobExecutor

The job of the JobExecutor is simple, all it needs to do is provide an ``Execute`` method to allow executing a ``interfaces.JobImpl`` (code for a job) and implement the below (simple) interfaces. There are currently two such JobExecutors: ``localjobs`` for local runs of jobserver and ``jobrunner`` for production runs.

```go
type State interface {
	// GetTransport returns the http transport to use for jobs
	//
	// E.g. localjobs uses a custom http transport to support local files via the file:// scheme
	Transport() *http.Transport

	// Returns the current operation mode of the service (jobserver/localjob etc.). Similar to webserver/state.CurrentOperationMode
	OperationMode() string

	// Returns a *discordgo.Session, the bots current user and a boolean indicating whether or not the gateway is available
	//
	// This boolean is currently unused
	Discord() (*discordgo.Session, *discordgo.User, bool)

	// Debug Info and extra debug info
	DebugInfo() *debug.BuildInfo

	// Context returns the context to use for the job
	Context() context.Context

	// GuildID returns the guild ID for the job, if applicable
	GuildID() string
}

type Progress struct {
	State string
	Data  map[string]any
}

type ProgressState interface {
	// GetProgress returns the current progress of the job. This is useful
	// for resumable jobs like server restores
	GetProgress() (*Progress, error)

	// Sets/demarkates the progress of the job, if supported
	SetProgress(prog *Progress) error
}
```

## Checkpointing

### SaveIntermediateResults

This simple helper method, while small, is a core primitive for checkpointing (Step is the other one):

```go
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
```

This simple primitives uses ``SetProgress`` to allow saving intermediate results to the current state. This allows for tasks to checkpoint data and then resume with said data.

### Step

The other core primitive for checkpointing and resuming with progress is ``step``. With ``step``, you split the job into multiple steps and provide them to the ``step.NewStepper`` followed by then calling ``Exec``. The ``step`` primitive will then keep track of where in the steps the job is using ``SetProgress`` to save the current step. If the jobserver fails during a specific step, it will then be automatically resumed at the exact step it failed at.

## Files
- `backups.md`: Details on the backup job system and implementation.
- `README.md`: Overview of the jobserver's responsibilities and architecture.

## Key Concepts
- Jobs are queued and executed independently of the main bot/API processes.
- Designed for reliability and resumability: jobs can survive bot/API restarts.
- Written in Go for concurrency and performance.

---

For more details, see each file in this folder.

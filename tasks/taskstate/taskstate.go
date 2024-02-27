// To avoid needing to store the entire state even when not needed, tasks defines a state interface
// to store and fetch only needed state
package taskstate

import (
	"context"
	"net/http"
	"runtime/debug"

	"github.com/bwmarrin/discordgo"
)

type TaskState interface {
	// GetTransport returns the http transport to use for tasks
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

	// Context returns the context to use for the task
	Context() context.Context
}

type TaskProgressState interface {
	// GetProgress returns the current progress of the task. This is useful
	// for resumable tasks like server restores
	GetProgress() (string, map[string]any, error)

	// Sets/demarkates the progress of the task, if supported
	SetProgress(state string, data map[string]any) error
}

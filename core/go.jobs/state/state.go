// To avoid needing to store the entire state even when not needed, jobs defines a state interface
// to store and fetch only needed state
package state

import (
	"context"
	"net/http"
	"runtime/debug"

	"github.com/bwmarrin/discordgo"
)

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

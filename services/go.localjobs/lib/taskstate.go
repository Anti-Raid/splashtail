package lib

import (
	"context"
	"net/http"
	"runtime/debug"

	"github.com/bwmarrin/discordgo"
	jobstate "go.jobs/state"
)

// Implementor of jobstate.State
type TaskState struct {
	HttpTransport *http.Transport
	DiscordSess   *discordgo.Session
	BotUser       *discordgo.User
	DebugInfoData *debug.BuildInfo
	ContextUse    context.Context
}

func (ts TaskState) Transport() *http.Transport {
	return ts.HttpTransport
}

func (TaskState) OperationMode() string {
	return "localjobs"
}

func (ts TaskState) Discord() (*discordgo.Session, *discordgo.User, bool) {
	return ts.DiscordSess, ts.BotUser, false
}

func (ts TaskState) DebugInfo() *debug.BuildInfo {
	return ts.DebugInfoData
}

func (ts TaskState) Context() context.Context {
	return ts.ContextUse
}

type TaskProgress struct{}

func (ts TaskProgress) GetProgress() (*jobstate.Progress, error) {
	return &jobstate.Progress{
		State: "",
		Data:  map[string]any{},
	}, nil
}

func (ts TaskProgress) SetProgress(prog *jobstate.Progress) error {
	return nil
}

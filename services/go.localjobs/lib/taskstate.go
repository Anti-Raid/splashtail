package lib

import (
	"context"
	"net/http"
	"runtime/debug"

	"github.com/bwmarrin/discordgo"
	"go.jobs/taskstate"
)

// Implementor of tasks.TaskState
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

func (ts TaskProgress) GetProgress() (*taskstate.Progress, error) {
	return &taskstate.Progress{
		State: "",
		Data:  map[string]any{},
	}, nil
}

func (ts TaskProgress) SetProgress(prog *taskstate.Progress) error {
	return nil
}

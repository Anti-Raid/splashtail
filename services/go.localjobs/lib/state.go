package lib

import (
	"context"
	"net/http"
	"runtime/debug"

	"github.com/bwmarrin/discordgo"
	jobstate "go.jobs/state"
)

// Implementor of jobstate.State
type State struct {
	HttpTransport *http.Transport
	DiscordSess   *discordgo.Session
	BotUser       *discordgo.User
	DebugInfoData *debug.BuildInfo
	ContextUse    context.Context
}

func (ts State) Transport() *http.Transport {
	return ts.HttpTransport
}

func (State) OperationMode() string {
	return "localjobs"
}

func (ts State) Discord() (*discordgo.Session, *discordgo.User, bool) {
	return ts.DiscordSess, ts.BotUser, false
}

func (ts State) DebugInfo() *debug.BuildInfo {
	return ts.DebugInfoData
}

func (ts State) Context() context.Context {
	return ts.ContextUse
}

type Progress struct{}

func (ts Progress) GetProgress() (*jobstate.Progress, error) {
	return &jobstate.Progress{
		State: "",
		Data:  map[string]any{},
	}, nil
}

func (ts Progress) SetProgress(prog *jobstate.Progress) error {
	return nil
}

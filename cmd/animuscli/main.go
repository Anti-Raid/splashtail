package main

import (
	"context"
	"fmt"

	"github.com/anti-raid/splashtail/animusmagic"
	"github.com/infinitybotlist/eureka/shellcli"
	"github.com/redis/rueidis"
)

type AnimusCliData struct {
	Context           context.Context
	ContextClose      context.CancelFunc
	Rueidis           rueidis.Client
	AnimusMagicClient *animusmagic.AnimusMagicClient
	Connected         bool
}

var root *shellcli.ShellCli[AnimusCliData]

func main() {
	root = &shellcli.ShellCli[AnimusCliData]{
		Data: &AnimusCliData{},
		Prompter: func(r *shellcli.ShellCli[AnimusCliData]) string {
			return "animuscli> "
		},
		Commands: map[string]*shellcli.Command[AnimusCliData]{
			"connect": {
				Description: "Connects animuscli with the given options",
				Args: [][3]string{
					{"redis", "Redis URL to connect to", "redis://localhost:6379"},
					{"channel", "AnimusMagic channel to connect to", "animus_magic-staging"},
				},
				Run: func(a *shellcli.ShellCli[AnimusCliData], args map[string]string) error {
					var redisUrl = "redis://localhost:6379"

					if args["redis"] != "" {
						redisUrl = args["redis"]
					}

					var animusMagicChannel = "animus_magic-staging"

					if ch, ok := args["channel"]; ok && ch != "" {
						animusMagicChannel = ch
					}

					// Reuidis
					ruOptions, err := rueidis.ParseURL(redisUrl)

					if err != nil {
						return fmt.Errorf("error parsing redis url: %s", err)
					}

					a.Data.Rueidis, err = rueidis.NewClient(ruOptions)

					if err != nil {
						return fmt.Errorf("error creating redis client: %s", err)
					}

					a.Data.AnimusMagicClient = animusmagic.New(animusMagicChannel)

					a.Data.Context, a.Data.ContextClose = context.WithCancel(context.Background())

					a.Data.Connected = true

					return nil
				},
			},
			"disconnect": {
				Description: "Disconnects animuscli from the current connection",
				Args:        [][3]string{},
				Run: func(a *shellcli.ShellCli[AnimusCliData], args map[string]string) error {
					if !a.Data.Connected {
						return fmt.Errorf("not connected")
					}

					a.Data.Rueidis.Close()
					a.Data.ContextClose()
					a.Data.Connected = false

					return nil
				},
			},
		},
	}

	root.AddCommand("help", root.Help())

	root.Run()
}

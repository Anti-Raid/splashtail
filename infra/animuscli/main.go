package main

import (
	"context"
	"fmt"
	"os"
	"strconv"
	"time"

	"github.com/anti-raid/splashtail/splashcore/animusmagic"
	"github.com/infinitybotlist/eureka/crypto"
	"github.com/infinitybotlist/eureka/shellcli"
	"github.com/redis/rueidis"
	"go.uber.org/zap"
	"go.uber.org/zap/zapcore"
)

type AnimusCliWriter struct{}

type AnimusCliData struct {
	Context           context.Context
	ContextClose      context.CancelFunc
	Rueidis           rueidis.Client
	AnimusMagicClient *animusmagic.AnimusMagicClient
	Connected         bool
	Logger            *zap.Logger
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
					{"from", "Source", "0x2 (AnimusTargetWebserver)"},
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

					var target = 0x2

					if from, ok := args["from"]; ok && from != "" {
						targetInt, err := strconv.Atoi(from)

						if err != nil {
							return fmt.Errorf("error converting target to integer: %s", err)
						}

						target = targetInt
					}

					a.Data.AnimusMagicClient = animusmagic.New(animusMagicChannel, animusmagic.AnimusTarget(target))

					a.Data.Context, a.Data.ContextClose = context.WithCancel(context.Background())

					a.Data.Connected = true

					a.Data.Logger = zap.New(
						zapcore.NewCore(
							zapcore.NewConsoleEncoder(zap.NewDevelopmentEncoderConfig()),
							os.Stdout,
							zap.DebugLevel,
						),
					)

					go func() {
						err := a.Data.AnimusMagicClient.Listen(a.Data.Context, a.Data.Rueidis, a.Data.Logger)

						if err != nil {
							a.Data.Logger.Fatal("error listening to animus magic", zap.Error(err))
						}
					}()

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
			"probe": {
				Description: "Probes the animus magic channel. This only works for clients which implement the Probe AnimusMessage",
				Args: [][3]string{
					{"timeout", "Timeout in seconds", "5"},
					{"to", "Target", "0"},
				},
				Run: func(a *shellcli.ShellCli[AnimusCliData], args map[string]string) error {
					if !a.Data.Connected {
						return fmt.Errorf("not connected")
					}

					timeout, ok := args["timeout"]

					if !ok {
						timeout = "5"
					}

					to, ok := args["to"]

					if !ok {
						to = "0"
					}

					// Convert to integer
					toInt, err := strconv.Atoi(to)

					if err != nil {
						return fmt.Errorf("error converting to integer: %s", err)
					}

					timeoutInt, err := strconv.Atoi(timeout)

					if err != nil {
						return fmt.Errorf("error converting timeout to integer: %s", err)
					}

					toTarget := animusmagic.AnimusTarget(byte(toInt))

					var msg animusmagic.AnimusMessage = animusmagic.CommonAnimusMessage{
						Probe: &struct{}{},
					}

					commandId := crypto.RandString(512)
					payload, err := a.Data.AnimusMagicClient.CreatePayload(
						animusmagic.AnimusTargetWebserver,
						toTarget,
						animusmagic.WildcardClusterID,
						animusmagic.OpRequest,
						commandId,
						msg,
					)

					if err != nil {
						return fmt.Errorf("error creating payload: %s", err)
					}

					// Create a channel to receive the response
					notify := a.Data.AnimusMagicClient.CreateNotifier(commandId, 0)

					// Publish the payload
					err = a.Data.AnimusMagicClient.Publish(a.Data.Context, a.Data.Rueidis, payload)

					if err != nil {
						// Remove the notifier
						a.Data.AnimusMagicClient.CloseNotifier(commandId)
						return fmt.Errorf("error publishing payload: %s", err)
					}

					// Wait for the response
					ticker := time.NewTicker(time.Second * time.Duration(timeoutInt))
					startTime := time.Now()
					for {
						select {
						case <-a.Data.Context.Done():
							return fmt.Errorf("context cancelled")
						case <-ticker.C:
							return nil
						case response := <-notify:
							since := time.Since(startTime)
							go func() {
								fmt.Println("Response:", response, "after time", since, "\nCluster:", response.Meta.ClusterID)
							}()
						}
					}
				},
			},
		},
	}

	root.AddCommand("help", root.Help())

	root.Run()
}

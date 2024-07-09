package main

import (
	"context"
	"encoding/json"
	"errors"
	"flag"
	"fmt"
	"os"
	"strconv"
	"strings"
	"time"

	"github.com/anti-raid/splashtail/core/go.std/animusmagic"
	"github.com/davecgh/go-spew/spew"
	"github.com/infinitybotlist/eureka/shellcli"
	"github.com/redis/rueidis"
	"go.uber.org/zap"
	"go.uber.org/zap/zapcore"
)

type AnimusCliData struct {
	Context           context.Context
	ContextClose      context.CancelFunc
	Rueidis           rueidis.Client
	AnimusMagicClient *animusmagic.AnimusMagicClient
	Connected         bool
	Logger            *zap.Logger
}

func prettyPrintAnimusMessageMetadata(meta *animusmagic.AnimusMessageMetadata) string {
	str := strings.Builder{}

	str.WriteString("From: " + meta.From.String() + "\n")
	str.WriteString("To: " + meta.To.String() + "\n")
	str.WriteString("Op: " + meta.Op.String() + "\n")
	str.WriteString("ClusterIDFrom: " + strconv.Itoa(int(meta.ClusterIDFrom)) + "\n")
	str.WriteString("ClusterIDTo: " + strconv.Itoa(int(meta.ClusterIDTo)) + "\n")
	str.WriteString("CommandID: " + meta.CommandID + "\n")
	str.WriteString("PayloadOffset: " + strconv.Itoa(int(meta.PayloadOffset)))

	return str.String()
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
					{"from", "Source", "0x3 (AnimusTargetInfra)"},
					{"fromClusterId", "Source Cluster ID. Uses 0 as it is unused", "1"},
				},
				Run: func(a *shellcli.ShellCli[AnimusCliData], args map[string]string) error {
					if a.Data != nil && a.Data.Connected {
						return fmt.Errorf("already connected")
					}

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

					var target = animusmagic.AnimusTargetInfra

					if from, ok := args["from"]; ok && from != "" {
						targetNew, ok := animusmagic.StringToAnimusTarget(from)

						if !ok {
							return fmt.Errorf("invalid source")
						}

						target = targetNew
					}

					var fromClusterId int

					if fromClusterIdStr, ok := args["fromClusterId"]; ok && fromClusterIdStr != "" {
						fromClusterIdInt, err := strconv.Atoi(fromClusterIdStr)

						if err != nil {
							return fmt.Errorf("error converting fromClusterId to integer: %s", err)
						}

						fromClusterId = fromClusterIdInt
					}

					a.Data.AnimusMagicClient = animusmagic.New(animusMagicChannel, animusmagic.AnimusTarget(target), uint16(fromClusterId))

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

						if err != nil && !errors.Is(err, context.Canceled) {
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
					toTarget, ok := animusmagic.StringToAnimusTarget(to)

					if !ok {
						return fmt.Errorf("invalid target")
					}

					timeoutInt, err := strconv.Atoi(timeout)

					if err != nil {
						return fmt.Errorf("error converting timeout to integer: %s", err)
					}

					commandId := animusmagic.NewCommandId()
					payload, err := a.Data.AnimusMagicClient.CreatePayload(
						a.Data.AnimusMagicClient.Identify,
						toTarget,
						a.Data.AnimusMagicClient.ClusterID,
						animusmagic.WildcardClusterID,
						animusmagic.OpProbe,
						commandId,
						[]byte{},
					)

					if err != nil {
						return fmt.Errorf("error creating payload: %s", err)
					}

					fmt.Println("Probing with commandId:", commandId, "and timeout:", timeoutInt, " seconds")

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
								// Try parsing the response
								var resp any

								err := animusmagic.DeserializeData(response.RawPayload, &resp)

								fmt.Print(
									prettyPrintAnimusMessageMetadata(response.Meta),
									"\nElapsed Time: ", since,
									"\nResponse: ", resp,
									"\n[DeserializeErrors]:", err,
									"\n\n",
								)
							}()
						}
					}
				},
			},
			"ping": {
				Description: "Pings redis",
				Args: [][3]string{
					{"to", "Target", "redis"},
				},
				Run: func(a *shellcli.ShellCli[AnimusCliData], args map[string]string) error {
					if !a.Data.Connected {
						return fmt.Errorf("not connected")
					}

					to, ok := args["to"]

					if !ok {
						to = "redis"
					}

					switch to {
					case "redis":
						ts1 := time.Now()
						_, err := a.Data.Rueidis.Do(a.Data.Context, a.Data.Rueidis.B().Ping().Build()).AsBytes()

						if err != nil {
							return fmt.Errorf("error pinging redis: %s", err)
						}

						ts2 := time.Now()

						fmt.Println("Latency: ", ts2.Sub(ts1))

						return nil
					default:
						return fmt.Errorf("invalid target")
					}
				},
			},
			"observe": {
				Description: "Observes the animus magic channel",
				Args: [][3]string{
					{"timeout", "Timeout in seconds", ""},
				},
				Run: func(a *shellcli.ShellCli[AnimusCliData], args map[string]string) error {
					if !a.Data.Connected {
						return fmt.Errorf("not connected")
					}

					timeout, ok := args["timeout"]

					if !ok {
						timeout = ""
					}

					var timeoutInt int

					if timeout != "" {
						var err error
						timeoutInt, err = strconv.Atoi(timeout)

						if err != nil {
							return fmt.Errorf("error converting timeout to integer: %s", err)
						}
					}

					type ObservableRequest struct {
						Meta *animusmagic.AnimusMessageMetadata

						// The raw payload
						RawPayload []byte

						// Time since last message
						TimeSince time.Duration
					}

					c := make(chan *ObservableRequest)

					var isReady bool

					restoreCtx := func() {
						a.Data.AnimusMagicClient.OnMiddleware = nil
						a.Data.AnimusMagicClient.AllowAll = false
					}

					defer restoreCtx()

					a.Data.AnimusMagicClient.AllowAll = true
					var lastMessage = time.Now()
					a.Data.AnimusMagicClient.OnMiddleware = func(meta *animusmagic.AnimusMessageMetadata, payload []byte) (bool, error) {
						newLm := time.Now()
						timeSince := newLm.Sub(lastMessage)
						lastMessage = newLm

						if !isReady {
							return false, nil
						}

						c <- &ObservableRequest{
							Meta:       meta,
							RawPayload: payload,
							TimeSince:  timeSince,
						}
						return false, nil
					}

					timeoutCtx, closer := context.WithCancel(context.Background())

					if timeout != "" {
						go func() {
							time.Sleep(time.Second * time.Duration(timeoutInt))
							closer()
						}()
					}

					for {
						isReady = true
						select {
						case <-a.Data.Context.Done():
							close(c)
							closer()
							return fmt.Errorf("context cancelled")
						case <-timeoutCtx.Done():
							close(c)
							closer()
							return nil
						case response := <-c:
							fmt.Println(
								prettyPrintAnimusMessageMetadata(response.Meta),
								"\nTime since last message: ", response.TimeSince,
								"\nPayload: ", string(response.RawPayload),
							)
						}
					}
				},
			},
			"request": {
				Description: "Sends a request to animus magic",
				Args: [][3]string{
					{"to", "Target", ""},
					{"clusterIdTo", "Cluster ID to send to", ""},
					{"op", "Operation", "0"},
					{"key", "Animuc magic request key", ""},
					{"[fields]", "Fields in format a=b", ""},
					{"timeout", "Timeout in seconds", "5"},
				},
				Run: func(a *shellcli.ShellCli[AnimusCliData], args map[string]string) error {
					if !a.Data.Connected {
						return fmt.Errorf("not connected")
					}

					timeout, ok := args["timeout"]

					if !ok {
						timeout = "5"
					}

					var timeoutInt int

					if timeout != "" {
						var err error
						timeoutInt, err = strconv.Atoi(timeout)

						if err != nil {
							return fmt.Errorf("error converting timeout to integer: %s", err)
						}
					}

					opStr, ok := args["op"]

					if !ok {
						opStr = "0"
					}

					op, ok := animusmagic.StringToAnimusOp(opStr)

					if !ok {
						return fmt.Errorf("invalid operation")
					}

					key, ok := args["key"]

					if !ok {
						return fmt.Errorf("key is required to create payload")
					}

					to, ok := args["to"]

					if !ok {
						return fmt.Errorf("to is required [bot/jobs/webserver/infra etc.]")
					}

					toTarget, ok := animusmagic.StringToAnimusTarget(to)

					if !ok {
						return fmt.Errorf("invalid target")
					}

					clusterIdTo, ok := args["clusterIdTo"]

					if !ok {
						return fmt.Errorf("clusterIdTo is required. Use 'wildcard' for wildcard cluster")
					}

					var clusterIdToInt int
					var err error

					if clusterIdTo == "wildcard" {
						clusterIdToInt = animusmagic.WildcardClusterID
					} else {
						clusterIdToInt, err = strconv.Atoi(clusterIdTo)

						if err != nil {
							return fmt.Errorf("error converting clusterIdTo to integer: %s", err)
						}
					}

					var data = map[string]any{}

					for arg, value := range args {
						if arg != "to" && arg != "op" && arg != "key" && arg != "timeout" && arg != "clusterIdTo" {
							argSplit := strings.Split(arg, "@")

							if len(argSplit) == 1 {
								argSplit = []string{argSplit[0], "str"}
							}

							var valueParsed any
							switch argSplit[1] {
							case "string":
								fallthrough
							case "str":
								valueParsed = value
							case "string[]":
								fallthrough
							case "str[]":
								valueParsed = strings.Split(value, ",")

								for i, v := range valueParsed.([]string) {
									valueParsed.([]string)[i] = strings.TrimSpace(v)
								}
							case "int":
								valueParsedInt, err := strconv.Atoi(value)

								if err != nil {
									return fmt.Errorf("error parsing int: %s", err)
								}

								valueParsed = valueParsedInt
							case "float":
								valueParsedFloat, err := strconv.ParseFloat(value, 64)

								if err != nil {
									return fmt.Errorf("error parsing float: %s", err)
								}

								valueParsed = valueParsedFloat
							case "bool":
								valueParsedBool, err := strconv.ParseBool(value)

								if err != nil {
									return fmt.Errorf("error parsing bool: %s", err)
								}

								valueParsed = valueParsedBool
							case "json":
								err := json.Unmarshal([]byte(value), &valueParsed)

								if err != nil {
									return fmt.Errorf("error parsing json: %s", err)
								}
							default:
								return fmt.Errorf("invalid type for key %s: %s", value, argSplit[1])
							}

							data[argSplit[0]] = valueParsed
						}
					}

					// Construct payload
					commandId := animusmagic.NewCommandId()
					initData := map[string]any{
						key: data,
					}
					payload, err := a.Data.AnimusMagicClient.CreatePayload(
						a.Data.AnimusMagicClient.Identify,
						toTarget,
						a.Data.AnimusMagicClient.ClusterID,
						uint16(clusterIdToInt),
						op,
						commandId,
						initData,
					)

					if err != nil {
						return fmt.Errorf("error creating payload: %s", err)
					}

					fmt.Println(spew.Sdump(initData))
					fmt.Println(spew.Sdump(payload))
					fmt.Println("Sending request with commandId:", commandId, "and timeout:", timeoutInt, " seconds")

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
								// Try parsing the response
								var resp any

								err := animusmagic.DeserializeData(response.RawPayload, &resp)

								fmt.Print(
									prettyPrintAnimusMessageMetadata(response.Meta),
									"\nElapsed Time: ", since,
									"\nResponse: ", resp,
									"\nDeserializeErrors:", err,
									"\n\n",
								)
							}()
						}
					}
				},
			},
		},
	}

	root.AddCommand("help", root.Help())

	// Handle --command args
	command := flag.String("command", "", "Command to run. If unset, will run as shell")
	flag.Parse()

	if command != nil && *command != "" {
		err := root.Init()

		if err != nil {
			fmt.Println("Error initializing cli: ", err)
			os.Exit(1)
		}

		cancel := root.ExecuteCommands(*command)
		if cancel {
			fmt.Println("Exiting")
		}

		return
	}

	root.Run()
}

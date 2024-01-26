package main

import (
	"fmt"
	"net/http"
	"os"
	"os/exec"
	"os/signal"
	"runtime"
	"strconv"
	"syscall"
	"time"

	mconfig "github.com/cheesycod/mewld/config"
	mloader "github.com/cheesycod/mewld/loader"
	mproc "github.com/cheesycod/mewld/proc"
	mutils "github.com/cheesycod/mewld/utils"
	"gopkg.in/yaml.v3"

	"github.com/anti-raid/splashtail/config"
	"github.com/anti-raid/splashtail/jobserver"
	"github.com/anti-raid/splashtail/jobserver/bgtasks"
	"github.com/anti-raid/splashtail/mewld_web"
	"github.com/anti-raid/splashtail/state"
	"github.com/anti-raid/splashtail/webserver"

	"github.com/cloudflare/tableflip"
	"go.uber.org/zap"

	_ "embed"
)

func main() {
	state.Setup()

	if len(os.Args) < 2 {
		os.Args = append(os.Args, "help")
	}

	state.CurrentOperationMode = os.Args[1]

	switch os.Args[1] {
	case "webserver":
		r := webserver.CreateWebserver()

		// Load mewld bot
		mldF, err := os.ReadFile("mewld-" + config.CurrentEnv + ".yaml")

		if err != nil {
			panic(err)
		}

		var mldConfig mconfig.CoreConfig

		err = yaml.Unmarshal(mldF, &mldConfig)

		if err != nil {
			panic(err)
		}

		mldConfig.Token = state.Config.DiscordAuth.Token
		mldConfig.Oauth = mconfig.Oauth{
			ClientID:     state.Config.DiscordAuth.ClientID,
			ClientSecret: state.Config.DiscordAuth.ClientSecret,
			RedirectURL:  state.Config.DiscordAuth.MewldRedirect,
		}

		if mldConfig.Redis != state.Config.Meta.RedisURL.Parse() {
			panic("Redis URL in mewld.yaml does not match the one in config.yaml")
		}

		il, rh, err := mloader.Load(&mldConfig, &mproc.LoaderData{
			Start: func(l *mproc.InstanceList, i *mproc.Instance, cm *mproc.ClusterMap) error {
				var cmd *exec.Cmd
				if l.Config.Interp != "" {
					cmd = exec.Command(
						l.Config.Interp,
						l.Dir+"/"+l.Config.Module,
						mutils.ToPyListUInt64(i.Shards),
						mutils.UInt64ToString(l.ShardCount),
						strconv.Itoa(i.ClusterID),
						cm.Name,
						l.Dir,
						strconv.Itoa(len(l.Map)),
						state.Config.Sites.API.Parse(),
						l.Config.RedisChannel,
						config.CurrentEnv,
					)
				} else {
					cmd = exec.Command(
						l.Dir+"/"+l.Config.Module,
						mutils.ToPyListUInt64(i.Shards),
						mutils.UInt64ToString(l.ShardCount),
						strconv.Itoa(i.ClusterID),
						cm.Name,
						l.Dir,
						strconv.Itoa(len(l.Map)),
						state.Config.Sites.API.Parse(),
						l.Config.RedisChannel,
						config.CurrentEnv,
					)
				}

				cmd.Stdout = os.Stdout
				cmd.Stderr = os.Stderr

				env := os.Environ()

				env = append(env, "MEWLD_CHANNEL="+l.Config.RedisChannel)
				env = append(env, "REDIS_URL="+state.Config.Meta.RedisURL.Parse())

				cmd.Env = env
				cmd.Dir = l.Dir

				i.Command = cmd

				// Spawn process
				return cmd.Start()
			},
		})

		if err != nil {
			panic(err)
		}

		defer func() {
			a := recover()

			if a != nil {
				il.KillAll()
			}
		}()

		state.MewldInstanceList = il

		r.Mount("/mewld", mewld_web.CreateServer(mewld_web.WebData{
			RedisHandler: rh,
			InstanceList: il,
		}))

		go state.AnimusMagicClient.Listen(state.Context, state.Rueidis, state.Logger)

		// If GOOS is windows, do normal http server
		if runtime.GOOS == "linux" || runtime.GOOS == "darwin" {
			upg, _ := tableflip.New(tableflip.Options{})
			defer upg.Stop()

			go func() {
				sig := make(chan os.Signal, 1)
				signal.Notify(sig, syscall.SIGHUP)
				for range sig {
					state.Logger.Info("Received SIGHUP, upgrading server")
					upg.Upgrade()
				}
			}()

			// Listen must be called before Ready
			ln, err := upg.Listen("tcp", ":"+strconv.Itoa(state.Config.Meta.Port.Parse()))

			if err != nil {
				il.KillAll()
				state.Logger.Fatal("Error binding to socket", zap.Error(err))
			}

			defer ln.Close()

			server := http.Server{
				ReadTimeout: 30 * time.Second,
				Handler:     r,
			}

			go func() {
				err := server.Serve(ln)
				if err != http.ErrServerClosed {
					state.Logger.Error("Server failed due to unexpected error", zap.Error(err))
				}
			}()

			if err := upg.Ready(); err != nil {
				state.Logger.Fatal("Error calling upg.Ready", zap.Error(err))
			}

			<-upg.Exit()
		} else {
			// Tableflip not supported
			state.Logger.Warn("Tableflip not supported on this platform, this is not a production-capable server.")
			err = http.ListenAndServe(":"+strconv.Itoa(state.Config.Meta.Port.Parse()), r)

			if err != nil {
				il.KillAll()
				state.Logger.Fatal("Error binding to socket", zap.Error(err))
			}
		}
	case "jobs":
		mldF, err := os.ReadFile("mewld-" + config.CurrentEnv + ".yaml")

		if err != nil {
			panic(err)
		}

		var mldConfig mconfig.CoreConfig

		err = yaml.Unmarshal(mldF, &mldConfig)

		if err != nil {
			panic(err)
		}

		state.MewldInstanceList = &mproc.InstanceList{
			Config: &mldConfig,
		}

		// Set state of all pending tasks to 'failed'
		_, err = state.Pool.Exec(state.Context, "UPDATE tasks SET state = $1 WHERE state = $2", "failed", "pending")

		if err != nil {
			panic(err)
		}

		r := jobserver.CreateJobServer()

		// Load jobs
		bgtasks.StartAllTasks()

		// If GOOS is windows, do normal http server
		if runtime.GOOS == "linux" || runtime.GOOS == "darwin" {
			upg, _ := tableflip.New(tableflip.Options{})
			defer upg.Stop()

			go func() {
				sig := make(chan os.Signal, 1)
				signal.Notify(sig, syscall.SIGHUP)
				for range sig {
					state.Logger.Info("Received SIGHUP, upgrading server")
					upg.Upgrade()
				}
			}()

			// Listen must be called before Ready
			ln, err := upg.Listen("tcp", ":"+strconv.Itoa(state.Config.Meta.JobserverPort.Parse()))

			if err != nil {
				state.Logger.Fatal("Error binding to socket", zap.Error(err))
			}

			defer ln.Close()

			server := http.Server{
				ReadTimeout: 30 * time.Second,
				Handler:     r,
			}

			go func() {
				err := server.Serve(ln)
				if err != http.ErrServerClosed {
					state.Logger.Error("Server failed due to unexpected error", zap.Error(err))
				}
			}()

			if err := upg.Ready(); err != nil {
				state.Logger.Fatal("Error calling upg.Ready", zap.Error(err))
			}

			<-upg.Exit()
		} else {
			// Tableflip not supported
			state.Logger.Warn("Tableflip not supported on this platform, this is not a production-capable server.")
			err = http.ListenAndServe(":"+strconv.Itoa(state.Config.Meta.JobserverPort.Parse()), r)

			if err != nil {
				state.Logger.Fatal("Error binding to socket", zap.Error(err))
			}
		}
	default:
		fmt.Println("Splashtail Usage: splashtail <component>")
		fmt.Println("webserver: Starts the webserver")
		fmt.Println("jobs: Starts the jobserver (currently includes IPC as well)")
		os.Exit(1)
	}
}

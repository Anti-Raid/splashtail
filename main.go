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

	"github.com/anti-raid/splashtail/jobserver"
	"github.com/anti-raid/splashtail/jobserver/bgtasks"
	jobserverstate "github.com/anti-raid/splashtail/jobserver/state"
	"github.com/anti-raid/splashtail/splashcore/config"
	"github.com/anti-raid/splashtail/webserver"
	"github.com/anti-raid/splashtail/webserver/mewld_web"
	webserverstate "github.com/anti-raid/splashtail/webserver/state"

	"github.com/cloudflare/tableflip"
	"go.uber.org/zap"

	_ "embed"
)

func main() {
	if len(os.Args) < 2 {
		os.Args = append(os.Args, "help")
	}

	switch os.Args[1] {
	case "webserver":
		webserverstate.Setup()

		webserverstate.CurrentOperationMode = os.Args[1]

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

		mldConfig.Token = webserverstate.Config.DiscordAuth.Token
		mldConfig.Oauth = mconfig.Oauth{
			ClientID:     webserverstate.Config.DiscordAuth.ClientID,
			ClientSecret: webserverstate.Config.DiscordAuth.ClientSecret,
			RedirectURL:  webserverstate.Config.DiscordAuth.MewldRedirect,
		}

		if mldConfig.Redis != webserverstate.Config.Meta.RedisURL.Parse() {
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
						webserverstate.Config.Sites.API.Parse(),
						l.Config.RedisChannel,
						config.CurrentEnv,
						webserverstate.Config.Meta.AnimusMagicChannel.Parse(),
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
						webserverstate.Config.Sites.API.Parse(),
						l.Config.RedisChannel,
						config.CurrentEnv,
						webserverstate.Config.Meta.AnimusMagicChannel.Parse(),
					)
				}

				cmd.Stdout = os.Stdout
				cmd.Stderr = os.Stderr

				env := os.Environ()

				env = append(env, "MEWLD_CHANNEL="+l.Config.RedisChannel)
				env = append(env, "REDIS_URL="+webserverstate.Config.Meta.RedisURL.Parse())

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

		webserverstate.MewldInstanceList = il

		r.Mount("/mewld", mewld_web.CreateServer(mewld_web.WebData{
			RedisHandler: rh,
			InstanceList: il,
		}))

		go webserverstate.AnimusMagicClient.Listen(webserverstate.Context, webserverstate.Rueidis, webserverstate.Logger)

		// If GOOS is windows, do normal http server
		if runtime.GOOS == "linux" || runtime.GOOS == "darwin" {
			upg, _ := tableflip.New(tableflip.Options{})
			defer upg.Stop()

			go func() {
				sig := make(chan os.Signal, 1)
				signal.Notify(sig, syscall.SIGHUP)
				for range sig {
					webserverstate.Logger.Info("Received SIGHUP, upgrading server")
					upg.Upgrade()
				}
			}()

			// Listen must be called before Ready
			ln, err := upg.Listen("tcp", ":"+strconv.Itoa(webserverstate.Config.Meta.Port.Parse()))

			if err != nil {
				il.KillAll()
				webserverstate.Logger.Fatal("Error binding to socket", zap.Error(err))
			}

			defer ln.Close()

			server := http.Server{
				ReadTimeout: 30 * time.Second,
				Handler:     r,
			}

			go func() {
				err := server.Serve(ln)
				if err != http.ErrServerClosed {
					webserverstate.Logger.Error("Server failed due to unexpected error", zap.Error(err))
				}
			}()

			if err := upg.Ready(); err != nil {
				webserverstate.Logger.Fatal("Error calling upg.Ready", zap.Error(err))
			}

			<-upg.Exit()
		} else {
			// Tableflip not supported
			webserverstate.Logger.Warn("Tableflip not supported on this platform, this is not a production-capable server.")
			err = http.ListenAndServe(":"+strconv.Itoa(webserverstate.Config.Meta.Port.Parse()), r)

			if err != nil {
				il.KillAll()
				webserverstate.Logger.Fatal("Error binding to socket", zap.Error(err))
			}
		}
	case "jobs":
		jobserverstate.Setup()
		jobserverstate.CurrentOperationMode = os.Args[1]

		// Set state of all pending tasks to 'failed'
		_, err := jobserverstate.Pool.Exec(jobserverstate.Context, "UPDATE tasks SET state = $1 WHERE state = $2", "failed", "pending")

		if err != nil {
			panic(err)
		}

		jobserver.CreateJobServer()

		// Load jobs
		bgtasks.StartAllTasks()
	default:
		fmt.Println("Splashtail Usage: splashtail <component>")
		fmt.Println("webserver: Starts the webserver")
		fmt.Println("jobs: Starts the jobserver (currently includes IPC as well)")
		os.Exit(1)
	}
}

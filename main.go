package main

import (
	"fmt"
	"net/http"
	"os"
	"os/exec"
	"os/signal"
	"runtime"
	"strconv"
	"strings"
	"syscall"
	"time"

	"github.com/bwmarrin/discordgo"
	mconfig "github.com/cheesycod/mewld/config"
	mloader "github.com/cheesycod/mewld/loader"
	mproc "github.com/cheesycod/mewld/proc"
	mutils "github.com/cheesycod/mewld/utils"
	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	"github.com/go-playground/validator/v10"
	"github.com/infinitybotlist/eureka/genconfig"
	"github.com/infinitybotlist/eureka/jsonimpl"
	"github.com/infinitybotlist/eureka/snippets"
	"gopkg.in/yaml.v3"

	"github.com/anti-raid/splashtail/jobs/jobserver"
	"github.com/anti-raid/splashtail/jobs/jobserver/bgtasks"
	jobserverstate "github.com/anti-raid/splashtail/jobs/jobserver/state"
	"github.com/anti-raid/splashtail/splashcore/config"
	"github.com/anti-raid/splashtail/splashcore/mewldresponder"
	"github.com/anti-raid/splashtail/splashcore/utils"
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
			err := http.ListenAndServe(":"+strconv.Itoa(webserverstate.Config.Meta.Port.Parse()), r)

			if err != nil {
				webserverstate.Logger.Fatal("Error binding to socket", zap.Error(err))
			}
		}
	case "bot":
		genconfig.GenConfig(config.Config{})

		cfgFile, err := os.ReadFile("config.yaml")

		if err != nil {
			panic(err)
		}

		var cfg config.Config
		err = yaml.Unmarshal(cfgFile, &cfg)

		if err != nil {
			panic(err)
		}

		var v = validator.New()

		err = v.Struct(cfg)

		if err != nil {
			panic("configError: " + err.Error())
		}

		logger := snippets.CreateZap()

		// Load mewld bot
		mldF, err := os.ReadFile("data/mewld/botv2-" + config.CurrentEnv + ".yaml")

		if err != nil {
			panic(err)
		}

		var mldConfig mconfig.CoreConfig

		err = yaml.Unmarshal(mldF, &mldConfig)

		if err != nil {
			panic(err)
		}

		mldConfig.Proxy = cfg.Meta.Proxy.Parse()
		mldConfig.Token = cfg.DiscordAuth.Token
		mldConfig.Oauth = mconfig.Oauth{
			ClientID:     cfg.DiscordAuth.ClientID,
			ClientSecret: cfg.DiscordAuth.ClientSecret,
			RedirectURL:  cfg.DiscordAuth.MewldRedirect,
		}

		if mldConfig.Redis == "" {
			mldConfig.Redis = cfg.Meta.RedisURL.Parse()
		}

		if mldConfig.Redis != cfg.Meta.RedisURL.Parse() {
			webserverstate.Logger.Warn("Redis URL in mewld.yaml does not match the one in config.yaml")
		}

		webh, err := utils.ParseWebhookURL(cfg.Wafflepaw.StatusWebhook)

		if err != nil {
			logger.Fatal("Error parsing webhook URL", zap.Error(err))
		}

		il, rh, err := mloader.Load(&mldConfig, &mproc.LoaderData{
			Start: func(l *mproc.InstanceList, i *mproc.Instance, cm *mproc.ClusterMap) error {
				cmd := exec.Command(
					l.Dir+"/"+l.Config.Module,
					mutils.ToPyListUInt64(i.Shards),
					mutils.UInt64ToString(l.ShardCount),
					strconv.Itoa(i.ClusterID),
					cm.Name,
					strconv.Itoa(len(l.Map)),
					l.Config.RedisChannel,
					config.CurrentEnv,
					cfg.Meta.AnimusMagicChannel.Parse(),
				)

				cmd.Stdout = os.Stdout
				cmd.Stderr = os.Stderr

				env := os.Environ()

				env = append(env, "MEWLD_CHANNEL="+l.Config.RedisChannel)
				env = append(env, "REDIS_URL="+cfg.Meta.RedisURL.Parse())

				cmd.Env = env
				cmd.Dir = l.Dir

				i.Command = cmd

				// Spawn process
				return cmd.Start()
			},
			OnActionLog: func(payload map[string]any) error {
				// Send webhook
				go func() {
					payloadStr := strings.Builder{}

					for k, v := range payload {
						payloadStr.WriteString(k + ": " + fmt.Sprint(v) + "\n")
					}

					_, err := webserverstate.Discord.WebhookExecute(
						webh.ID,
						webh.Token,
						false,
						&discordgo.WebhookParams{
							Content: "@everyone **MEWLD ALERT [webserver]**\n" + payloadStr.String(),
						},
					)

					if err != nil {
						logger.Error("Error sending webhook", zap.Error(err))
					}
				}()

				return nil
			},
		})

		if err != nil {
			panic(err)
		}

		killInstanceList := func() {
			time.Sleep(5 * time.Second)

			il.KillAll()

			for _, instance := range il.Instances {
				if instance.Command != nil {
					logger.Info("Waiting for instance to exit", zap.Int("clusterId", instance.ClusterID))
					instance.Command.Wait()
				}
			}
		}

		defer func() {
			a := recover()

			if a != nil {
				killInstanceList()
			}
		}()

		r := chi.NewMux()

		r.Use(
			middleware.Recoverer,
			//zapchi.Logger(logger, "bot"),
			middleware.Timeout(30*time.Second),
		)

		r.Get("/getMewldInstanceList", func(w http.ResponseWriter, r *http.Request) {
			bytes, err := jsonimpl.Marshal(il)

			if err != nil {
				w.WriteHeader(http.StatusInternalServerError)
				w.Write([]byte("Error marshalling instance list"))
				return
			}

			w.Write(bytes)
		})

		r.Mount("/mewld", mewld_web.CreateServer(mewld_web.WebData{
			RedisHandler: rh,
			InstanceList: il,
		}))

		err = http.ListenAndServe(":"+strconv.Itoa(cfg.Meta.BotPort.Parse()), r)

		if err != nil {
			logger.Fatal("Error binding to socket", zap.Error(err))
		}
	case "jobs":
		jobserverstate.SetupBase()

		wmldF, err := os.ReadFile("data/mewld/botv2-" + config.CurrentEnv + ".yaml")

		if err != nil {
			panic(err)
		}

		var wmldConfig mconfig.CoreConfig

		err = yaml.Unmarshal(wmldF, &wmldConfig)

		if err != nil {
			panic(err)
		}

		// Load mewld bot
		mldF, err := os.ReadFile("data/mewld/jobs-" + config.CurrentEnv + ".yaml")

		if err != nil {
			panic(err)
		}

		var mldConfig mconfig.CoreConfig

		err = yaml.Unmarshal(mldF, &mldConfig)

		if err != nil {
			panic(err)
		}

		jobserverstate.Logger.Info("Setting up mewld")

		mldConfig.Proxy = jobserverstate.Config.Meta.Proxy.Parse()
		mldConfig.Token = jobserverstate.Config.DiscordAuth.Token
		mldConfig.Oauth = mconfig.Oauth{
			ClientID:     jobserverstate.Config.DiscordAuth.ClientID,
			ClientSecret: jobserverstate.Config.DiscordAuth.ClientSecret,
			RedirectURL:  jobserverstate.Config.DiscordAuth.MewldRedirect,
		}

		if mldConfig.Redis == "" {
			mldConfig.Redis = jobserverstate.Config.Meta.RedisURL.Parse()
		}

		if mldConfig.Redis != jobserverstate.Config.Meta.RedisURL.Parse() {
			jobserverstate.Logger.Warn("Redis URL in mewld.yaml does not match the one in config.yaml")
		}

		for _, clusterName := range wmldConfig.Names {
			var i uint64
			for i < wmldConfig.PerCluster {
				mldConfig.Names = append(mldConfig.Names, clusterName+"@"+strconv.FormatUint(i, 10))
				i++
			}
		}

		webh, err := utils.ParseWebhookURL(jobserverstate.Config.Wafflepaw.StatusWebhook)

		if err != nil {
			jobserverstate.Logger.Fatal("Error parsing webhook URL", zap.Error(err))
		}

		il, rh, err := mloader.Load(&mldConfig, &mproc.LoaderData{
			Start: func(l *mproc.InstanceList, i *mproc.Instance, cm *mproc.ClusterMap) error {
				cmd := exec.Command(
					os.Args[0],
					"jobs.node",
					mutils.ToPyListUInt64(i.Shards),
					strconv.Itoa(i.ClusterID),
					cm.Name,
					mldConfig.RedisChannel,
					mutils.UInt64ToString(l.ShardCount),
				)

				cmd.Stdout = os.Stdout
				cmd.Stderr = os.Stderr

				env := os.Environ()

				env = append(env, "MEWLD_CHANNEL="+l.Config.RedisChannel)
				env = append(env, "REDIS_URL="+jobserverstate.Config.Meta.RedisURL.Parse())

				cmd.Env = env
				cmd.Dir = l.Dir

				i.Command = cmd

				// Spawn process
				return cmd.Start()
			},
			OnActionLog: func(payload map[string]any) error {
				// Send webhook
				go func() {
					payloadStr := strings.Builder{}

					for k, v := range payload {
						payloadStr.WriteString(k + ": " + fmt.Sprint(v) + "\n")
					}

					_, err := jobserverstate.Discord.WebhookExecute(
						webh.ID,
						webh.Token,
						false,
						&discordgo.WebhookParams{
							Content: "@everyone **MEWLD ALERT [jobserver]**\n" + payloadStr.String(),
						},
					)

					if err != nil {
						jobserverstate.Logger.Error("Error sending webhook", zap.Error(err))
					}
				}()

				return nil
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

		r := chi.NewMux()

		r.Mount("/mewld", mewld_web.CreateServer(mewld_web.WebData{
			RedisHandler: rh,
			InstanceList: il,
		}))

		// Tableflip not supported
		jobserverstate.Logger.Warn("Tableflip not supported on this platform, this is not a production-capable server.")
		err = http.ListenAndServe(":"+strconv.Itoa(jobserverstate.Config.Meta.JobserverPort.Parse()), r)

		if err != nil {
			il.KillAll()
			jobserverstate.Logger.Fatal("Error binding to socket", zap.Error(err))
		}
	case "jobs.node":
		jobserverstate.CurrentOperationMode = "jobs"

		// Read cmd args
		if len(os.Args) < 7 {
			panic("Not enough arguments. Expected <cmd> jobs.node <shards> <clusterID> <clusterName> <redisChannel> <shard count>")
		}

		shardsStr := os.Args[2]

		var shards []uint16

		err := jsonimpl.Unmarshal([]byte(shardsStr), &shards)
		if err != nil {
			panic(err)
		}

		jobserverstate.Shard = shards[0]

		clusterId := os.Args[3]
		clusterIdInt, err := strconv.Atoi(clusterId)
		if err != nil {
			panic(err)
		}

		jobserverstate.ClusterID = uint16(clusterIdInt)

		clusterName := os.Args[4]
		jobserverstate.ClusterName = clusterName

		redisChannel := os.Args[5]

		shardCount := os.Args[6]

		shardCountInt, err := strconv.Atoi(shardCount)

		if err != nil {
			panic(err)
		}

		jobserverstate.ShardCount = uint16(shardCountInt)

		jobserverstate.Setup()

		jobserverstate.Logger = jobserverstate.Logger.With(zap.Uint16("shard", jobserverstate.Shard), zap.Int("clusterId", clusterIdInt), zap.String("clusterName", clusterName))

		jobserverstate.Logger.Info("Starting node")

		jobserverstate.MewldResponder = &mewldresponder.MewldResponder{
			ClusterID:   jobserverstate.ClusterID,
			ClusterName: jobserverstate.ClusterName,
			Shards:      shards,
			Channel:     redisChannel,
			OnDiag: func(payload *mewldresponder.MewldDiagPayload) (*mewldresponder.MewldDiagResponse, error) {
				data := []mewldresponder.MewldDiagShardHealth{
					{
						ShardID: jobserverstate.Shard,
						Up:      true, // TODO: Check if shard is up once we add dgo
						Latency: 0,    // TODO: Get shard latency once we add dgo
						Guilds:  0,    // TODO: Get shard guild count once we add dgo
						Users:   0,    // TODO: Get shard user count once we add dgo
					},
				}

				return &mewldresponder.MewldDiagResponse{
					ClusterID: jobserverstate.ClusterID,
					Nonce:     payload.Nonce,
					Data:      data,
				}, nil
			},
		}

		jobserver.CreateJobServer()

		// Load jobs
		bgtasks.StartAllTasks()

		// Handle mewld by starting ping checks and sending launch_next
		go func() {
			err := jobserverstate.MewldResponder.LaunchNext(jobserverstate.Context, jobserverstate.Rueidis, jobserverstate.Logger)

			if err != nil {
				jobserverstate.Logger.Fatal("Error sending launch_next command", zap.Error(err))
				return
			}

			jobserverstate.Logger.Info("Sent launch_next command")
		}()

		go jobserverstate.MewldResponder.Listen(jobserverstate.Context, jobserverstate.Rueidis, jobserverstate.Logger)

		// Wait until signal is received
		c := make(chan os.Signal, 1)

		signal.Notify(c, syscall.SIGTERM, syscall.SIGINT, syscall.SIGHUP)

		<-c
	default:
		fmt.Println("Splashtail Usage: splashtail <component>")
		fmt.Println("webserver: Starts the webserver")
		fmt.Println("bot: Starts the bot")
		fmt.Println("jobs: Starts all nodes of the jobserver")
		fmt.Println("jobs.node: Starts a node for the jobserver. This is meant to be executed by mewld when using the jobs command. Currently a node can only service *one* shard")
		os.Exit(1)
	}
}

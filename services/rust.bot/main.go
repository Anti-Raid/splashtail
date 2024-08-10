package main

import (
	"fmt"
	"net/http"
	"os"
	"os/exec"
	"strconv"
	"strings"
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

	"go.std/config"
	"go.std/mewld_web"
	"go.std/utils"

	"go.uber.org/zap"

	_ "embed"
)

func main() {
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

	discordSess, err := discordgo.New("Bot " + cfg.DiscordAuth.Token)

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
		logger.Warn("Redis URL in mewld.yaml does not match the one in config.yaml")
	}

	webh, err := utils.ParseWebhookURL(cfg.Wafflepaw.StatusWebhook)

	if err != nil {
		logger.Fatal("Error parsing webhook URL", zap.Error(err))
	}

	il, rh, err := mloader.Load(&mldConfig, &mproc.LoaderData{
		Start: func(l *mproc.InstanceList, i *mproc.Instance, cm *mproc.ClusterMap) error {
			cmd := exec.Command(
				func() string {
					if l.Dir == "" {
						return l.Dir + "/" + l.Config.Module
					}
					return "./" + l.Config.Module
				}(),
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

				_, err := discordSess.WebhookExecute(
					webh.ID,
					webh.Token,
					false,
					&discordgo.WebhookParams{
						Content: "**MEWLD ALERT [bot]**\n" + payloadStr.String(),
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

	mewld_web.SetState(cfg.Meta.DPSecret)
	r.Mount("/mewld", mewld_web.CreateServer(mewld_web.WebData{
		RedisHandler: rh,
		InstanceList: il,
	}))

	err = http.ListenAndServe(":"+strconv.Itoa(cfg.Meta.BotPort.Parse()), r)

	if err != nil {
		logger.Fatal("Error binding to socket", zap.Error(err))
	}
}

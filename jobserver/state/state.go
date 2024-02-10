package state

import (
	"context"
	"net/http"
	"os"
	"reflect"
	"runtime/debug"
	"strings"

	"github.com/anti-raid/splashtail/splashcore/animusmagic"
	"github.com/anti-raid/splashtail/splashcore/config"
	"github.com/anti-raid/splashtail/splashcore/objectstorage"
	"github.com/bwmarrin/discordgo"
	"github.com/go-playground/validator/v10"
	"github.com/go-playground/validator/v10/non-standard/validators"
	"github.com/infinitybotlist/eureka/genconfig"
	"github.com/infinitybotlist/eureka/proxy"
	"github.com/infinitybotlist/eureka/snippets"
	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/redis/rueidis"
	"go.uber.org/zap"
	"gopkg.in/yaml.v3"
)

var (
	Context              = context.Background()
	Config               *config.Config
	Validator            = validator.New()
	BotUser              *discordgo.User
	ObjectStorage        *objectstorage.ObjectStorage
	CurrentOperationMode string // Current mode splashtail is operating in

	Rueidis           rueidis.Client
	AnimusMagicClient *animusmagic.AnimusMagicClient

	// Debug stuff
	BuildInfo  *debug.BuildInfo
	ExtraDebug ExtraDebugInfo

	// Task stuff
	TaskTransport *http.Transport = &http.Transport{}

	Pool    *pgxpool.Pool
	Discord *discordgo.Session
	Logger  *zap.Logger
)

type ExtraDebugInfo struct {
	VSC         string
	VSCRevision string
}

func SetupDebug() {
	var ok bool
	BuildInfo, ok = debug.ReadBuildInfo()

	if !ok {
		panic("failed to read build info")
	}

	// Get vcs.revision
	for _, d := range BuildInfo.Settings {
		if d.Key == "vcs" {
			ExtraDebug.VSC = d.Value
		}
		if d.Key == "vcs.revision" {
			ExtraDebug.VSCRevision = d.Value
		}
	}
}

func nonVulgar(fl validator.FieldLevel) bool {
	// get the field value
	switch fl.Field().Kind() {
	case reflect.String:
		value := fl.Field().String()

		for _, v := range Config.Meta.VulgarList {
			if strings.Contains(value, v) {
				return false
			}
		}
		return true
	default:
		return false
	}
}

func Setup() {
	SetupDebug()
	Validator.RegisterValidation("nonvulgar", nonVulgar)
	Validator.RegisterValidation("notblank", validators.NotBlank)
	Validator.RegisterValidation("nospaces", snippets.ValidatorNoSpaces)
	Validator.RegisterValidation("https", snippets.ValidatorIsHttps)
	Validator.RegisterValidation("httporhttps", snippets.ValidatorIsHttpOrHttps)

	genconfig.GenConfig(config.Config{})

	cfg, err := os.ReadFile("config.yaml")

	if err != nil {
		panic(err)
	}

	err = yaml.Unmarshal(cfg, &Config)

	if err != nil {
		panic(err)
	}

	err = Validator.Struct(Config)

	if err != nil {
		panic("configError: " + err.Error())
	}

	Logger = snippets.CreateZap()

	// Postgres
	Pool, err = pgxpool.New(Context, Config.Meta.PostgresURL)

	if err != nil {
		panic(err)
	}

	// Object Storage
	ObjectStorage, err = objectstorage.New(&Config.ObjectStorage)

	if err != nil {
		panic(err)
	}

	// Discordgo
	Discord, err = discordgo.New("Bot " + Config.DiscordAuth.Token)

	if err != nil {
		panic(err)
	}

	Discord.Client.Transport = proxy.NewHostRewriter("localhost:3219", http.DefaultTransport, func(s string) {
		Logger.Info("[PROXY]", zap.String("note", s))
	})

	// Verify token
	bu, err := Discord.User("@me")

	if err != nil {
		panic(err)
	}

	BotUser = bu

	// Shouldnt be called yet as we don't start websocket
	Discord.AddHandler(func(s *discordgo.Session, r *discordgo.Ready) {
		Logger.Info("[DISCORD]", zap.String("note", "ready"))
	})

	// Reuidis
	ruOptions, err := rueidis.ParseURL(Config.Meta.RedisURL.Parse())

	if err != nil {
		panic(err)
	}

	Rueidis, err = rueidis.NewClient(ruOptions)

	if err != nil {
		panic(err)
	}

	AnimusMagicClient = animusmagic.New(Config.Meta.AnimusMagicChannel.Parse(), animusmagic.AnimusTargetJobserver)
}

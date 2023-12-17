package state

import (
	"context"
	"net/http"
	"os"
	"reflect"
	"runtime/debug"
	"strings"
	"time"

	"github.com/anti-raid/splashtail/config"
	"github.com/anti-raid/splashtail/objectstorage"

	"github.com/bwmarrin/discordgo"
	mproc "github.com/cheesycod/mewld/proc"
	"github.com/go-playground/validator/v10"
	"github.com/go-playground/validator/v10/non-standard/validators"
	"github.com/infinitybotlist/eureka/dovewing"
	"github.com/infinitybotlist/eureka/dovewing/dovetypes"
	"github.com/infinitybotlist/eureka/genconfig"
	hredis "github.com/infinitybotlist/eureka/hotcache/redis"
	"github.com/infinitybotlist/eureka/proxy"
	"github.com/infinitybotlist/eureka/ratelimit"
	"github.com/infinitybotlist/eureka/snippets"
	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/redis/go-redis/v9"
	"github.com/redis/rueidis"
	"go.uber.org/zap"
	"gopkg.in/yaml.v3"
)

var (
	Pool                    *pgxpool.Pool
	Redis                   *redis.Client  // Used by dovewing and other services etc.
	Rueidis                 rueidis.Client // where perf is needed
	DovewingPlatformDiscord *dovewing.DiscordState
	Discord                 *discordgo.Session
	Logger                  *zap.Logger
	Context                 = context.Background()
	Validator               = validator.New()
	BotUser                 *discordgo.User
	ObjectStorage           *objectstorage.ObjectStorage
	CurrentOperationMode    string // Current mode splashtail is operating in

	Config *config.Config

	// This is only non-nil for webserver
	MewldInstanceList *mproc.InstanceList

	// Debug stuff
	BuildInfo  *debug.BuildInfo
	ExtraDebug ExtraDebugInfo
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

	// Reuidis
	ruOptions, err := rueidis.ParseURL(Config.Meta.RedisURL.Parse())

	if err != nil {
		panic(err)
	}

	Rueidis, err = rueidis.NewClient(ruOptions)

	if err != nil {
		panic(err)
	}

	// Redis
	rOptions, err := redis.ParseURL(Config.Meta.RedisURL.Parse())

	if err != nil {
		panic(err)
	}

	Redis = redis.NewClient(rOptions)

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

	// Load dovewing state
	baseDovewingState := dovewing.BaseState{
		Pool:    Pool,
		Logger:  Logger,
		Context: Context,
		PlatformUserCache: hredis.RedisHotCache[dovetypes.PlatformUser]{
			Redis:  Redis,
			Prefix: "uobj__",
		},
		UserExpiryTime: 8 * time.Hour,
	}

	DovewingPlatformDiscord, err = dovewing.DiscordStateConfig{
		Session:        Discord,
		PreferredGuild: Config.Servers.Main,
		BaseState:      &baseDovewingState,
	}.New()

	if err != nil {
		panic(err)
	}

	ratelimit.SetupState(&ratelimit.RLState{
		HotCache: hredis.RedisHotCache[int]{
			Redis:  Redis,
			Prefix: "rl:",
		},
	})
}

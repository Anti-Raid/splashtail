package state

import (
	"context"
	"crypto/tls"
	"fmt"
	"net"
	"net/http"
	"os"
	"strings"
	"time"

	"go.api/state/redishotcache"
	"go.std/config"
	"go.std/objectstorage"
	"go.std/utils"
	"golang.org/x/net/http2"

	"github.com/bwmarrin/discordgo"
	mproc "github.com/cheesycod/mewld/proc"
	"github.com/go-playground/validator/v10"
	"github.com/go-playground/validator/v10/non-standard/validators"
	"github.com/infinitybotlist/eureka/dovewing"
	"github.com/infinitybotlist/eureka/dovewing/dovetypes"
	"github.com/infinitybotlist/eureka/genconfig"
	"github.com/infinitybotlist/eureka/jsonimpl"
	"github.com/infinitybotlist/eureka/proxy"
	"github.com/infinitybotlist/eureka/ratelimit"
	"github.com/infinitybotlist/eureka/snippets"
	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/redis/rueidis"
	"go.uber.org/zap"
	"gopkg.in/yaml.v3"
)

var (
	Pool                    *pgxpool.Pool
	Rueidis                 rueidis.Client // where perf is needed
	DovewingPlatformDiscord *dovewing.DiscordState
	Discord                 *discordgo.Session
	Logger                  *zap.Logger
	Context                 = context.Background()
	Validator               = validator.New()
	BotUser                 *discordgo.User
	ObjectStorage           *objectstorage.ObjectStorage
	CurrentOperationMode    string // Current mode splashtail is operating in
	Config                  *config.Config
	MewldInstanceList       *mproc.InstanceList

	IpcClient http.Client
)

func fetchMewldInstanceList() (*mproc.InstanceList, error) {
	var mc *mproc.InstanceList

	resp, err := http.Get(fmt.Sprintf("http://localhost:%d/getMewldInstanceList", Config.BasePorts.Bot.Parse()-1))

	if err != nil {
		return nil, err
	}

	defer resp.Body.Close()

	err = jsonimpl.UnmarshalReader(resp.Body, &mc)

	if err != nil {
		return nil, err
	}

	return mc, nil
}

func Setup() {
	utils.Must(
		Validator.RegisterValidation("notblank", validators.NotBlank),
		Validator.RegisterValidation("nospaces", snippets.ValidatorNoSpaces),
		Validator.RegisterValidation("https", snippets.ValidatorIsHttps),
		Validator.RegisterValidation("httporhttps", snippets.ValidatorIsHttpOrHttps),
	)

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

	for {
		mil, err := fetchMewldInstanceList()

		if err != nil {
			Logger.Error("Error fetching mewld instance list, waiting", zap.Error(err))
			time.Sleep(5 * time.Second)
			continue
		}

		MewldInstanceList = mil
		break
	}

	go func() {
		// Keep updating instance list every 5 seconds
		for {
			mil, err := fetchMewldInstanceList()

			if err != nil {
				Logger.Error("Error fetching mewld instance list, waiting", zap.Error(err))
			} else {
				MewldInstanceList = mil
				Logger.Debug("Updated mewld instance list")
			}

			time.Sleep(20 * time.Second)
		}
	}()

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

	// Object Storage
	ObjectStorage, err = objectstorage.New(&Config.ObjectStorage)

	if err != nil {
		panic(err)
	}

	// Discordgo
	Discord, err = discordgo.New("Bot " + Config.DiscordAuth.Token.Parse())

	if err != nil {
		panic(err)
	}

	Discord.Client.Transport = proxy.NewHostRewriter(strings.Replace(Config.Meta.Proxy.Parse(), "http://", "", 1), http.DefaultTransport, func(s string) {
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
		Logger.Info("[DISCORD] Launching with shard count", zap.Int("shard_count", Discord.ShardCount), zap.Int("shard_id", Discord.ShardID))
	})

	// Load dovewing state
	baseDovewingState := dovewing.BaseState{
		Pool:    Pool,
		Logger:  Logger,
		Context: Context,
		PlatformUserCache: redishotcache.RuedisHotCache[dovetypes.PlatformUser]{
			Redis:  Rueidis,
			Prefix: "uobj__",
			For:    "dovewing",
		},
		UserExpiryTime: 8 * time.Hour,
	}

	DovewingPlatformDiscord, err = dovewing.DiscordStateConfig{
		Session:        Discord,
		PreferredGuild: Config.Servers.Main.Parse(),
		BaseState:      &baseDovewingState,
	}.New()

	if err != nil {
		panic(err)
	}

	ratelimit.SetupState(&ratelimit.RLState{
		HotCache: redishotcache.RuedisHotCache[int]{
			Redis:    Rueidis,
			Prefix:   "rl:",
			For:      "ratelimit",
			Disabled: Config.Meta.WebDisableRatelimits,
		},
	})

	IpcClient.Timeout = 30 * time.Second
	IpcClient.Transport = &http2.Transport{
		AllowHTTP:      true,
		DialTLSContext: nil,
		DialTLS: func(network, addr string, cfg *tls.Config) (net.Conn, error) {
			return net.Dial(network, addr)
		},
	}
}

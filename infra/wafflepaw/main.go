package main

import (
	"context"
	"net/http"
	"os"
	"os/signal"
	"strings"
	"syscall"

	"github.com/anti-raid/splashtail/cmd/wafflepaw/bgtasks"
	"github.com/bwmarrin/discordgo"
	"github.com/go-playground/validator/v10"
	"github.com/infinitybotlist/eureka/proxy"
	"github.com/infinitybotlist/eureka/snippets"
	"github.com/redis/rueidis"
	"go.std/animusmagic"
	"go.std/config"
	"go.std/mewldresponder"
	"go.std/utils"
	"go.uber.org/zap"
	"gopkg.in/yaml.v3"
)

var (
	Context           context.Context
	ContextClose      context.CancelFunc
	Logger            *zap.Logger
	Config            *config.Config
	Rueidis           rueidis.Client
	Discord           *discordgo.Session
	AnimusMagicClient *animusmagic.AnimusMagicClient
	MewldResponder    *mewldresponder.MewldResponder
	MonitorWebhook    *utils.ParsedWebhookUrl
	v                 = validator.New()
)

func logPanic(msg string, err error) {
	// TODO: Add potential discord logging
	Logger.Panic(msg, zap.Error(err))
}

func main() {
	Context, ContextClose = context.WithCancel(context.Background())

	Logger = snippets.CreateZap()

	// Load monitors.yaml
	var monitors []AMProbeTask
	monitorFile, err := os.ReadFile("infra/wafflepaw/monitors.yaml")

	if err != nil {
		panic(err)
	}

	err = yaml.Unmarshal(monitorFile, &monitors)

	if err != nil {
		panic(err)
	}

	// Load config.yaml
	cfgFile, err := os.ReadFile("config.yaml")

	if err != nil {
		panic(err)
	}

	err = yaml.Unmarshal(cfgFile, &Config)

	if err != nil {
		panic(err)
	}

	err = v.Struct(Config)

	if err != nil {
		panic("configError: " + err.Error())
	}

	// Parse webhook
	MonitorWebhook, err = utils.ParseWebhookURL(Config.Wafflepaw.StatusWebhook)

	if err != nil {
		logPanic("error parsing webhook url", err)
	}

	// Reuidis
	ruOptions, err := rueidis.ParseURL(Config.Meta.BotRedisURL)

	if err != nil {
		logPanic("error parsing redis url", err)
	}

	Rueidis, err = rueidis.NewClient(ruOptions)

	if err != nil {
		logPanic("error creating redis client", err)
	}

	// Discord
	Discord, err = discordgo.New("Bot " + Config.DiscordAuth.Token)

	if err != nil {
		logPanic("error creating discord session", err)
	}

	Discord.Client.Transport = proxy.NewHostRewriter(strings.Replace(Config.Meta.Proxy.Parse(), "http://", "", 1), http.DefaultTransport, func(s string) {
		Logger.Info("[PROXY]", zap.String("note", s))
	})

	err = StartMonitors(monitors)

	if err != nil {
		logPanic("error starting monitors", err)
	}

	go bgtasks.StartAllTasks(Logger)

	// Wait for signals
	syscalls := []os.Signal{
		os.Interrupt,
		syscall.SIGTERM,
		syscall.SIGILL,
	}

	sig := make(chan os.Signal, 1)

	signal.Notify(sig, syscalls...)

	<-sig

	Logger.Info("Shutting down animus magic client")

	ContextClose()

	// TODO: Send log message
}

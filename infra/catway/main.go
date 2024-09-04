package main

import (
	"catway/core"
	"log/slog"
	"os"
	"os/signal"
	"syscall"

	"gopkg.in/yaml.v3"
)

func main() {
	// Open catway.yaml
	f, err := os.Open("catway.yaml")

	if err != nil {
		panic(err)
	}

	defer f.Close()

	// Parse catway.yaml
	var cfg core.CatwayConfig

	err = yaml.NewDecoder(f).Decode(&cfg)

	if err != nil {
		panic(err)
	}

	catway, err := core.NewCatway(&cfg)

	if err != nil {
		panic(err)
	}

	slog.Info("Catway started", slog.Any("appID", catway.Discord.ApplicationID()))

	signalCh := make(chan os.Signal, 1)
	signal.Notify(signalCh, syscall.SIGINT, syscall.SIGTERM, os.Interrupt)
	<-signalCh

	err = catway.Close()
	if err != nil {
		catway.Logger.Error("Exception whilst closing catway", slog.String("err", err.Error()))
	}

	slog.Info("Catway closed due to signal")
}

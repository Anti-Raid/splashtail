package main

import (
	"errors"
	"fmt"
	"os"

	"github.com/anti-raid/splashtail/cmd/localjobs/easyconfig"
	"github.com/anti-raid/splashtail/cmd/localjobs/ljstate"
	"github.com/anti-raid/splashtail/state"
	"github.com/bwmarrin/discordgo"
	"gopkg.in/yaml.v3"
)

func main() {
	f, err := os.Open("localjobs-config.yaml")

	if errors.Is(err, os.ErrNotExist) {
		// No config, trigger EasyConfig
		c, err := easyconfig.EasyConfig()

		if err != nil {
			fmt.Println("ERROR: Failed to create config:", err.Error())
			os.Exit(1)
		}

		f, err = os.Create("localjobs-config.yaml")

		if err != nil {
			fmt.Println("ERROR: Failed to create config:", err.Error())
			os.Exit(1)
		}

		err = yaml.NewEncoder(f).Encode(c)

		if err != nil {
			fmt.Println("ERROR: Failed to encode config:", err.Error())
			os.Exit(1)
		}

		ljstate.Config = c

		err = f.Close()

		if err != nil {
			fmt.Println("ERROR: Failed to close config:", err.Error())
		}
	} else {
		// Config exists, load it
		err = yaml.NewDecoder(f).Decode(&ljstate.Config)

		if err != nil {
			fmt.Println("ERROR: Failed to decode config:", err.Error())
			os.Exit(1)
		}

		err = f.Close()

		if err != nil {
			fmt.Println("ERROR: Failed to close config:", err.Error())
		}
	}

	discordSess, err := discordgo.New("Bot " + ljstate.Config.BotToken)

	if err != nil {
		fmt.Println("ERROR: Failed to create Discord session:", err.Error())
		os.Exit(1)
	}

	state.Discord = discordSess
}

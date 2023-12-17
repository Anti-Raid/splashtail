package main

import (
	"embed"
	"errors"
	"fmt"
	"io"
	"os"

	"github.com/anti-raid/splashtail/cmd/localjobs/easyconfig"
	"github.com/anti-raid/splashtail/cmd/localjobs/ljstate"
	"github.com/anti-raid/splashtail/state"
	"github.com/bwmarrin/discordgo"
	"github.com/infinitybotlist/eureka/snippets"
	"gopkg.in/yaml.v3"
)

var prefixDir = "ljconfig"

//go:embed all:presets
var allPresets embed.FS

func main() {
	state.Logger = snippets.CreateZap()

	err := os.MkdirAll(prefixDir, 0755)

	if err != nil {
		fmt.Println("ERROR: Failed to create localjobs directory:", err.Error())
		os.Exit(1)
	}

	f, err := os.Open(prefixDir + "/localjobs-config.yaml")

	if errors.Is(err, os.ErrNotExist) {
		// No config, trigger EasyConfig
		c, err := easyconfig.EasyConfig()

		if err != nil {
			fmt.Println("ERROR: Failed to create config:", err.Error())
			os.Exit(1)
		}

		f, err = os.Create(prefixDir + "/localjobs-config.yaml")

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

	// Unravel presets to presets directory if not found
	s, err := os.Stat(prefixDir + "/presets")

	if err == nil && !s.IsDir() {
		err = os.Remove(prefixDir + "/presets")

		if err != nil {
			fmt.Println("ERROR: Failed to remove presets file:", err.Error())
			os.Exit(1)
		}
	}

	if errors.Is(err, os.ErrNotExist) {
		err = os.Mkdir(prefixDir+"/presets", 0755)

		if err != nil {
			fmt.Println("ERROR: Failed to create presets directory:", err.Error())
			os.Exit(1)
		}
	}

	presets, err := allPresets.ReadDir("presets")

	if err != nil {
		fmt.Println("ERROR: Failed to read presets:", err.Error())
		os.Exit(1)
	}

	for _, preset := range presets {
		if preset.IsDir() {
			continue
		}

		// Stat localjobs/presets/preset.Name()
		s, err := os.Stat(prefixDir + "/presets/" + preset.Name())

		var isNotExist bool
		if err == nil && !s.IsDir() {
			err = os.Remove(prefixDir + "/presets/" + preset.Name())

			if err != nil {
				fmt.Println("ERROR: Failed to remove preset:", err.Error())
				os.Exit(1)
			}

			isNotExist = true
		} else if errors.Is(err, os.ErrNotExist) {
			isNotExist = true
		} else if err != nil {
			fmt.Println("ERROR: Failed to stat preset:", err.Error())
			os.Exit(1)
		}

		if !isNotExist {
			continue
		}

		f, err := allPresets.Open("presets/" + preset.Name())

		if err != nil {
			fmt.Println("ERROR: Failed to open preset:", err.Error())
			os.Exit(1)
		}

		presetFile, err := os.Create(prefixDir + "/presets/" + preset.Name())

		if err != nil {
			fmt.Println("ERROR: Failed to create preset:", err.Error())
			os.Exit(1)
		}

		size, err := io.Copy(presetFile, f)

		if err != nil {
			fmt.Println("ERROR: Failed to write preset:", err.Error())
			os.Exit(1)
		}

		fmt.Printf("INFO: Wrote preset %s (%d bytes)\n", preset.Name(), size)

		err = f.Close()

		if err != nil {
			fmt.Println("ERROR: Failed to close preset:", err.Error())
		}

		err = presetFile.Close()

		if err != nil {
			fmt.Println("ERROR: Failed to close preset:", err.Error())
		}
	}

	discordSess, err := discordgo.New("Bot " + ljstate.Config.BotToken)

	if err != nil {
		fmt.Println("ERROR: Failed to create Discord session:", err.Error())
		os.Exit(1)
	}

	state.Discord = discordSess
}

package easyconfig

import (
	"fmt"

	"github.com/anti-raid/splashtail/cmd/localjobs/lib"
	"github.com/anti-raid/splashtail/cmd/localjobs/types"
	"github.com/bwmarrin/discordgo"
	"github.com/fatih/color"
)

var bold = color.New(color.Bold).PrintlnFunc()
var debug = color.New(color.FgWhite, color.Faint).PrintfFunc()
var red = color.New(color.FgRed).PrintfFunc()

func EasyConfig() (*types.Config, error) {
	bold("Welcome to EasyConfig!")
	err := lib.OpenDefault("https://discord.com/developers/applications?new_application=true")

	if err != nil {
		debug("Failed to open browser: %s, please manually open https://discord.com/developers/applications?new_application=true using your favorite web browser!\n", err.Error())
	}

	fmt.Println("In order for Anti-Raid LocalJobs to work, you need to provide a bot token.")
	fmt.Println("\n\nTo get a bot token, follow the following steps:")
	fmt.Println("1. Navigate to https://discord.com/developers/applications?new_application=true using your favorite web browser and click 'New Application'.")
	fmt.Println("2. Give your application a name such as 'antiraid-ljh', agree to the Terms And Conditions, then click 'Create'.")
	fmt.Println("3. Click on 'Bot' on the left side of the screen, then click 'Reset Token' to create a new token for the bot!")

	for {
		token := lib.UserInput("Please enter your bot token")

		if token == "" {
			red("You must provide a bot token!")
			continue
		}

		sess, err := discordgo.New("Bot " + token)

		if err != nil {
			red("Failed to create Discord session: %s, please recheck the token and try again", err.Error())
			continue
		}

		bot, err := sess.User("@me")

		if err != nil {
			red("Failed to get bot user: %s, please recheck the token and try again", err.Error())
			continue
		}

		// Set the intents flags manually, we can't trust the user
		currApp, err := sess.Application("@me")

		if err != nil {
			red("Failed to get application: %s, please recheck the token and try again", err.Error())
			continue
		}

		if bot.Flags&int(discordgo.UserFlagVerifiedBot) != int(discordgo.UserFlagVerifiedBot) {
			// Set flags correctly
			var flags = (1 << 15) | (1 << 19) // GATEWAY_GUILD_MEMBERS_LIMITED and GATEWAY_MESSAGE_CONTENT_LIMITED

			_, err = sess.Request("PATCH", discordgo.EndpointApplication("@me"), map[string]any{
				"flags": flags,
			})

			if err != nil {
				red("Failed to set bot flags: %s, please recheck the token and try again", err.Error())
				continue
			}
		}

		inviteUrl := fmt.Sprintf("https://discord.com/oauth2/authorize?client_id=%s&scope=bot&permissions=8", currApp.ID)
		bold("Invite this bot to your server using the following URL: %s", inviteUrl)

		err = lib.OpenDefault(inviteUrl)

		if err != nil {
			debug("Failed to open browser: %s, please manually open %s using your favorite web browser!\n", err.Error(), inviteUrl)
		}

		return &types.Config{
			BotToken: token,
			Secrets: map[string]string{
				"BackupPassword": "",
			},
		}, nil
	}
}

# Permissions

Imagine. Imagine a discord bot that you could completely control. You could decide who can use any specific command, who can change the bot's settings, and who can even use the bot at all. 

*Thats AntiRaid...*

AntiRaid has a customizable permission system that uses both Discord permissions for simplicity and [kittycat](https://github.com/InfinityBotList/kittycat) permissions for more specific requirements. While AntiRaid does provide some base-line defaults, Lua scripting can be used to augment these defaults and extend Antiraid with arbitrarily complex permission systems, among other things. See the [templating guide](../templating-lua/1-intro.md) for more information on how to use Lua templates. Then, just code away!

## TIP

For best results, consider limiting the server permissions of other users to the minimum required. Then, use AntiRaid for actual moderation. That's better than giving everyone admin permissions and then trying to restrict them with AntiRaid.
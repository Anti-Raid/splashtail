# Permissions

Imagine. Imagine a discord bot which you could completely control. You could decide who can use any specific command, who can change the bot's settings, and who can even use the bot at all. 

*Thats AntiRaid...*

AntiRaid has a customizable permission system utilizing both Discord native permissions for ease of use and [kittycat](https://github.com/InfinityBotList/kittycat) for more complex use cases. And for the really unique cases, AntiRaid provides support for Lua script templating which case be used to extend AntiRaid into more complex/unique permission systems itself.

The idea is simple: All roles have permissions attached to them and members can have special permission overrides on top of that. The permissions are then checked when a command is run.

**Note:** The documentation for this is not yet finished and is a WIP.

## Modes

Anti-Raid has two different modes for permission checks depending on how custom your needs are:

- ``Simple``: In simple mode, you just need to specify the exact permissions needed to run a command. This is the default mode.
- ``Template``: If you have more advanced needs, you can also use custom templates to determine if a user has the required permissions. See [`Templating`](../templating-lua/1-intro.md) for more information on how templating works.

## Simple Permission Checks

Since not everyone knows how to code, AntiRaid provides a simple permission checking system builtin that should be enough for most. Heres how it works:

1. Commands are the base primitive of AntiRaid. These commands can be either real or virtual. Real commands are commands that you can actually run (as well as configure) while virtual commands are placeholders for permissions, help commands or external modules that don't use the normal AntiRaid module system [e.g. modules written in Gleam].
2. Commands can be configured through either permissions or by simply disabling them (some commands cannot be disabled however to ensure you can't break the bot permanently).
3. Server admins can set permissions on their server roles and then override them for specific users through permission overrides. 
4. Server admins can then set permissions on commands and default permissions on modules. These permissions are then checked when a command is run.

Of course, the above is just an overview of AntiRaid permission system. This is just an overview, of course.

## Template Permission Checks

For more advanced users, AntiRaid provides a template system that allows you to create custom permission checks. This is done through the use of our custom Luau templating system. 

See the [templating guide](../templating-lua/1-intro.md) for more information on how to use Lua templates. Then, just code away!

## TIP

For best results, consider limiting server permissions of other users to the minimum required. Then, use AntiRaid for actual moderation. That's better than giving everyone admin permissions and then trying to restrict them with AntiRaid.

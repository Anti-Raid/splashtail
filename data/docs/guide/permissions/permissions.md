# Permissions

AntiRaid has a customizable permission system utilizing both Discord native permissions for ease of use and [kittycat](https://github.com/InfinityBotList/kittycat) for more complex use cases. And for the really unique cases, AntiRaid provides support for Lua script templating which case be used to extend AntiRaid into more complex/unique permission systems itself.

The idea is simple: All roles have permissions attached to them and members can have special permission overrides on top of that. The permissions are then checked when a command is run.

## TIP

For best results, consider limiting server permissions of other users to the minimum required. Then, use AntiRaid for actual moderation.

## Simple Permission Checks

Since not everyone knows how to code, AntiRaid provides a simple permission checking system builtin that should be enough for most.
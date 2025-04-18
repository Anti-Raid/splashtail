# Lua Templating

At AntiRaid, we prioritize flexibility and customization for our users. To this end, our bot supports advanced templating to allow for extensive personalization of embeds and messages. While many bots utilize proprietary languages or templating engines, we have chosen to leverage Lua—a renowned scripting language widely used in game development and other applications. This decision ensures that our users benefit from a powerful, well-documented, and versatile language, enhancing the capability and ease of customizing their AntiRaid experience. 

Specifically, Anti Raid uses a variant of Lua called Luau. If you've ever used Roblox before, this is the same variant of Lua used there too (which is why Luau is also known as Roblox Lua in many places). You can check out the [Luau docs](https://luau-lang.org/) for more information on the language itself. Unlike PUC Lua (the reference implementation), Luau is both faster and offers robust sandboxing capabilities allowing AntiRaid to run scripts in as safe an environment as possible.

## Getting Started

Note that the remainder of these docs will cover AntiRaids Lua SDKs. To learn more about Lua itself, please checkout Lua's official tutorial for Lua 5.0 [here](https://www.lua.org/pil/1.html). Other resources for Lua exist (Lua is very popular after all), including [Roblox's tutorial](https://devforum.roblox.com/t/lua-scripting-starter-guide/394618#print-5) (ignore the Studio bits), [TutorialPoint](https://www.tutorialspoint.com/lua/lua_quick_guide.htm) and [Codecademy](https://www.codecademy.com/learn/learn-lua).

## Limitations

AntiRaid applies the following 3 global limits to all Lua templates. Note that we may provide increased limits as a Premium feature in the future:

```rust
pub const MAX_TEMPLATE_MEMORY_USAGE: usize = 1024 * 1024 * 3; // 3MB maximum memory
pub const MAX_TEMPLATES_EXECUTION_TIME: std::time::Duration = std::time::Duration::from_secs(30); // 30 seconds maximum execution time
```

The above limits are in place to prevent abuse and ensure that the bot remains responsive. If you require increased limits, please contact support (once again, this may change in the future).

## Some key notes

- Each guild is assigned a dedicated Lua(u) VM. This VM is used to execute Lua code that is used in the templates.
- The total memory usage that a guild can use is limited to ``MAX_TEMPLATE_MEMORY_USAGE`` (currently 3MB). This is to prevent a single guild from using too much memory.
- Execution of all scripts is timed out when the last executed script takes longer than ``MAX_TEMPLATES_EXECUTION_TIME`` (currently 30 seconds).
- A guilds Lua(u) VM will persist until marked as broken (either by explicitly requesting it or by exceeding memory limits)
- While a guilds Lua(u) VM will internally have a read-only shared global table, AntiRaid provides a special global table that is isolated at the template level with a custom ``__index`` and ``__metatable`` set to proxy writes (similar to Roblox's setup). This means that builtins will remain read-only, but new values can be added.
- The standard ``require`` statement can be used to either import AntiRaid plugins or to import other Luau assets that belong to the template. AntiRaid internally uses standard Luau require-by-string semantics with support for ``init.luau`` files for requires.
- Note that plugins are read-only/sandboxed and cannot be monkey-patched etc.
- All templates are executed as Luau threads.

In general, all AntiRaid templates should start with the following:

```lua
local evt, token = ...
-- Do something
return output -- Optionally return something here
```

## Interop

Many features of Lua don't work so well when calling functions within the AntiRaid SDK. For example, both arrays and maps are expressed as tables in Lua. However, AntiRaid, being written in Rust, doesn't know this and hance needs some help to convert certain types for FFI. This is where the `@antiraid/interop` module comes in.

### Arrays

To pass arrays to modules within the AntiRaid SDK, you need to set the metatable to ``@antiraid/interop#array_metatable``. This will allow the SDK to convert the array to a Rust ``Vec`` internally.

```lua
local interop = require '@antiraid/interop'
setmetatable({a = 5}, interop.array_metatable)
```

### Null

In some instances, interactions with AntiRaid may yield a special ``null`` lightuserdata value. This lightuserdata value is exposed under ``@antiraid/interop#null`` and can also be used template-side as desired:

```lua
local interop = require '@antiraid/interop'
local null = interop.null -- This is the null value
```

### Memory Usage

While not strictly useful for interop, it is often desirable to know the memory usage of a Lua template as AntiRaid will kill your template if it exceeds the memory limit. For this, you can use the `@antiraid/interop#memusage` function.

```lua
local interop = require '@antiraid/interop'
print(interop.memusage())
```

## Events

All Lua templates are invoked via events. As such, the first argument to the template is an ``Event``. ``Event`` is a ``userdata``. The below will explain the most important fields exposed by ``Event``. Note that all fields, unless stated otherwise, are read-only and cached.

## Template Context

All Lua templates are passed both the ``Event`` (denoted by `args`) and a `TemplateContext` userdata (denoted by `token`). Note that like ``Event``, ``TemplateContext`` is a *userdata* (not a table). As such, they cannot be manually constructed in templates themselves.

"Executors" and other sensistive APIs use the `TemplateContext` to read internal data. Examples of executors include the ``@antiraid/discord`` `DiscordActionExecutor`, which allows you to perform actions such as banning/kicking/timing out users and other Discord actions and ``@antiraid/kv`` `KvExecutor` which allow for persistent storage via a key-value interface. 

``TemplateContext`` is guaranteed to be valid while accessible in the VM . This means that templates can choose to share their capabilities with other templates using ``_G``.

Likewise, it is also guaranteed that the created executor is complete and does not rely on the token itself whatsoever after creation. This means that a template executor can be used after the template has finished executing (e.g. in a coroutine).

### Example

```lua
local args, token = ...
print(token)
```
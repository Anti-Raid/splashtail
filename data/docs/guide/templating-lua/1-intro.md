# Lua Templating

At AntiRaid, we prioritize flexibility and customization for our users. To this end, our bot supports advanced templating to allow for extensive personalization of embeds and messages. While many bots utilize proprietary languages or templating engines, we have chosen to leverage Lua—a renowned scripting language widely used in game development and other applications. This decision ensures that our users benefit from a powerful, well-documented, and versatile language, enhancing the capability and ease of customizing their AntiRaid experience. 

Specifically, Anti Raid uses a variant of Lua called Luau. If you've ever used Roblox before, this is the same variant of Lua used there too (which is why Luau is also known as Roblox Lua in many places). You can check out the [Luau docs](https://luau-lang.org/) for more information on the language itself. Unlike PUC Lua (the reference implementation), Luau is both faster and offers robust sandboxing capabilities allowing AntiRaid to run scripts in as safe an environment as possible.

## Getting Started

Note that the remainder of these docs will cover AntiRaids Lua SDKs. To learn more about Lua itself, please checkout Lua's official tutorial for Lua 5.0 [here](https://www.lua.org/pil/1.html). Other resources for Lua exist (Lua is very popular after all), including [Roblox's tutorial](https://devforum.roblox.com/t/lua-scripting-starter-guide/394618#print-5) (ignore the Studio bits), [TutorialPoint](https://www.tutorialspoint.com/lua/lua_quick_guide.htm) and [Codecademy](https://www.codecademy.com/learn/learn-lua).

## Limitations

AntiRaid applies the following 3 global limits to all Lua templates. Note that we may provide increased limits as a Premium feature in the future:

```rust
pub const MAX_TEMPLATE_MEMORY_USAGE: usize = 1024 * 1024 * 3; // 3MB maximum memory
pub const MAX_TEMPLATE_LIFETIME: std::time::Duration = std::time::Duration::from_secs(60 * 10); // 10 minutes maximum lifetime
pub const MAX_TEMPLATES_EXECUTION_TIME: std::time::Duration = std::time::Duration::from_secs(5); // 5 seconds maximum execution time
```

The above limits are in place to prevent abuse and ensure that the bot remains responsive. If you require increased limits, please contact support (once again, this may change in the future).

## Some key notes

- Each guild is assigned a dedicated Lua VM. This VM is used to execute Lua code that is used in the templates.
- The total memory usage that a guild can use is limited to ``MAX_TEMPLATE_MEMORY_USAGE`` (currently 3MB). This is to prevent a single guild from using too much memory.
- Execution of all scripts is timed out when the last executed script takes longer than ``MAX_TEMPLATES_EXECUTION_TIME`` (currently 5 seconds).
- A lua VM will exist for a total of ``MAX_TEMPLATE_LIFETIME`` (currently 10 minutes) after the last access before being destroyed. This is to reduce memory+CPU usage.
- The ``__stack`` table can be used to share data across templates safely *while the VM is running*. without affecting other templates. This is useful for sharing data between templates such as Audit Logs. **Note that AntiRaid uses luau sandboxing meaning that `_G` is readonly.**
- The entrypoint of any Lua template is ``function(args)``. All code must be inside this function.
- The standard ``require`` statement can be used to import AntiRaid modules. **Note that the modules are read-only** and cannot be monkey-patched etc.
- **Because Lua is a single-threaded language, only one template can be executed at a time**
- Naked returns are not allowed within a function (outside the last return) are not allowed. E.g, the below may not compile:

```lua
function (args)
    print("Hello")
    return ABC
    print("World")
    return DEF
end
```

However, the following will:

```lua
function (args)
    if 1 == 1 then
        print("Hello")
        return ABC
        print("World")
        return DEF
    end
end
```

If you need this behaviour, consider using [luau-bundler](https://www.github.com/Anti-Raid/luau-bundler).

## Interop

Many features of Lua don't work so well when calling functions within the AntiRaid SDK. For example, both arrays and maps are expressed as tables in Lua. However, AntiRaid, being written in Rust, doesn't know this and hance needs some help to convert certain types for FFI. This is where the `@antiraid/interop` module comes in.

### Arrays

To pass arrays to modules within the AntiRaid SDK, you need to set the metatable to ``@antiraid/interop#array_metatable``. This will allow the SDK to convert the array to a Rust ``Vec`` internally.

```lua
local interop = require '@antiraid/interop'
setmetatable({a = 5}, interop.array_metatable)
```

### Null

While the Lua ``nil`` does work in many cases (and even when calling the SDK), its not the best choice. When querying AntiRaid SDK, the SDK will use the ``@antiraid/interop#null`` value to represent a null value. Your Lua templates can also use this value if desired

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

### User Error vs Runtime Error

As Lua does not have a built-in way to distinguish between user errors and runtime errors, AntiRaid provides a way to do so. Simply return a table with the key ``__error`` set, and the value set to the error message to create a user error. You can use the standard ``error`` function for runtime errors. E.g.


```lua
-- User Error
return { __error = "You have reached the maximum number of tries in this 5 minute window." }

-- Runtime Error
error("Could not parse user ID for some reason")
```

## Template Tokens

All Lua templates include a special template token in addition to the template arguments. Modules requiring more privileged levels of access will require this token as an argument.

## Example

```lua
function(args, token)
    print(token)
end
```
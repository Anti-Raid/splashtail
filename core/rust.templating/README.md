# Anti-Raid Templating System

## Supported Languages

- Lua (luau / Roblox Lua) - Tier 1
- Tera* - Tier 2 (poor sandboxing capabilities)
- Rhai* - Tier 2 (sandboxing capabilities rely on thread-unsafe methods and pointer arithmetic. Not suitable for the async environment Anti-Raid needs)

Lua is the recommended language for templating. Tera and Rhai are supported but are not recommended due to their poor sandboxing capabilities and will be removed/disabled in production builds

## Lua notes

- Each guild is assigned a Lua VM. This VM is used to execute Lua code that is used in the templates.
- The total memory usage that a guild can use is limited to ``MAX_TEMPLATE_MEMORY_USAGE`` (currently 3MB). This is to prevent a single guild from using too much memory.
- Execution of all scripts is timed out when the last executed script takes longer than ``MAX_TEMPLATES_EXECUTION_TIME`` (currently 5 seconds).
- A lua VM will exist for a total of ``MAX_TEMPLATE_LIFETIME`` (currently 5 minutes) after the last access before being destroyed. This is to reduce memory+CPU usage.
- The ``__stack`` table can be used to share data across templates safely *while the VM is running*. without affecting other templates. This is useful for sharing data between templates such as Audit Logs. **Note that Anti-Raid uses luau sandboxing meaning that `_G` is readonly.**
- All Anti-Raid specific modules can be found in the ``__ar_modules`` table. This includes the ``messages`` and ``permissions`` modules.
- The entrypoint of any Lua template is ``function(args)``. 
- **Currently, only one template may run at any given time on a specific guild. Other executions will block/queue until the current execution is complete. Anti-Raid is looking into using Workers and/or WebAssembly to solve this**

## WIP/Potential Languages

- JavaScript (see ``lang_javascript_quickjs`` and ``lang_javascript_v8`` for the current load experiments + integration experiments), potential but unlikely unless someone finds a solution
- WebAssembly (potential, not yet decided)

## Language Requirements

1. All languages must export the following modules/helpers to the extent required as per the templating documentation. (TODO: Improve this spec)

- Messages
- Permissions

2. All languages must provide a way to sandbox the execution of the code. This is a security requirement. In particular, timeouts and heap/stack/memory limits are required.
3. Callers must use the abstracted out function calls from ``lib.rs``. Language support is auto-determined based on the first line of the file, which must be: ``"//lang:lang_XXX"`` where ``XXX`` is the language name.

## My language vent

**For reference on my discord vents: https://discord.com/channels/763812938875535361/1040734156327501905/1267195190100361440**

Why is lua the only sane language for embedding
V8 has big ffi problems with rust. If you try spawning too many isolates, you have a 100% chance of aborting your process and this issue can only be resolved by performing unsafe void* pointer casts 
Quickjs is a bit too slow and poorly documented 
Rhai is good but it’s a custom language and it’s sandboxing abilities need unsafe code to fully work (and said unsafe code involves pointer arithmetic that is not thread safe) and heap memory limits require you to manually calculate the heap usage 
Tera has virtually no safety features and will gladly execute an infinite recursion
For starlark/skylark, go to the point on rhai but hopefully without the unsafe bits
I can understand now why the game modding industry uses lua, it’s basically the only sane language for handling user input
Lua is legit the only sane scripting language on this entire list

[rhai is not only slower than lua, its sandboxing (i said it above here too in a vent i think) requires actual pointer arithmetic that isnt thread safe, its also a custom lang no one knows while lua is well known in the game community. Luau is used in Roblox games so it caters to Discords target market as well]
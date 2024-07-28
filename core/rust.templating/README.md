# Anti-Raid Templating System

## Supported Languages

- Tera*
- Rhai*

`*` - These languages do not have sufficient sandboxing yet. Solving this is required before releasing Anti-Raid in production. In addition, there are talks of replacing the entire templating system with just WebAssembly.

## WIP Languages

- JavaScript (see ``lang_javascript_quickjs`` and ``lang_javascript_v8`` for the current load experiments + integration experiments)
- WebAssembly (potential, not yet decided)

# Developer Documentation

1. All languages must export the following modules/helpers to the extent required as per the templating documentation. (TODO: Improve this spec)

- Messages
- Permissions

2. All languages must provide a way to sandbox the execution of the code. This is a security requirement. In particular, timeouts and heap/stack/memory limits are required.
3. Callers must use the abstracted out function calls from ``lib.rs``. Language support is auto-determined based on the first line of the file, which must be: ``"//lang:lang_XXX"`` where ``XXX`` is the language name.
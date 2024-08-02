pub mod message;

use mlua::prelude::*;
use once_cell::sync::Lazy;

static PLUGINS: Lazy<indexmap::IndexMap<String, ModuleFn>> = Lazy::new(|| {
    indexmap::indexmap! {
        "@antiraid/builtins".to_string() => builtins as ModuleFn,
        "@antiraid/message".to_string() => message::init_plugin as ModuleFn,
    }
});

type ModuleFn = fn(&Lua) -> LuaResult<LuaTable>;

/// Provides the lua builtins as a seperate table
pub fn builtins(lua: &Lua) -> LuaResult<LuaTable> {
    let module = lua.create_table()?;
    module.set("require", lua.create_function(require)?)?;
    module.set(
        "memusage",
        lua.create_function(|lua, _: ()| Ok(lua.used_memory()))?,
    )?;

    module.set_readonly(true); // Block any attempt to modify this table

    Ok(module)
}

pub fn require(lua: &Lua, (plugin_name,): (String,)) -> LuaResult<LuaTable> {
    match PLUGINS.get(plugin_name.as_str()) {
        Some(plugin) => plugin(lua),
        None => {
            // These core modules are provided directly
            let is_module = matches!(
                plugin_name.as_str(),
                "math"
                    | "table"
                    | "string"
                    | "coroutine"
                    | "bit32"
                    | "utf8"
                    | "os"
                    | "debug"
                    | "buffer"
            );

            if is_module {
                return lua.globals().get::<_, LuaTable>(plugin_name);
            }

            // Import the plugin from lua stdlib
            let require = lua.named_registry_value::<LuaFunction>("_lua_require")?;
            require.call::<_, LuaTable>(plugin_name)
        }
    }
}

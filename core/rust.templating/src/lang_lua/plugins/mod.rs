pub mod actions;
pub mod interop;
pub mod kv;
pub mod lune;
pub mod message;
pub mod permissions;

use mlua::prelude::*;
use std::sync::LazyLock;

// Modules can load their own plugins
pub static PLUGINS: LazyLock<indexmap::IndexMap<String, ModuleFn>> = LazyLock::new(|| {
    indexmap::indexmap! {
        "@antiraid/actions".to_string() => actions::init_plugin as ModuleFn,
        "@antiraid/builtins".to_string() => builtins as ModuleFn,
        "@antiraid/interop".to_string() => interop::init_plugin as ModuleFn,
        "@antiraid/kv".to_string() => kv::init_plugin as ModuleFn,
        "@antiraid/message".to_string() => message::init_plugin as ModuleFn,
        "@antiraid/permissions".to_string() => permissions::init_plugin as ModuleFn,
        "@lune/datetime".to_string() => lune::datetime::init_plugin as ModuleFn,
        "@lune/regex".to_string() => lune::regex::init_plugin as ModuleFn,
        "@lune/serde".to_string() => lune::serde::init_plugin as ModuleFn,
    }
});

type ModuleFn = fn(&Lua) -> LuaResult<LuaTable>;

/// Provides the lua builtins as a seperate table
pub fn builtins(lua: &Lua) -> LuaResult<LuaTable> {
    let module = lua.create_table()?;
    module.set("require", lua.create_function(require)?)?;
    module.set_readonly(true); // Block any attempt to modify this table
    Ok(module)
}

pub fn require(lua: &Lua, (plugin_name,): (String,)) -> LuaResult<LuaTable> {
    match PLUGINS.get(plugin_name.as_str()) {
        Some(plugin) => plugin(lua),
        None => {
            if let Ok(table) = lua.globals().get::<LuaTable>(plugin_name.clone()) {
                return Ok(table);
            }

            Err(LuaError::runtime(format!(
                "module '{}' not found",
                plugin_name
            )))
        }
    }
}

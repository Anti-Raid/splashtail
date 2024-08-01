pub mod message;

use mlua::prelude::*;
use once_cell::sync::Lazy;

static PLUGINS: Lazy<indexmap::IndexMap<String, ModuleFn>> = Lazy::new(|| {
    indexmap::indexmap! {
        "builtins".to_string() => builtins as ModuleFn,
        "message".to_string() => message::init_plugin as ModuleFn,
    }
});

type ModuleFn = fn(&Lua) -> LuaResult<LuaTable>;

/// Provides the plugin core, that is:
/// - __ar_modules.load_plugin("plugin_name")
pub fn builtins(lua: &Lua) -> LuaResult<LuaTable> {
    let module = lua.create_table()?;
    module.set(
        "load_plugin",
        lua.create_function(|lua, (plugin_name,): (String,)| {
            let plugin = PLUGINS.get(plugin_name.as_str()).ok_or_else(|| {
                mlua::Error::external(format!("Plugin {} not found", plugin_name))
            })?;

            plugin(lua)
        })?,
    )?;

    module.set_readonly(true); // Block any attempt to modify this table

    Ok(module)
}

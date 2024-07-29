pub mod message;

use mlua::prelude::*;

type ModuleFn = fn(&Lua) -> LuaResult<LuaTable>;

pub fn lua_plugins() -> Vec<(&'static str, ModuleFn)> {
    vec![("message", message::init_plugin)]
}

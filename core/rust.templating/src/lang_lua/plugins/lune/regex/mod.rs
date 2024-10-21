mod captures;
mod matches;
#[allow(clippy::module_inception)]
mod regex;

use crate::lang_lua::plugins::lune::{regex::regex::LuaRegex, utils::TableBuilder};
use mlua::prelude::*;

/**
    Creates the `regex` standard library module.

    # Errors

    Errors when out of memory.
*/
pub fn init_plugin(lua: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::new(lua)?
        .with_function("new", new_regex)?
        .build_readonly()
}

fn new_regex(_: &Lua, pattern: String) -> LuaResult<LuaRegex> {
    LuaRegex::new(pattern)
}

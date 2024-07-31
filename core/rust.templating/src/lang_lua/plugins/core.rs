use mlua::{IntoLua, IntoLuaMulti, Lua, LuaSerdeExt, Result as LuaResult};

pub struct ArLuaResult<T>(pub Result<T, String>);

impl<T: IntoLuaMulti> IntoLuaMulti for ArLuaResult<T> {
    fn into_lua_multi(self, lua: &Lua) -> LuaResult<mlua::MultiValue> {
        match self.0 {
            Ok(v) => v.into_lua_multi(lua),
            Err(s) => Ok(mlua::MultiValue::from_iter([lua.null(), s.into_lua(lua)?])),
        }
    }
}

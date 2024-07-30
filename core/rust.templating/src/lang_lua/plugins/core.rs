use mlua::{
    Error as LuaError, FromLua, IntoLua, IntoLuaMulti, Lua, LuaSerdeExt, Result as LuaResult,
};

pub struct ArLuaResult<T>(pub Result<T, String>);

impl<'lua, T: IntoLuaMulti<'lua>> IntoLuaMulti<'lua> for ArLuaResult<T> {
    fn into_lua_multi(self, lua: &'lua Lua) -> LuaResult<mlua::MultiValue<'lua>> {
        match self.0 {
            Ok(v) => v.into_lua_multi(lua),
            Err(s) => Ok(mlua::MultiValue::from_iter([lua.null(), s.into_lua(lua)?])),
        }
    }
}

impl<'lua, T: FromLua<'lua>> mlua::FromLuaMulti<'lua> for ArLuaResult<T> {
    fn from_lua_multi(values: mlua::MultiValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        let mut values = values.into_vec();
        if values.len() == 1 {
            Ok(ArLuaResult(Ok(FromLua::from_lua(values.remove(0), lua)?)))
        } else if values.len() == 2 {
            if values[0].is_nil() {
                Ok(ArLuaResult(Err(lua.from_value(values.remove(1))?)))
            } else {
                Err(LuaError::external("Multiple values not supported"))
            }
        } else if values.is_empty() {
            unreachable!()
        } else {
            Err(LuaError::external("Too many values"))
        }
    }
}

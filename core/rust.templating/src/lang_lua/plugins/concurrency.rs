use mlua::{prelude::*, Variadic};
/*
    let select = LuaFunction::wrap_async(|_, futs: Variadic<LuaFunction>| async move {
        let (res, _) = futures_util::future::select_ok(
            futs.into_iter()
                .map(|f| Box::pin(f.call_async::<_, LuaValue>(()))),
        )
        .await?;
        Ok(res)
    });
*/

pub fn init_plugin(lua: &Lua) -> LuaResult<LuaTable> {
    let module = lua.create_table()?;

    module.set(
        "select",
        lua.create_async_function(|_, futs: Variadic<LuaFunction>| async move {
            if futs.len() == 0 {
                return Err(LuaError::external(
                    "select() requires at least one argument",
                ));
            }

            let (res, _) = futures_util::future::select_ok(
                futs.into_iter()
                    .map(|f| Box::pin(f.call_async::<_, LuaValue>(()))),
            )
            .await?;
            Ok(res)
        })?,
    )?;

    module.set_readonly(true); // Block any attempt to modify this table

    Ok(module)
}

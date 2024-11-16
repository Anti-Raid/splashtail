use tokio::time::sleep;
use std::time::{Instant, Duration};
use crate::lang_lua::state;
use mlua::prelude::*;

pub fn plugin_docs() -> templating_docgen::Plugin {
    templating_docgen::Plugin::default()
        .name("@antiraid/async")
        .description("Utilities for asynchronous operations and timing")
        .method_mut("sleep", |m| {
            m.description("Sleep for a given duration.")
            .parameter("duration", |p| {
                p.typ("f64").description("The duration to sleep for.")
            })
            .return_("slept_time", |r| {
                r.typ("f64").description("The actual duration slept for.")
            })
        })

}

pub fn init_plugin(lua: &Lua) -> LuaResult<LuaTable> {
    let module = lua.create_table()?;

    module.set(
        "sleep",
        lua.create_async_function(|lua, duration: f64| async move {
            let last_exec_time = {
                let Some(data) = lua.app_data_ref::<state::LuaUserData>() else {
                    return Err(LuaError::external("No app data found"));
                };
    
                // Get the last_execution_time of the VM
                data.last_execution_time
                .load(std::sync::atomic::Ordering::Acquire) // Get the elapsed time since the last execution
            };

            let start = Instant::now();

            // If the VM would timeout before the sleep duration, return an error
            if (start + Duration::from_secs_f64(duration)) > (last_exec_time + crate::lang_lua::MAX_TEMPLATE_LIFETIME) {
                return Err(LuaError::external("Unsafe operation attempted: sleep duration would exceed maximum VM execution time."));
            }                        

            sleep(Duration::from_secs_f64(duration)).await;
            let after = Instant::now();
            Ok((after - start).as_secs_f64())
        })?,
    )?;

    module.set_readonly(true); // Block any attempt to modify this table

    Ok(module)
}

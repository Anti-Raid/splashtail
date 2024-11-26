use super::state;
use super::{resolve_template_to_bytecode, ArLuaThreadInnerState, LuaVmAction, LuaVmResult};
use mlua::prelude::*;

/// Handles a Lua VM action, returning a result
pub async fn handle_event(action: LuaVmAction, tis_ref: &ArLuaThreadInnerState) -> LuaVmResult {
    match action {
        LuaVmAction::Exec {
            content,
            template,
            pragma,
            event,
        } => {
            if tis_ref.broken.load(std::sync::atomic::Ordering::Acquire) {
                return LuaVmResult::VmBroken {};
            }

            // Check bytecode cache first, compile template if not found
            let template_bytecode = match resolve_template_to_bytecode(
                content,
                template.clone(),
                &tis_ref.bytecode_cache,
                &tis_ref.compiler,
            )
            .await
            {
                Ok(bytecode) => bytecode,
                Err(e) => {
                    return LuaVmResult::LuaError { err: e };
                }
            };

            let token = match state::add_template(
                &tis_ref.lua,
                match template {
                    crate::Template::Raw(_) => "".to_string(),
                    crate::Template::Named(ref name) => name.clone(),
                },
                template.clone(),
                pragma,
            ) {
                Ok(token) => token,
                Err(e) => {
                    return LuaVmResult::LuaError {
                        err: LuaError::external(e),
                    };
                }
            };

            let exec_name = match template {
                crate::Template::Raw(_) => "script".to_string(),
                crate::Template::Named(ref name) => name.to_string(),
            };

            let v: LuaValue = match tis_ref
                .lua
                .load(&template_bytecode)
                .set_name(&exec_name)
                .set_mode(mlua::ChunkMode::Binary) // Ensure auto-detection never selects binary mode
                .call_async((event, token.clone()))
                .await
            {
                Ok(f) => f,
                Err(e) => {
                    while let Err(e) = state::remove_template(&tis_ref.lua, &token) {
                        log::error!(
                            "Could not remove template: {}. Trying again in 300 milliseconds",
                            e
                        );
                        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                    }

                    return LuaVmResult::LuaError { err: e };
                }
            };

            while let Err(e) = state::remove_template(&tis_ref.lua, &token) {
                log::error!(
                    "Could not remove template: {}. Trying again in 300 milliseconds",
                    e
                );
                tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            }

            match tis_ref.lua.from_value::<serde_json::Value>(v) {
                Ok(v) => {
                    return LuaVmResult::Ok { result_val: v };
                }
                Err(e) => {
                    return LuaVmResult::LuaError { err: e };
                }
            }
        }
        LuaVmAction::Stop {} => {
            // Mark VM as broken
            tis_ref
                .broken
                .store(true, std::sync::atomic::Ordering::Release);
            return LuaVmResult::Ok {
                result_val: serde_json::Value::Null,
            };
        }
        LuaVmAction::GetMemoryUsage {} => {
            let used = tis_ref.lua.used_memory();
            return LuaVmResult::Ok {
                result_val: serde_json::Value::Number(used.into()),
            };
        }
        LuaVmAction::SetMemoryLimit { limit } => {
            match tis_ref.lua.set_memory_limit(limit) {
                Ok(limit) => {
                    return LuaVmResult::Ok {
                        result_val: serde_json::Value::Number(limit.into()),
                    };
                }
                Err(e) => {
                    return LuaVmResult::LuaError { err: e };
                }
            };
        }
    }
}

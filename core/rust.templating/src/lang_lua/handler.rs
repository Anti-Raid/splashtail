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
                    return LuaVmResult::LuaError { err: e.to_string() };
                }
            };

            let exec_name = match template {
                crate::Template::Raw(_) => "script".to_string(),
                crate::Template::Named(ref name) => name.to_string(),
            };

            // Now, create the template context that should be passed to the template
            let template_context = super::ctx::TemplateContext::new(super::state::TemplateData {
                path: match template {
                    crate::Template::Raw(_) => "".to_string(),
                    crate::Template::Named(ref name) => name.clone(),
                },
                template,
                pragma,
            });

            let v: LuaValue = match tis_ref
                .lua
                .load(&template_bytecode)
                .set_name(&exec_name)
                .set_mode(mlua::ChunkMode::Binary) // Ensure auto-detection never selects binary mode
                .call_async((event, template_context))
                .await
            {
                Ok(f) => f,
                Err(e) => {
                    // Mark memory error'd VMs as broken automatically to avoid user grief/pain
                    match e {
                        LuaError::MemoryError(_) => {
                            // Mark VM as broken
                            tis_ref
                                .broken
                                .store(true, std::sync::atomic::Ordering::Release);
                        }
                        _ => {}
                    }

                    return LuaVmResult::LuaError { err: e.to_string() };
                }
            };

            match tis_ref.lua.from_value::<serde_json::Value>(v) {
                Ok(v) => {
                    return LuaVmResult::Ok { result_val: v };
                }
                Err(e) => {
                    return LuaVmResult::LuaError { err: e.to_string() };
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
                    return LuaVmResult::LuaError { err: e.to_string() };
                }
            };
        }
    }
}

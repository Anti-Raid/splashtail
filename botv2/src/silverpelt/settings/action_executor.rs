use super::config_opts::ColumnAction;
use super::state::State;
use crate::silverpelt::value::Value;
use async_recursion::async_recursion;
use mlua::LuaSerdeExt;
use sqlx::Row;

fn _getluavm() -> mlua::Lua {
    let lua = mlua::Lua::new();

    let string_extrafuncs = r#"
function string:contains(sub)
    return self:find(sub, 1, true) ~= nil
end

function string:startswith(start)
    return self:sub(1, #start) == start
end

function string:endswith(ending)
    return ending == "" or self:sub(-#ending) == ending
end

function string:replace(old, new)
    local s = self
    local search_start_idx = 1

    while true do
        local start_idx, end_idx = s:find(old, search_start_idx, true)
        if (not start_idx) then
            break
        end

        local postfix = s:sub(end_idx + 1)
        s = s:sub(1, (start_idx - 1)) .. new .. postfix

        search_start_idx = -1 * postfix:len()
    end

    return s
end

function string:insert(pos, text)
    return self:sub(1, pos - 1) .. text .. self:sub(pos)
end
    "#;

    lua.load(string_extrafuncs).exec().unwrap();

    lua
}

#[allow(dead_code)]
#[async_recursion]
pub async fn execute_actions(
    state: &mut State,
    actions: &[ColumnAction],
    ctx: &serenity::all::Context,
) -> Result<(), crate::Error> {
    let cache_http = botox::cache::CacheHttpImpl::from_ctx(ctx);
    let data = &ctx.data::<crate::Data>();
    for action in actions {
        match action {
            ColumnAction::IpcPerModuleFunction {
                module,
                function,
                arguments,
            } => {
                // Get the toggle
                let toggle = crate::ipc::animus_magic::bot::dynamic::PERMODULE_FUNCTIONS
                    .get(&(module.to_string(), function.to_string()));

                let Some(toggle) = toggle else {
                    return Err(format!(
                        "No IPC function found for module {} and function {}",
                        module, function
                    )
                    .into());
                };

                let mut args = indexmap::IndexMap::new();

                for (key, value) in arguments {
                    let key = key.to_string();
                    let value = value.to_string();
                    let Some(state_value) = state.state.get(&value) else {
                        return Err(format!("State variable {} not found", key).into());
                    };

                    args.insert(key, state_value.clone());
                }

                toggle(&cache_http, &args).await?;
            }
            ColumnAction::CollectColumnToMap {
                table,
                column,
                key,
                fetch_all,
            } => {
                if *fetch_all {
                    let result = sqlx::query(&format!("SELECT {} FROM {}", column, table))
                        .fetch_all(&data.pool)
                        .await?;

                    // Note: Now parse the PgRow to a Value
                    let mut value: Vec<Value> = Vec::new();

                    for row in result {
                        value.push(Value::from_sqlx(&row, 0)?);
                    }

                    state.state.insert(key.to_string(), Value::List(value));
                } else {
                    let result =
                        sqlx::query(&format!("SELECT {}::jsonb FROM {} LIMIT 1", column, table))
                            .fetch_one(&data.pool)
                            .await?;

                    let v = result.try_get::<serde_json::Value, _>(0)?;
                    state.state.insert(key.to_string(), Value::from_json(&v));
                }
            }
            ColumnAction::Error { message } => {
                return Err(state.template_to_string(message).into());
            }
            ColumnAction::ExecLuaScript {
                script,
                on_success,
                on_failure,
            } => {
                let script = state.template_to_string(script);

                let res = {
                    let vm = _getluavm();

                    // Load in the state
                    let globals = vm.globals();

                    for (key, value) in state.state.iter() {
                        let v = value.to_json();

                        // Convert serde_json::Value to mlua::Value using serde
                        let v: mlua::Value = vm.to_value(&v)?;

                        globals.set(key.to_string(), v)?;
                    }

                    vm.load(&script).eval::<bool>()?
                };

                if res {
                    execute_actions(state, on_success, ctx).await?;
                } else {
                    execute_actions(state, on_failure, ctx).await?;
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_luavm() {
        assert!(_getluavm()
            .load(
                r#"
                            s = "Hello, world!"
                            return s:contains("world")
                        "#,
            )
            .eval::<bool>()
            .unwrap());
    }
}

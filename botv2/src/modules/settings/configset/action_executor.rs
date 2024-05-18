use super::state::State;
use super::value::Value;
use crate::silverpelt::config_opts::ColumnAction;
use sqlx::Row;

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

                    args.insert(key, state_value.to_cbor());
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
                // TODO
            }
        }
    }
    Ok(())
}

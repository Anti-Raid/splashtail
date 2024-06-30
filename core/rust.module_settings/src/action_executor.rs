use super::state::State;
use super::types::{ActionConditionContext, ColumnAction, NativeActionContext, SettingsError};
use async_recursion::async_recursion;
use splashcore_rs::value::Value;

#[allow(dead_code)]
#[async_recursion]
pub async fn execute_actions(
    state: &mut State,
    actions: &[ColumnAction],
    cache_http: &botox::cache::CacheHttpImpl,
    pool: &sqlx::PgPool,
    author: serenity::all::UserId,
    guild_id: serenity::all::GuildId,
    permodule_executor: &Box<dyn base_data::permodule::PermoduleFunctionExecutor>,
) -> Result<(), SettingsError> {
    for action in actions {
        match action {
            ColumnAction::IpcPerModuleFunction {
                module,
                function,
                arguments,
                on_condition,
            } => {
                if let Some(on_condition) = on_condition {
                    let acc = ActionConditionContext { author, guild_id };

                    match (on_condition)(acc, state) {
                        Ok(true) => (),          // Go ahead
                        Ok(false) => continue,   // Skip execution
                        Err(e) => return Err(e), // Return error
                    }
                }

                // Get the toggle
                let mut args = indexmap::IndexMap::new();

                for (key, value) in arguments {
                    let key = key.to_string();
                    let value = state.template_to_string(author, guild_id, value);

                    args.insert(key, value);
                }

                match permodule_executor
                    .execute_permodule_function(cache_http, module, function, &args)
                    .await
                {
                    Ok(()) => (),
                    Err(e) => {
                        return Err(SettingsError::Generic {
                            message: format!(
                                "Error running IPC function: {} [args: {}]",
                                e,
                                args.iter()
                                    .map(|(k, v)| format!("{}: {}", k, v))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                            src: format!("execute_actions/{}::{}", module, function),
                            typ: "internal".to_string(),
                        });
                    }
                }
            }
            ColumnAction::NativeAction {
                action,
                on_condition,
            } => {
                if let Some(on_condition) = on_condition {
                    let acc = ActionConditionContext { author, guild_id };

                    match (on_condition)(acc, state) {
                        Ok(true) => (),          // Go ahead
                        Ok(false) => continue,   // Skip execution
                        Err(e) => return Err(e), // Return error
                    }
                }

                let nac = NativeActionContext {
                    author,
                    guild_id,
                    pool: pool.clone(),
                };
                action(nac, state).await?;
            }
            ColumnAction::SetVariable {
                key,
                value,
                on_condition,
            } => {
                if let Some(on_condition) = on_condition {
                    let acc = ActionConditionContext { author, guild_id };
                    match (on_condition)(acc, state) {
                        Ok(true) => (),          // Go ahead
                        Ok(false) => continue,   // Skip execution
                        Err(e) => return Err(e), // Return error
                    }
                }

                state.state.insert(key.to_string(), Value::from_json(value));
            }
        }
    }
    Ok(())
}

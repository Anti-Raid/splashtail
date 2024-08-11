use super::state::State;
use super::types::{
    ActionConditionContext, ColumnAction, NativeActionContext, OperationType, SettingsError,
};
use splashcore_rs::value::Value;

#[allow(clippy::too_many_arguments)]
pub async fn execute_actions(
    state: &mut State,
    operation_type: OperationType,
    actions: &[ColumnAction],
    author: serenity::all::UserId,
    guild_id: serenity::all::GuildId,
    data_store: &mut dyn super::types::DataStore,
    data: &base_data::Data,
    cache_http: &botox::cache::CacheHttpImpl,
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
                    let acc = ActionConditionContext {
                        operation_type,
                        author,
                        guild_id,
                    };

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
                    let value = state.template_to_string(value);

                    args.insert(key, value);
                }

                match data
                    .props
                    .permodule_executor()
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
                    let acc = ActionConditionContext {
                        operation_type,
                        author,
                        guild_id,
                    };

                    match (on_condition)(acc, state) {
                        Ok(true) => (),          // Go ahead
                        Ok(false) => continue,   // Skip execution
                        Err(e) => return Err(e), // Return error
                    }
                }

                let nac = NativeActionContext {
                    author,
                    guild_id,
                    operation_type,
                    data_store,
                    data,
                    cache_http,
                };
                action(nac, state).await?;
            }
            ColumnAction::SetVariable {
                key,
                value,
                on_condition,
            } => {
                if let Some(on_condition) = on_condition {
                    let acc = ActionConditionContext {
                        operation_type,
                        author,
                        guild_id,
                    };
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

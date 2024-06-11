use super::config_opts::{ColumnAction, NativeActionContext};
use super::state::State;
use crate::silverpelt::value::Value;
use async_recursion::async_recursion;

#[allow(dead_code)]
#[async_recursion]
pub async fn execute_actions(
    state: &mut State,
    actions: &[ColumnAction],
    ctx: &serenity::all::Context,
    author: serenity::all::UserId,
    guild_id: serenity::all::GuildId,
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
                    let value = state.template_to_string(author, guild_id, value);

                    args.insert(key, value);
                }

                toggle(&cache_http, &args).await?;
            }
            ColumnAction::Error { message } => {
                return Err(state
                    .template_to_string(author, guild_id, message)
                    .to_string()
                    .into());
            }
            ColumnAction::NativeAction { action } => {
                let nac = NativeActionContext {
                    author,
                    guild_id,
                    pool: data.pool.clone(),
                };
                action(nac, state).await?;
            }
            ColumnAction::SetVariable { key, value } => {
                state.state.insert(key.to_string(), Value::from_json(value));
            }
        }
    }
    Ok(())
}

use futures_util::future::FutureExt;
use serenity::all::GuildId;
use splashcore_rs::value::Value;
use std::sync::Arc;

pub async fn setup(data: &silverpelt::data::Data) -> Result<(), silverpelt::Error> {
    let sc = data.silverpelt_cache.clone();
    data.props.add_permodule_function(
        "settings",
        "invalidate_module_enabled_cache",
        Box::new(move |_, options| invalidate_module_enabled_cache(sc.clone(), options).boxed()),
    );

    Ok(())
}

/// Arguments:
///
/// - `module` - The module to invalidate module enabled cache [String]
/// - `guild_id` - The guild ID to clear the cache for. If not provided, the cache will be cleared globally [Option<String>]
pub async fn invalidate_module_enabled_cache(
    silverpelt_cache: Arc<silverpelt::cache::SilverpeltCache>,
    value: &indexmap::IndexMap<String, Value>,
) -> Result<(), silverpelt::Error> {
    let module = match value.get("module") {
        Some(Value::String(s)) => s,
        _ => return Err("`module` could not be parsed".into()),
    };

    let guild_id = value.get("guild_id");

    if let Some(guild_id) = guild_id {
        let guild_id = match guild_id {
            Value::String(s) => s.parse::<GuildId>()?,
            _ => return Err("`guild_id` could not be parsed".into()),
        };

        silverpelt_cache
            .module_enabled_cache
            .invalidate(&(guild_id, module.clone()))
            .await;
    } else {
        // Global enable/disable the module by iterating the entire cache
        for (k, _) in silverpelt_cache.module_enabled_cache.iter() {
            if k.1 == *module {
                silverpelt_cache
                    .module_enabled_cache
                    .invalidate(&(k.0, module.clone()))
                    .await;
            }
        }
    }

    Ok(())
}

use futures_util::future::FutureExt;
use serenity::all::GuildId;
use splashcore_rs::value::Value;
use std::sync::Arc;

pub async fn setup(data: &silverpelt::data::Data) -> Result<(), silverpelt::Error> {
    let sc = data.silverpelt_cache.clone();
    data.props.add_permodule_function(
        "settings",
        "toggle_module",
        Box::new(move |_, options| toggle_module(sc.clone(), options).boxed()),
    );

    Ok(())
}

/// Arguments:
///
/// - `module` - The module to toggle [String]
/// - `enabled` - Whether the module is enabled or not [bool]
/// - `guild_id` - The guild ID to clear the cache for. If not provided, the cache will be cleared globally [Option<String>]
pub async fn toggle_module(
    silverpelt_cache: Arc<silverpelt::cache::SilverpeltCache>,
    value: &indexmap::IndexMap<String, Value>,
) -> Result<(), silverpelt::Error> {
    let module = match value.get("module") {
        Some(Value::String(s)) => s,
        _ => return Err("`module` could not be parsed".into()),
    };

    let enabled = match value.get("enabled") {
        Some(Value::Boolean(b)) => *b,
        _ => return Err("`enabled` could not be parsed".into()),
    };

    let guild_id = value.get("guild_id");

    if let Some(guild_id) = guild_id {
        let guild_id = match guild_id {
            Value::String(s) => s.parse::<GuildId>()?,
            _ => return Err("`guild_id` could not be parsed".into()),
        };

        silverpelt_cache
            .module_enabled_cache
            .insert((guild_id, module.clone()), enabled)
            .await;
    } else {
        // Global enable/disable the module by iterating the entire cache
        for (k, v) in silverpelt_cache.module_enabled_cache.iter() {
            if k.1 == *module && enabled != v {
                silverpelt_cache
                    .module_enabled_cache
                    .insert((k.0, module.clone()), enabled)
                    .await;
            }
        }
    }

    Ok(())
}

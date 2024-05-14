use crate::silverpelt::silverpelt_cache::SILVERPELT_CACHE;
use futures::future::FutureExt;
use serenity::all::GuildId;

pub async fn setup(_data: &crate::Data) -> Result<(), crate::Error> {
    crate::ipc::animus_magic::bot::dynamic::PERMODULE_FUNCTIONS.insert(
        ("settings".to_string(), "toggle_module".to_string()),
        Box::new(move |options| toggle_module(options).boxed()),
    );

    Ok(())
}

/// Arguments:
///
/// - `module` - The module to toggle [String]
/// - `enabled` - Whether the module is enabled or not [bool]
/// - `guild_id` - The guild ID to clear the cache for. If not provided, the cache will be cleared globally [Option<String>]
pub async fn toggle_module(
    value: &indexmap::IndexMap<String, serde_cbor::Value>,
) -> Result<(), crate::Error> {
    let module = match value.get("module") {
        Some(serde_cbor::Value::Text(s)) => s,
        _ => return Err("`module` could not be parsed".into()),
    };

    let enabled = match value.get("enabled") {
        Some(serde_cbor::Value::Bool(b)) => *b,
        _ => return Err("`enabled` could not be parsed".into()),
    };

    let guild_id = value.get("guild_id");

    if let Some(guild_id) = guild_id {
        let guild_id = match guild_id {
            serde_cbor::Value::Text(s) => s.parse::<GuildId>()?,
            _ => return Err("`guild_id` could not be parsed".into()),
        };

        if enabled {
            SILVERPELT_CACHE
                .module_enabled_cache
                .insert((guild_id, module.clone()), true)
                .await;
        } else {
            SILVERPELT_CACHE
                .module_enabled_cache
                .insert((guild_id, module.clone()), false)
                .await;
        }

        tokio::spawn(async move {
            if let Err(err) = SILVERPELT_CACHE
                .command_permission_cache
                .invalidate_entries_if(move |k, _| k.0 == guild_id)
            {
                log::error!(
                    "Failed to invalidate command permission cache for guild {}: {}",
                    guild_id,
                    err
                );
            } else {
                log::info!("Invalidated cache for guild {}", guild_id);
            }
        });
    } else {
        // Global enable/disable the module by iterating the entire cache
        for (k, v) in SILVERPELT_CACHE.module_enabled_cache.iter() {
            if k.1 == *module && enabled != v {
                SILVERPELT_CACHE
                    .module_enabled_cache
                    .insert((k.0, module.clone()), enabled)
                    .await;

                // Invalidate command permission cache entries here too
                let gid = k.0;
                tokio::spawn(async move {
                    if let Err(err) = SILVERPELT_CACHE
                        .command_permission_cache
                        .invalidate_entries_if(move |g, _| g.0 == gid)
                    {
                        log::error!(
                            "Failed to invalidate command permission cache for guild {}: {}",
                            k.0,
                            err
                        );
                    } else {
                        log::info!("Invalidated cache for guild {}", k.0);
                    }
                });
            }
        }
    }

    Ok(())
}

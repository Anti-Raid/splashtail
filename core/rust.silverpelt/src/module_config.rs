use crate::{
    cache::SilverpeltCache,
    types::{GuildCommandConfiguration, GuildModuleConfiguration},
    utils::permute_command_names,
    CommandExtendedData,
};
use serenity::all::GuildId;
use sqlx::PgPool;

/// Returns whether or not a module is enabled based on cache and/or database
///
/// Note that fetching directly from database may be more reliable in certain cases
/// such as module_enable/disable and as such *SHOULD* be used there. This function
/// should only be called for cases where querying the database every time would be
/// too great a cost
#[allow(dead_code)] // This function is a useful utility function
pub async fn is_module_enabled(
    silverpelt_cache: &SilverpeltCache,
    pool: &PgPool,
    guild_id: GuildId,
    module: &str,
) -> Result<bool, crate::Error> {
    if let Some(state) = silverpelt_cache
        .module_enabled_cache
        .get(&(guild_id, module.to_string()))
        .await
    {
        Ok(state)
    } else {
        // Fetch from database
        let disabled = sqlx::query!(
            "SELECT disabled FROM guild_module_configurations WHERE guild_id = $1 AND module = $2",
            guild_id.to_string(),
            module
        )
        .fetch_optional(pool)
        .await?;

        if let Some(disabled) = disabled {
            if let Some(disabled) = disabled.disabled {
                silverpelt_cache
                    .module_enabled_cache
                    .insert((guild_id, module.to_string()), !disabled)
                    .await;
                Ok(!disabled)
            } else {
                // User wants to use the default value
                let module = silverpelt_cache
                    .module_cache
                    .get(module)
                    .ok_or::<crate::Error>(
                        format!("Could not find module {} in cache", module).into(),
                    )?;

                silverpelt_cache
                    .module_enabled_cache
                    .insert(
                        (guild_id, module.id().to_string()),
                        module.is_default_enabled(),
                    )
                    .await;
                Ok(module.is_default_enabled())
            }
        } else {
            // User wants to use the default value
            let module = silverpelt_cache
                .module_cache
                .get(module)
                .ok_or::<crate::Error>(
                    format!("Could not find module {} in cache", module).into(),
                )?;

            silverpelt_cache
                .module_enabled_cache
                .insert(
                    (guild_id, module.id().to_string()),
                    module.is_default_enabled(),
                )
                .await;
            Ok(module.is_default_enabled())
        }
    }
}

/// Returns the configuration for a module, if it exists
pub async fn get_module_configuration(
    pool: &PgPool,
    guild_id: &str,
    module: &str,
) -> Result<Option<GuildModuleConfiguration>, crate::Error> {
    let rec = sqlx::query!(
        "SELECT id, guild_id, module, disabled FROM guild_module_configurations WHERE guild_id = $1 AND module = $2",
        guild_id,
        module,
    )
    .fetch_optional(pool)
    .await?;

    if let Some(rec) = rec {
        Ok(Some(GuildModuleConfiguration {
            id: rec.id.hyphenated().to_string(),
            guild_id: rec.guild_id,
            module: rec.module,
            disabled: rec.disabled,
        }))
    } else {
        Ok(None)
    }
}

/// Returns the extended data for a command
pub fn get_command_extended_data(
    silverpelt_cache: &SilverpeltCache,
    permutations: &[String],
) -> Result<CommandExtendedData, crate::Error> {
    let root_cmd = permutations.first().unwrap();

    let root_cmd_data = silverpelt_cache.command_extra_data_map.get(root_cmd);

    let Some(root_cmd_data) = root_cmd_data else {
        return Err(format!(
            "The command ``{}`` does not exist [command not found in cache?]",
            root_cmd
        )
        .into());
    };

    let mut cmd_data = root_cmd_data
        .get("")
        .unwrap_or(&CommandExtendedData::kittycat_or_admin(root_cmd, "*"))
        .clone();

    for command in permutations.iter() {
        let cmd_replaced = command
            .replace(&root_cmd.to_string(), "")
            .trim()
            .to_string();

        if let Some(data) = root_cmd_data.get(&cmd_replaced.as_str()) {
            cmd_data = data.clone();
        }
    }

    Ok(cmd_data)
}

/// Gets the best possible command configuation to run a specific command
///
/// This assumes the base command is the first element in the permutations
///
/// Note that disabled+perms are inherited from parent commands if explicitly set on parent but not explicitly set on the command itself
///
/// @returns
///  Option<GuildCommandConfiguration> - The best command configuration to use
pub async fn get_best_command_configuration(
    pool: &PgPool,
    guild_id: &str,
    permutations: &[String],
) -> Result<Option<GuildCommandConfiguration>, crate::Error> {
    let mut command_configuration: Option<GuildCommandConfiguration> = None;
    for permutation in permutations.iter() {
        let rec = sqlx::query!(
            "SELECT id, guild_id, command, perms, disabled FROM guild_command_configurations WHERE guild_id = $1 AND command = $2",
            guild_id,
            permutation,
        )
        .fetch_optional(pool)
        .await?;

        // We are deeper in the tree, so we can overwrite the command configuration
        if let Some(rec) = rec {
            let new_command_configuration = GuildCommandConfiguration {
                id: rec.id.hyphenated().to_string(),
                guild_id: rec.guild_id,
                command: rec.command,
                perms: {
                    if let Some(perms) = rec.perms {
                        serde_json::from_value(perms)?
                    } else {
                        // Check parent command for perms
                        if let Some(ref bcc) = command_configuration {
                            bcc.perms.clone()
                        } else {
                            // No perms found, return None (no perms
                            None
                        }
                    }
                },
                disabled: {
                    if rec.disabled.is_some() {
                        rec.disabled
                    } else {
                        // Check parent command for disabled
                        if let Some(ref bcc) = command_configuration {
                            bcc.disabled
                        } else {
                            // No disabled found, return None
                            None
                        }
                    }
                },
            };

            command_configuration = Some(new_command_configuration);
        }
    }

    Ok(command_configuration)
}

// Gets the exact command configuation for a specific command
pub async fn get_exact_command_configuration(
    pool: &PgPool,
    guild_id: &str,
    command: &str,
) -> Result<Option<GuildCommandConfiguration>, crate::Error> {
    let mut command_configuration = None;
    let rec = sqlx::query!(
        "SELECT id, guild_id, command, perms, disabled FROM guild_command_configurations WHERE guild_id = $1 AND command = $2",
        guild_id,
        command
    )
    .fetch_optional(pool)
    .await?;

    if let Some(rec) = rec {
        command_configuration = Some(GuildCommandConfiguration {
            id: rec.id.hyphenated().to_string(),
            guild_id: rec.guild_id,
            command: rec.command,
            perms: {
                if let Some(perms) = rec.perms {
                    serde_json::from_value(perms)?
                } else {
                    None
                }
            },
            disabled: rec.disabled,
        });
    }

    Ok(command_configuration)
}

/// Returns all configurations of a command
#[allow(dead_code)]
pub async fn get_all_command_configurations(
    pool: &PgPool,
    guild_id: &str,
    name: &str,
) -> Result<Vec<GuildCommandConfiguration>, crate::Error> {
    let permutations = permute_command_names(name);

    let mut command_configurations = Vec::new();

    for permutation in permutations.iter() {
        let rec = sqlx::query!(
            "SELECT id, guild_id, command, perms, disabled FROM guild_command_configurations WHERE guild_id = $1 AND command = $2",
            guild_id,
            permutation,
        )
        .fetch_optional(pool)
        .await?;

        if let Some(rec) = rec {
            command_configurations.push(GuildCommandConfiguration {
                id: rec.id.hyphenated().to_string(),
                guild_id: rec.guild_id,
                command: rec.command,
                perms: {
                    if let Some(perms) = rec.perms {
                        serde_json::from_value(perms)?
                    } else {
                        None
                    }
                },
                disabled: rec.disabled,
            });
        }
    }

    Ok(command_configurations)
}

use super::silverpelt_cache::SILVERPELT_CACHE;
use crate::impls::cache::CacheHttpImpl;
use crate::silverpelt;
use serenity::all::{GuildId, UserId};
use sqlx::PgPool;
use log::info;

#[derive(Clone, Debug)]
pub struct CheckCommandError {
    pub code: String,
    pub message: String,
}

// Impl From trait for all types satisfying Display
impl<T: core::fmt::Display> From<T> for CheckCommandError {
    fn from(e: T) -> Self {
        Self {
            code: "generic_error".to_string(),
            message: e.to_string(),
        }
    }
}

pub async fn check_command(
    base_command: &str,
    command: &str,
    guild_id: GuildId,
    user_id: UserId,
    pool: &PgPool,
    cache_http: &CacheHttpImpl
) -> Result<String, CheckCommandError> {
    if !SILVERPELT_CACHE
                    .command_id_module_map
                    .contains_key(base_command)
                {
                    return Err(
                        "This command is not registered in the database, please contact support"
                            .into(),
                    );
                }

                let module = SILVERPELT_CACHE
                .command_id_module_map
                .get(base_command)
                .unwrap();

                if module == "root" {
                    if !crate::config::CONFIG
                        .discord_auth
                        .root_users
                        .contains(&user_id)
                    {
                        return Err("Root commands are off-limits unless you are a bot owner or otherwise have been granted authorization!".into());
                    }
                    return Ok("root_cmd".to_string());
                }

                if ["register"].contains(&base_command) {
                    return Ok("register_cmd".to_string());
                }

                let key = SILVERPELT_CACHE
                    .command_permission_cache
                    .get(&(guild_id, user_id))
                    .await;

                if let Some(ref map) = key {
                    let cpr = map.get(base_command);

                    if let Some(cpr) = cpr {
                        match cpr {
                            Ok(()) => return Ok("cached".to_string()),
                            Err(e) => {
                                return Err(e.clone())
                            }
                        }
                    }
                }

                let (cmd_data, command_config, module_config) =
                    silverpelt::module_config::get_command_configuration(
                        pool,
                        guild_id.to_string().as_str(),
                        command,
                    )
                    .await?;

                let command_config = command_config.unwrap_or(silverpelt::GuildCommandConfiguration {
                    id: "".to_string(),
                    guild_id: guild_id.to_string(),
                    command: command.to_string(),
                    perms: None,
                    disabled: None,
                });

                let module_config = module_config.unwrap_or(silverpelt::GuildModuleConfiguration {
                    id: "".to_string(),
                    guild_id: guild_id.to_string(),
                    module: module.clone(),
                    disabled: None,
                });

                let guild = guild_id.to_partial_guild(&cache_http).await?;
                let member = guild.member(&cache_http, user_id).await?;

                let (is_owner, member_perms) = {
                    let is_owner = member.user.id == guild.owner_id;

                    let member_perms = {
                        if is_owner {
                            serenity::model::permissions::Permissions::all()
                        } else {
                            guild.member_permissions(&member)
                        }
                    };

                    drop(guild);

                    (is_owner, member_perms)
                };

                if is_owner {
                    return Ok("owner".to_string());
                }

                let kittycat_perms = silverpelt::member_permission_calc::get_kittycat_perms(pool, guild_id, member.user.id, &member.roles).await?;

                info!(
                    "Checking if user {} ({}) can run command {} with permissions {:?}",
                    member.user.name,
                    member.user.id,
                    command,
                    member_perms
                );
                if let Err(e) = silverpelt::permissions::can_run_command(
                    &cmd_data,
                    &command_config,
                    &module_config,
                    command,
                    member_perms,
                    &kittycat_perms,
                ) {
                    let err = CheckCommandError {
                        code: e.0,
                        message: e.1.to_string(),
                    };

                    let mut key = SILVERPELT_CACHE
                    .command_permission_cache
                    .get(&(guild_id, user_id))
                    .await;
                
                    if let Some(ref mut map) = key {
                        map.insert(
                            command.to_string(),
                            Err(err.clone()),
                        );
                    } else {
                        let mut map = indexmap::IndexMap::new();
                        map.insert(
                            command.to_string(),
                            Err(err.clone()),
                        );
                        SILVERPELT_CACHE
                            .command_permission_cache
                            .insert((guild_id, user_id), map)
                            .await;
                    }

                    return Err(err.into());
                }

                let mut key = SILVERPELT_CACHE
                    .command_permission_cache
                    .get(&(guild_id, user_id))
                    .await;
                if let Some(ref mut map) = key {
                    map.insert(
                        command.to_string(),
                        Ok(()),
                    );
                } else {
                    let mut map = indexmap::IndexMap::new();
                    map.insert(
                        command.to_string(),
                        Ok(()),
                    );
                    SILVERPELT_CACHE
                        .command_permission_cache
                        .insert((guild_id, user_id), map)
                        .await;
                }

                Ok("".into())
}
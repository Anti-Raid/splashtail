use super::silverpelt_cache::SILVERPELT_CACHE;
use crate::silverpelt::{
    self,
    module_config::{
        get_best_command_configuration, get_command_extended_data, get_module_configuration,
    },
    permissions::PermissionChecksContext,
    utils::permute_command_names,
    GuildCommandConfiguration, GuildModuleConfiguration,
};
use botox::cache::CacheHttpImpl;
use kittycat::perms::Permission;
use log::debug;
use serde::{Deserialize, Serialize};
use serenity::all::{GuildId, UserId};
use serenity::small_fixed_array::FixedArray;
use splashcore_rs::types::silverpelt::PermissionResult;
use sqlx::PgPool;

#[inline]
pub async fn get_user_discord_info(
    guild_id: GuildId,
    user_id: UserId,
    cache_http: &CacheHttpImpl,
    poise_ctx: &Option<crate::Context<'_>>,
) -> Result<
    (
        bool,                              // is_owner
        UserId,                            // owner_id
        serenity::all::Permissions,        // member_perms
        FixedArray<serenity::all::RoleId>, // roles
    ),
    PermissionResult,
> {
    if let Some(cached_guild) = guild_id.to_guild_cached(&cache_http.cache) {
        // OPTIMIZATION: if owner, we dont need to continue further
        if user_id == cached_guild.owner_id {
            return Ok((
                true,                              // is_owner
                cached_guild.owner_id,             // owner_id
                serenity::all::Permissions::all(), // member_perms
                FixedArray::new(), // OPTIMIZATION: no role data is needed for perm checks for owners
            ));
        }

        // OPTIMIZATION: If we have a poise_ctx which is also a ApplicationContext, we can directly use it
        if let Some(poise::Context::Application(ref a)) = poise_ctx {
            if let Some(ref mem) = a.interaction.member {
                return Ok((
                    mem.user.id == cached_guild.owner_id,
                    cached_guild.owner_id,
                    cached_guild.member_permissions(mem),
                    mem.roles.clone(),
                ));
            }
        }

        // Now fetch the member, here calling member automatically tries to find in its cache first
        if let Some(member) = cached_guild.members.get(&user_id) {
            return Ok((
                member.user.id == cached_guild.owner_id,
                cached_guild.owner_id,
                cached_guild.member_permissions(member),
                member.roles.clone(),
            ));
        }
    }

    let guild = match guild_id.to_partial_guild(&cache_http).await {
        Ok(guild) => guild,
        Err(e) => {
            return Err(PermissionResult::DiscordError {
                error: e.to_string(),
            })
        }
    };

    // OPTIMIZATION: if owner, we dont need to continue further
    if user_id == guild.owner_id {
        return Ok((
            true,
            guild.owner_id,
            serenity::all::Permissions::all(),
            FixedArray::new(),
        ));
    }

    // OPTIMIZATION: If we have a poise_ctx which is also a ApplicationContext, we can directly use it
    if let Some(poise::Context::Application(ref a)) = poise_ctx {
        if let Some(ref mem) = a.interaction.member {
            return Ok((
                mem.user.id == guild.owner_id,
                guild.owner_id,
                guild.member_permissions(mem),
                mem.roles.clone(),
            ));
        }
    }

    let member = {
        let member = match proxy_support::member_in_guild(
            cache_http,
            &reqwest::Client::new(),
            guild_id,
            user_id,
        )
        .await
        {
            Ok(member) => member,
            Err(e) => {
                return Err(PermissionResult::DiscordError {
                    error: e.to_string(),
                });
            }
        };

        let Some(member) = member else {
            return Err(PermissionResult::DiscordError {
                error: "Member could not fetched".to_string(),
            });
        };

        member
    };

    Ok((
        member.user.id == guild.owner_id,
        guild.owner_id,
        guild.member_permissions(&member),
        member.roles.clone(),
    ))
}

pub async fn get_user_kittycat_perms(
    opts: &CheckCommandOptions,
    pool: &PgPool,
    guild_id: GuildId,
    guild_owner_id: UserId,
    user_id: UserId,
    roles: &FixedArray<serenity::all::RoleId>,
) -> Result<Vec<kittycat::perms::Permission>, crate::Error> {
    if let Some(ref custom_resolved_kittycat_perms) = opts.custom_resolved_kittycat_perms {
        if !opts.skip_custom_resolved_fit_checks {
            let kc_perms = silverpelt::member_permission_calc::get_kittycat_perms(
                pool,
                guild_id,
                guild_owner_id,
                user_id,
                roles,
            )
            .await?;

            let mut resolved_perms = Vec::new();
            for perm in custom_resolved_kittycat_perms {
                if kittycat::perms::has_perm(&kc_perms, perm) {
                    resolved_perms.push(perm.clone());
                }
            }

            Ok(resolved_perms)
        } else {
            Ok(custom_resolved_kittycat_perms.to_vec())
        }
    } else {
        Ok(silverpelt::member_permission_calc::get_kittycat_perms(
            pool,
            guild_id,
            guild_owner_id,
            user_id,
            roles,
        )
        .await?)
    }
}

/// Extra options for checking a command
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct CheckCommandOptions {
    /// Whether or not to ignore the cache
    #[serde(default)]
    pub ignore_cache: bool,

    /// Whether or not to cache the result at all
    #[serde(default)]
    pub cache_result: bool,

    /// Whether or not to ignore the fact that the module is disabled in the guild
    #[serde(default)]
    pub ignore_module_disabled: bool,

    /// Whether or not to ignore the fact that the command is disabled in the guild
    #[serde(default)]
    pub ignore_command_disabled: bool,

    /// Skip custom resolved kittycat permission fit 'checks' (AKA does the user have the actual permissions ofthe custom resolved permissions)
    #[serde(default)]
    pub skip_custom_resolved_fit_checks: bool,

    /// What custom resolved permissions to use for the user. API needs this for limiting the permissions of a user
    #[serde(default)]
    pub custom_resolved_kittycat_perms: Option<Vec<Permission>>,

    /// Custom command configuration to use
    #[serde(default)]
    pub custom_command_configuration: Option<GuildCommandConfiguration>,

    /// Custom module configuration to use
    #[serde(default)]
    pub custom_module_configuration: Option<GuildModuleConfiguration>,

    /// The current channel id
    #[serde(default)]
    pub channel_id: Option<serenity::all::ChannelId>,
}

impl Default for CheckCommandOptions {
    fn default() -> Self {
        Self {
            ignore_cache: false,
            cache_result: true,
            ignore_module_disabled: false,
            ignore_command_disabled: false,
            custom_resolved_kittycat_perms: None,
            skip_custom_resolved_fit_checks: false,
            custom_command_configuration: None,
            custom_module_configuration: None,
            channel_id: None,
        }
    }
}

/// Check command checks whether or not a user has permission to run a command
#[allow(clippy::too_many_arguments)]
pub async fn check_command(
    base_command: &str,
    command: &str,
    guild_id: GuildId,
    user_id: UserId,
    pool: &PgPool,
    cache_http: &CacheHttpImpl,
    // If a poise::Context is available and originates from a Application Command, we can fetch the guild+member from cache itself
    poise_ctx: &Option<crate::Context<'_>>,
    // Needed for settings and the website (potentially)
    opts: CheckCommandOptions,
) -> PermissionResult {
    if !SILVERPELT_CACHE
        .command_id_module_map
        .contains_key(base_command)
    {
        return "This command is not registered in the database, please contact support".into();
    }

    let module = match SILVERPELT_CACHE.command_id_module_map.get(base_command) {
        Some(module) => module,
        None => {
            return PermissionResult::ModuleNotFound {};
        }
    };

    if module == "root" {
        if !config::CONFIG.discord_auth.root_users.contains(&user_id) {
            return PermissionResult::SudoNotGranted {};
        }

        return PermissionResult::OkWithMessage {
            message: "root_cmd".to_string(),
        };
    }

    if !opts.ignore_cache {
        let key = SILVERPELT_CACHE
            .command_permission_cache
            .get(&(guild_id, user_id, opts.clone()))
            .await;

        if let Some(ref map) = key {
            let cpr = map.get(command);

            if let Some(cpr) = cpr {
                return cpr.clone();
            }
        }
    }

    debug!("Checking command {} for user {}", command, user_id);

    let permutations = permute_command_names(command);

    let mut module_config = {
        if let Some(ref custom_module_configuration) = opts.custom_module_configuration {
            custom_module_configuration.clone()
        } else {
            let gmc = match get_module_configuration(pool, &guild_id.to_string(), module.as_str())
                .await
            {
                Ok(v) => v,
                Err(e) => {
                    return e.into();
                }
            };

            gmc.unwrap_or(silverpelt::GuildModuleConfiguration {
                id: "".to_string(),
                guild_id: guild_id.to_string(),
                module: module.clone(),
                disabled: None,
                default_perms: None,
            })
        }
    };

    let cmd_data = match get_command_extended_data(&permutations) {
        Ok(v) => v,
        Err(e) => {
            return e.into();
        }
    };

    let mut command_config = {
        if let Some(ref custom_command_configuration) = opts.custom_command_configuration {
            custom_command_configuration.clone()
        } else {
            let gcc =
                match get_best_command_configuration(pool, &guild_id.to_string(), &permutations)
                    .await
                {
                    Ok(v) => v,
                    Err(e) => {
                        return e.into();
                    }
                };

            gcc.unwrap_or(silverpelt::GuildCommandConfiguration {
                id: "".to_string(),
                guild_id: guild_id.to_string(),
                command: command.to_string(),
                perms: None,
                disabled: None,
            })
        }
    };

    if opts.ignore_command_disabled {
        command_config.disabled = Some(false);
    }

    if opts.ignore_module_disabled {
        module_config.disabled = Some(false);
    }

    // Try getting guild+member from cache to speed up response times first
    let (is_owner, guild_owner_id, member_perms, roles) =
        match get_user_discord_info(guild_id, user_id, cache_http, poise_ctx).await {
            Ok(v) => v,
            Err(e) => {
                return e;
            }
        };

    if is_owner {
        return PermissionResult::OkWithMessage {
            message: "owner".to_string(),
        };
    }

    let kittycat_perms =
        match get_user_kittycat_perms(&opts, pool, guild_id, guild_owner_id, user_id, &roles).await
        {
            Ok(v) => v,
            Err(e) => {
                return e.into();
            }
        };

    let module_default_enabled = {
        let Some(module) = SILVERPELT_CACHE.module_cache.get(module) else {
            return PermissionResult::UnknownModule { module_config };
        };

        module.is_default_enabled
    };

    debug!(
        "Checking if user {} can run command {} with permissions {:?} with module_default_enabled {}",
        user_id, command, member_perms, module_default_enabled
    );

    let perm_res = super::permissions::can_run_command(
        &cmd_data,
        &command_config,
        &module_config,
        command,
        member_perms,
        kittycat_perms,
        module_default_enabled,
        PermissionChecksContext {
            guild_id,
            user_id,
            guild_owner_id,
            channel_id: opts.channel_id,
        },
    )
    .await;

    if !opts.cache_result {
        return perm_res;
    }

    let mut key = SILVERPELT_CACHE
        .command_permission_cache
        .get(&(guild_id, user_id, opts.clone()))
        .await;

    if let Some(ref mut map) = key {
        map.insert(command.to_string(), perm_res.clone());
    } else {
        let mut map = indexmap::IndexMap::new();
        map.insert(command.to_string(), perm_res.clone());
        SILVERPELT_CACHE
            .command_permission_cache
            .insert((guild_id, user_id, opts), map)
            .await;
    }

    perm_res
}

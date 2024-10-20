use crate::cache::SilverpeltCache;
use crate::{
    module_config::{
        get_best_command_configuration, get_command_extended_data, get_module_configuration,
    },
    types::{GuildCommandConfiguration, GuildModuleConfiguration},
    utils::permute_command_names,
};
use botox::cache::CacheHttpImpl;
use kittycat::perms::Permission;
use log::info;
use permissions::types::{PermissionChecks, PermissionResult};
use serde::{Deserialize, Serialize};
use serenity::all::{GuildId, UserId};
use serenity::small_fixed_array::FixedArray;
use sqlx::PgPool;

#[inline]
pub async fn get_user_discord_info(
    guild_id: GuildId,
    user_id: UserId,
    cache_http: &CacheHttpImpl,
    reqwest: &reqwest::Client,
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
    #[cfg(test)]
    {
        // Check for env var CHECK_MODULES_TEST_ENABLED, if so, return dummy data
        if std::env::var("CHECK_MODULES_TEST_ENABLED").unwrap_or_default() == "true" {
            return Ok((
                true,
                UserId::new(1),
                serenity::all::Permissions::all(),
                FixedArray::new(),
            ));
        }
    }

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
        let member =
            match sandwich_driver::member_in_guild(cache_http, reqwest, guild_id, user_id).await {
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
            let kc_perms = crate::member_permission_calc::get_kittycat_perms(
                &mut *pool.acquire().await?,
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
        Ok(crate::member_permission_calc::get_kittycat_perms(
            &mut *pool.acquire().await?,
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

#[allow(clippy::derivable_impls)]
impl Default for CheckCommandOptions {
    fn default() -> Self {
        Self {
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
    silverpelt_cache: &SilverpeltCache,
    command: &str,
    guild_id: GuildId,
    user_id: UserId,
    pool: &PgPool,
    cache_http: &CacheHttpImpl,
    reqwest: &reqwest::Client,
    // If a poise::Context is available and originates from a Application Command, we can fetch the guild+member from cache itself
    poise_ctx: &Option<crate::Context<'_>>,
    // Needed for settings and the website (potentially)
    opts: CheckCommandOptions,
) -> PermissionResult {
    let command_permutations = permute_command_names(command);

    let module_ref = match silverpelt_cache
        .command_id_module_map
        .try_get(&command_permutations[0])
    {
        dashmap::try_result::TryResult::Present(v) => v,
        dashmap::try_result::TryResult::Absent => {
            return PermissionResult::ModuleNotFound {};
        }
        dashmap::try_result::TryResult::Locked => {
            return PermissionResult::GenericError {
                error: "This module is being updated! Please try again later.".to_string(),
            };
        }
    };

    let module = match silverpelt_cache.module_cache.get(module_ref.value()) {
        Some(v) => v,
        None => {
            return PermissionResult::UnknownModule {
                module: module_ref.to_string(),
            };
        }
    };

    info!(
        "Checking if user {} can run command {} on module {}",
        user_id,
        command,
        module.id()
    );

    if module.root_module() {
        if !config::CONFIG.discord_auth.root_users.contains(&user_id) {
            return PermissionResult::SudoNotGranted {};
        }

        return PermissionResult::OkWithMessage {
            message: "root_cmd".to_string(),
        };
    }

    let module_config = {
        if let Some(ref custom_module_configuration) = opts.custom_module_configuration {
            custom_module_configuration.clone()
        } else {
            let gmc =
                match get_module_configuration(pool, &guild_id.to_string(), module_ref.value())
                    .await
                {
                    Ok(v) => v,
                    Err(e) => {
                        return e.into();
                    }
                };

            gmc.unwrap_or(GuildModuleConfiguration {
                id: "".to_string(),
                guild_id: guild_id.to_string(),
                module: module_ref.clone(),
                disabled: None,
                default_perms: None,
            })
        }
    };

    let cmd_data = match get_command_extended_data(silverpelt_cache, &command_permutations) {
        Ok(v) => v,
        Err(e) => {
            return e.into();
        }
    };

    let command_config = {
        if let Some(ref custom_command_configuration) = opts.custom_command_configuration {
            custom_command_configuration.clone()
        } else {
            let gcc = match get_best_command_configuration(
                pool,
                &guild_id.to_string(),
                &command_permutations,
            )
            .await
            {
                Ok(v) => v,
                Err(e) => {
                    return e.into();
                }
            };

            gcc.unwrap_or(GuildCommandConfiguration {
                id: "".to_string(),
                guild_id: guild_id.to_string(),
                command: command.to_string(),
                perms: None,
                disabled: None,
            })
        }
    };

    // Check if command is disabled if and only if ignore_command_disabled is false
    #[allow(clippy::collapsible_if)]
    if !opts.ignore_command_disabled {
        if command_config
            .disabled
            .unwrap_or(!cmd_data.is_default_enabled)
        {
            return PermissionResult::CommandDisabled {
                command: command.to_string(),
            };
        }
    }

    // Check if module is disabled if and only if ignore_module_disabled is false
    #[allow(clippy::collapsible_if)]
    if !opts.ignore_module_disabled {
        let module_default_enabled = {
            let Some(module) = silverpelt_cache.module_cache.get(module_ref.value()) else {
                return PermissionResult::UnknownModule {
                    module: module_ref.to_string(),
                };
            };

            module.is_default_enabled()
        };

        if module_config.disabled.unwrap_or(!module_default_enabled) {
            return PermissionResult::ModuleDisabled {
                module: module_ref.to_string(),
            };
        }
    }

    // Try getting guild+member from cache to speed up response times first
    let (is_owner, guild_owner_id, member_perms, roles) =
        match get_user_discord_info(guild_id, user_id, cache_http, reqwest, poise_ctx).await {
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

    // Check for permission checks in this order:
    // - command_config.perms
    // - module_config.default_perms
    // - cmd_data.default_perms
    let perms = {
        if let Some(perms) = &command_config.perms {
            perms
        } else if let Some(perms) = &module_config.default_perms {
            perms
        } else {
            &cmd_data.default_perms
        }
    };

    match perms {
        PermissionChecks::Simple { checks } => {
            if checks.is_empty() {
                return PermissionResult::Ok {};
            }

            permissions::eval_checks(checks, member_perms, kittycat_perms)
        }
        PermissionChecks::Template { template } => {
            match templating::execute(
                guild_id,
                templating::Template::Named(template.clone()),
                pool.clone(),
                cache_http.clone(),
                reqwest.clone(),
                PermissionTemplateContext {
                    member_native_permissions: member_perms,
                    member_kittycat_permissions: kittycat_perms,
                    user_id,
                    guild_id,
                    guild_owner_id,
                    channel_id: opts.channel_id,
                },
            )
            .await
            {
                Ok(v) => v,
                Err(e) => {
                    return PermissionResult::GenericError {
                        error: format!("Failed to render permission template: {}", e),
                    };
                }
            }
        }
    }
}

/// A PermissionTemplateContext is a context for permission templates
/// that can be accessed in permission templates
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct PermissionTemplateContext {
    pub member_native_permissions: serenity::all::Permissions,
    pub member_kittycat_permissions: Vec<kittycat::perms::Permission>,
    pub user_id: serenity::all::UserId,
    pub guild_id: serenity::all::GuildId,
    pub guild_owner_id: serenity::all::UserId,
    pub channel_id: Option<serenity::all::ChannelId>,
}

#[typetag::serde]
impl templating::Context for PermissionTemplateContext {}

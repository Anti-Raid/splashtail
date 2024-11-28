use botox::cache::CacheHttpImpl;
use kittycat::perms::Permission;
use log::info;
use permissions::types::{PermissionCheck, PermissionResult};
use serde::{Deserialize, Serialize};
use serenity::all::{GuildId, UserId};
use serenity::small_fixed_array::FixedArray;
use silverpelt::cache::SilverpeltCache;
use silverpelt::{
    module_config::{
        get_best_command_configuration, get_command_extended_data, get_module_configuration,
    },
    types::{GuildCommandConfiguration, GuildModuleConfiguration},
    utils::permute_command_names,
};
use sqlx::PgPool;

#[inline]
pub async fn get_user_discord_info(
    guild_id: GuildId,
    user_id: UserId,
    cache_http: &CacheHttpImpl,
    reqwest: &reqwest::Client,
    poise_ctx: &Option<silverpelt::Context<'_>>,
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
                    mem.permissions
                        .unwrap_or(splashcore_rs::serenity_backport::user_permissions(
                            mem.user.id,
                            &mem.roles,
                            cached_guild.id,
                            &cached_guild.roles,
                            cached_guild.owner_id,
                        )),
                    mem.roles.clone(),
                ));
            }
        }

        // Now fetch the member, here calling member automatically tries to find in its cache first
        if let Some(member) = cached_guild.members.get(&user_id) {
            return Ok((
                member.user.id == cached_guild.owner_id,
                cached_guild.owner_id,
                splashcore_rs::serenity_backport::user_permissions(
                    member.user.id,
                    &member.roles,
                    cached_guild.id,
                    &cached_guild.roles,
                    cached_guild.owner_id,
                ),
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
                mem.permissions
                    .unwrap_or(splashcore_rs::serenity_backport::user_permissions(
                        mem.user.id,
                        &mem.roles,
                        guild.id,
                        &guild.roles,
                        guild.owner_id,
                    )),
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
        splashcore_rs::serenity_backport::user_permissions(
            member.user.id,
            &member.roles,
            guild.id,
            &guild.roles,
            guild.owner_id,
        ),
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
) -> Result<Vec<kittycat::perms::Permission>, silverpelt::Error> {
    if let Some(ref custom_resolved_kittycat_perms) = opts.custom_resolved_kittycat_perms {
        let kc_perms = silverpelt::member_permission_calc::get_kittycat_perms(
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
        Ok(silverpelt::member_permission_calc::get_kittycat_perms(
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
    serenity_context: &serenity::all::Context,
    reqwest: &reqwest::Client,
    // If a poise::Context is available and originates from a Application Command, we can fetch the guild+member from cache itself
    poise_ctx: &Option<silverpelt::Context<'_>>,
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
    let (is_owner, guild_owner_id, member_perms, roles) = match get_user_discord_info(
        guild_id,
        user_id,
        &botox::cache::CacheHttpImpl::from_ctx(serenity_context),
        reqwest,
        poise_ctx,
    )
    .await
    {
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
    let check = {
        if let Some(perms) = &command_config.perms {
            perms
        } else {
            &cmd_data.default_perms
        }
    };

    match silverpelt::ar_event::dispatch_event_to_modules(
        &silverpelt::ar_event::EventHandlerContext {
            guild_id,
            data: serenity_context.data::<silverpelt::data::Data>(),
            event: silverpelt::ar_event::AntiraidEvent::Custom(silverpelt::ar_event::CustomEvent {
                event_name: "AR/CheckCommand".to_string(),
                event_titlename: "(Anti-Raid) Check Command".to_string(),
                event_data: serde_json::json!({
                    "command": command,
                    "user_id": user_id,
                    "member_native_perms": member_perms,
                    "member_kittycat_perms": kittycat_perms,
                    "opts": opts,
                    "check": check,
                    "module_config": module_config,
                    "command_config": command_config,
                    "command_extended_data": cmd_data,
                    "is_owner": is_owner,
                    "guild_owner_id": guild_owner_id,
                    "roles": roles,
                }),
            }),
            serenity_context: serenity_context.clone(),
        },
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            for (i, ei) in e.iter().enumerate() {
                if ei.to_string() == "AR/CheckCommand/Skip" {
                    return PermissionResult::OkWithMessage {
                        message: format!("IDX=>{},message=>SKIP", i),
                    };
                }
            }

            return PermissionResult::GenericError {
                error: e
                    .into_iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
                    .to_string(),
            };
        }
    };

    permissions::check_perms(check, member_perms, &kittycat_perms)
}

/// Returns whether a member has a kittycat permission
///
/// Note that in opts, only custom_resolved_kittycat_perms is used
pub async fn member_has_kittycat_perm(
    guild_id: GuildId,
    user_id: UserId,
    pool: &PgPool,
    serenity_context: &serenity::all::Context,
    reqwest: &reqwest::Client,
    // If a poise::Context is available and originates from a Application Command, we can fetch the guild+member from cache itself
    poise_ctx: &Option<silverpelt::Context<'_>>,
    perm: &kittycat::perms::Permission,
    opts: CheckCommandOptions,
) -> PermissionResult {
    // Try getting guild+member from cache to speed up response times first
    let (is_owner, guild_owner_id, member_perms, roles) = match get_user_discord_info(
        guild_id,
        user_id,
        &botox::cache::CacheHttpImpl::from_ctx(serenity_context),
        reqwest,
        poise_ctx,
    )
    .await
    {
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

    match silverpelt::ar_event::dispatch_event_to_modules(
        &silverpelt::ar_event::EventHandlerContext {
            guild_id,
            data: serenity_context.data::<silverpelt::data::Data>(),
            event: silverpelt::ar_event::AntiraidEvent::Custom(silverpelt::ar_event::CustomEvent {
                event_name: "AR/CheckKittycatPermissions".to_string(),
                event_titlename: "(Anti-Raid) Check Kittycat Permissions".to_string(),
                event_data: serde_json::json!({
                    "user_id": user_id,
                    "member_native_perms": member_perms,
                    "member_kittycat_perms": kittycat_perms,
                    "perm": perm,
                    "is_owner": is_owner,
                    "guild_owner_id": guild_owner_id,
                    "roles": roles,
                    "opts": opts,
                }),
            }),
            serenity_context: serenity_context.clone(),
        },
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            for (i, ei) in e.iter().enumerate() {
                if ei.to_string() == "AR/CheckKittycatPermissions/Skip" {
                    return PermissionResult::OkWithMessage {
                        message: format!("IDX=>{},message=>SKIP", i),
                    };
                }
            }

            return PermissionResult::GenericError {
                error: e
                    .into_iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
                    .to_string(),
            };
        }
    };

    if !kittycat::perms::has_perm(&kittycat_perms, perm) {
        return PermissionResult::MissingKittycatPerms {
            check: PermissionCheck {
                kittycat_perms: vec![perm.to_string()],
                native_perms: vec![],
                inner_and: false,
            },
        };
    }

    PermissionResult::Ok {}
}

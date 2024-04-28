use super::permissions::PermissionResult;
use super::silverpelt_cache::SILVERPELT_CACHE;
use botox::cache::CacheHttpImpl;
use crate::silverpelt::{
    self, 
    utils::permute_command_names, 
    CommandExtendedData, 
    GuildCommandConfiguration, 
    GuildModuleConfiguration,
    module_config::{
        get_module_configuration,
        get_command_extended_data,
        get_best_command_configuration,
    }
};
use log::info;
use serenity::all::{GuildId, UserId};
use serenity::small_fixed_array::FixedArray;
use sqlx::PgPool;
use serde::{Serialize, Deserialize};

/// Returns the effective configuration of a command
///
/// This is intentionally private as it is a helper function
/// to avoid rewriting a ton of code
#[inline]
async fn get_effective_module_command_configuration(
    pool: &PgPool,
    guild_id: &str,
    name: &str,
) -> Result<
    (
        CommandExtendedData,
        Option<GuildCommandConfiguration>,
        Option<GuildModuleConfiguration>,
    ),
    crate::Error,
> {
    let permutations = permute_command_names(name);
    let root_cmd = permutations.first().unwrap();

    let module = SILVERPELT_CACHE
        .command_id_module_map
        .get(root_cmd)
        .ok_or::<crate::Error>("Unknown error determining module of command".into())?;

    // Check if theres any module configuration
    let module_configuration = get_module_configuration(pool, guild_id, module.as_str()).await?;
    let cmd_data = get_command_extended_data(&permutations)?;
    let command_configuration = get_best_command_configuration(pool, guild_id, &permutations).await?;


    Ok((cmd_data, command_configuration, module_configuration))
}

#[inline]
pub async fn get_perm_info(
    guild_id: GuildId,
    user_id: UserId,
    cache_http: &CacheHttpImpl,
    poise_ctx: &Option<crate::Context<'_>>,
) -> Result<
    (
        bool, // is_owner
        UserId, // owner_id
        serenity::all::Permissions, // member_perms
        FixedArray<serenity::all::RoleId>, // roles
    ),
    PermissionResult,
> {
    if let Some(cached_guild) = guild_id.to_guild_cached(&cache_http.cache) {
        // OPTIMIZATION: if owner, we dont need to continue further
        if user_id == cached_guild.owner_id {
            return Ok((
                true, // is_owner
                cached_guild.owner_id, // owner_id
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

    let member = match guild.member(&cache_http, user_id).await {
        Ok(member) => member,
        Err(e) => {
            return Err(PermissionResult::DiscordError {
                error: e.to_string(),
            })
        }
    };

    Ok((
        member.user.id == guild.owner_id,
        guild.owner_id,
        guild.member_permissions(&member),
        member.roles.clone(),
    ))
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

    /// What custom resolved permissions to use for the user. Note that ensure_user_has_custom_resolved must be true to ensure that the user has all the permissions in the custom_resolved_kittycat_perms
    /// 
    /// API needs this for limiting the permissions of a user, allows setting custom resolved perms
    #[serde(default)]
    pub custom_resolved_kittycat_perms: Option<Vec<String>>,
    
    /// Whether or not to ensure that the user has all the permissions in the custom_resolved_kittycat_perms
    #[serde(default)]
    pub ensure_user_has_custom_resolved: bool,

    /// Custom permission checks to use
    #[serde(default)]
    pub custom_command_configuration: Option<GuildCommandConfiguration>,
}

impl Default for CheckCommandOptions {
    fn default() -> Self {
        Self {
            ignore_cache: false,
            cache_result: true,
            ignore_module_disabled: false,
            ignore_command_disabled: false,
            custom_resolved_kittycat_perms: None,
            ensure_user_has_custom_resolved: true,
            custom_command_configuration: None,
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
            return "Root commands are off-limits unless you are a bot owner or otherwise have been granted authorization!".into();
        }

        return PermissionResult::OkWithMessage {
            message: "root_cmd".to_string(),
        };
    }

    if ["register"].contains(&base_command) {
        return PermissionResult::OkWithMessage {
            message: "register_cmd".to_string(),
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

    let (cmd_data, command_config, module_config) =
        match get_effective_module_command_configuration(
            pool,
            guild_id.to_string().as_str(),
            command,
        )
        .await
        {
            Ok(v) => v,
            Err(e) => return e.into(),
        };

    let mut command_config = {
        if let Some(ref custom_command_configuration) = opts.custom_command_configuration {
            custom_command_configuration.clone()
        } else {
            command_config.unwrap_or(silverpelt::GuildCommandConfiguration {
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

    let mut module_config = module_config.unwrap_or(silverpelt::GuildModuleConfiguration {
        id: "".to_string(),
        guild_id: guild_id.to_string(),
        module: module.clone(),
        disabled: None,
    });

    if opts.ignore_module_disabled {
        module_config.disabled = Some(false);
    }

        // Try getting guild+member from cache to speed up response times first
        let (is_owner, guild_owner_id, member_perms, roles) =
            match get_perm_info(guild_id, user_id, cache_http, poise_ctx).await {
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
    
    let kittycat_perms = {
        if let Some(ref custom_resolved_kittycat_perms) = opts.custom_resolved_kittycat_perms {
            if opts.ensure_user_has_custom_resolved {
                let kc_perms = match silverpelt::member_permission_calc::get_kittycat_perms(
                    pool, guild_id, guild_owner_id, user_id, &roles,
                )
                .await
                {
                    Ok(v) => v,
                    Err(e) => {
                        return e.into();
                    }
                };

                let mut resolved_perms = Vec::new();
                for perm in custom_resolved_kittycat_perms.iter() {
                    if kittycat::perms::has_perm(&kc_perms, perm) {
                        resolved_perms.push(perm.to_string());
                    }  
                }

                resolved_perms
            } else {
                custom_resolved_kittycat_perms.to_vec()
            }
        } else {
            match silverpelt::member_permission_calc::get_kittycat_perms(
                pool, guild_id, guild_owner_id, user_id, &roles,
            )
            .await
            {
                Ok(v) => v,
                Err(e) => {
                    return e.into();
                }
            }
        }
    };

    info!(
        "Checking if user {} can run command {} with permissions {:?}",
        user_id, command, member_perms
    );

    let perm_res = silverpelt::permissions::can_run_command(
        &cmd_data,
        &command_config,
        &module_config,
        command,
        member_perms,
        &kittycat_perms,
    );

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

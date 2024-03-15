use super::silverpelt_cache::SILVERPELT_CACHE;
use crate::impls::cache::CacheHttpImpl;
use crate::silverpelt;
use serenity::all::{GuildId, UserId};
use sqlx::PgPool;
use super::permissions::PermissionResult;
use log::info;

pub async fn check_command(
    base_command: &str,
    command: &str,
    guild_id: GuildId,
    user_id: UserId,
    pool: &PgPool,
    cache_http: &CacheHttpImpl
) -> PermissionResult {
    if !SILVERPELT_CACHE
        .command_id_module_map
        .contains_key(base_command) {
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

    let key = SILVERPELT_CACHE
        .command_permission_cache
        .get(&(guild_id, user_id))
        .await;

    if let Some(ref map) = key {
        let cpr = map.get(command);

        if let Some(cpr) = cpr {
            return cpr.clone();
        }
    }

    let (cmd_data, command_config, module_config) =
        match silverpelt::module_config::get_command_configuration(
            pool,
            guild_id.to_string().as_str(),
            command,
        )
        .await {
            Ok(v) => v,
            Err(e) => { 
                return e.into() 
            }
        };

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

    #[inline]
    async fn get_perm_info(
        guild_id: GuildId,
        user_id: UserId,
        cache_http: &CacheHttpImpl,
    ) -> Result<(bool, serenity::all::Permissions, small_fixed_array::FixedArray<serenity::all::RoleId>), PermissionResult> {
        if let Some(cached_guild) = guild_id.to_guild_cached(&cache_http.cache) {
            // OPTIMIZATION: if owner, we dont need to continue further
            if user_id == cached_guild.owner_id {
                return Ok((true, serenity::all::Permissions::all(), small_fixed_array::FixedArray::new()));
            }
            
            // Now fetch the member, here calling member automatically tries to find in its cache first
            if let Some(member) = cached_guild.members.get(&user_id) {
                return Ok((member.user.id == cached_guild.owner_id, cached_guild.member_permissions(member), member.roles.clone()));
            }
        }

        let guild = match guild_id.to_partial_guild(&cache_http).await {
            Ok(guild) => guild,
            Err(e) => {
                return Err(PermissionResult::DiscordError { error: e.to_string() })
            }
        };

        // OPTIMIZATION: if owner, we dont need to continue further
        if user_id == guild.owner_id {
            return Ok((true, serenity::all::Permissions::all(), small_fixed_array::FixedArray::new()));
        }
    
        let member = match guild.member(&cache_http, user_id).await {
            Ok(member) => member,
            Err(e) => {
                return Err(PermissionResult::DiscordError { error: e.to_string() })
            }
        };

        Ok((member.user.id == guild.owner_id, guild.member_permissions(&member), member.roles.clone()))
    }

    // Try getting guild+member from cache to speed up response times first
    let (is_owner, member_perms, roles) = match get_perm_info(guild_id, user_id, cache_http).await {
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

    let kittycat_perms = match silverpelt::member_permission_calc::get_kittycat_perms(pool, guild_id, user_id, &roles).await {
        Ok(v) => v,
        Err(e) => {
            return e.into();
        }   
    };

    info!(
        "Checking if user {} can run command {} with permissions {:?}",
        user_id,
        command,
        member_perms
    );

    let perm_res = silverpelt::permissions::can_run_command(
        &cmd_data,
        &command_config,
        &module_config,
        command,
        member_perms,
        &kittycat_perms,
    );

    let mut key = SILVERPELT_CACHE
    .command_permission_cache
    .get(&(guild_id, user_id))
    .await;
            
    if let Some(ref mut map) = key {
        map.insert(
            command.to_string(),
            perm_res.clone(),
        );
    } else {
        let mut map = indexmap::IndexMap::new();
        map.insert(
            command.to_string(),
            perm_res.clone(),
        );
        SILVERPELT_CACHE
            .command_permission_cache
            .insert((guild_id, user_id), map)
            .await;
    }

    perm_res
}
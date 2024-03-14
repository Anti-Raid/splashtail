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
        let cpr = map.get(base_command);

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

    let guild = match guild_id.to_partial_guild(&cache_http).await {
        Ok(guild) => guild,
        Err(e) => {
            return PermissionResult::DiscordError { error: e.to_string() }
        }
    };

    let member = match guild.member(&cache_http, user_id).await {
        Ok(member) => member,
        Err(e) => {
            return PermissionResult::DiscordError { error: e.to_string() }
        }
    };

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
        return PermissionResult::OkWithMessage {
            message: "owner".to_string(),
        };
    }

    let kittycat_perms = match silverpelt::member_permission_calc::get_kittycat_perms(pool, guild_id, member.user.id, &member.roles).await {
        Ok(v) => v,
        Err(e) => {
            return e.into();
        }   
    };

    info!(
        "Checking if user {} ({}) can run command {} with permissions {:?}",
        member.user.name,
        member.user.id,
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
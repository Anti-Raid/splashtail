pub mod settings_execute;
pub mod templating_exec;
pub mod types;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use rust_rpc_server::AppData;
use std::sync::Arc;

type Response<T> = Result<Json<T>, (StatusCode, String)>;

pub fn create_bot_rpc_server(
    data: Arc<silverpelt::data::Data>,
    ctx: &serenity::all::Context,
) -> axum::routing::IntoMakeService<Router> {
    let router = rust_rpc_server::create_blank_rpc_server()
        // Returns the list of modules [Modules]
        .route("/modules", get(modules))
        // Given a list of guild ids, return a set of 0s and 1s indicating whether each guild exists in cache [GuildsExist]
        .route("/guilds-exist", get(guilds_exist))
        // Returns basic user/guild information [BaseGuildUserInfo]
        .route(
            "/base-guild-user-info/:guild_id/:user_id",
            get(base_guild_user_info),
        )
        // Returns if the user has permission to run a command on a given guild [CheckCommandPermission]
        .route(
            "/check-command-permission/:guild_id/:user_id",
            get(check_command_permission),
        )
        // Verify/parse a set of permission checks returning the parsed checks [ParsePermissionChecks]
        .route("/parse-permission-checks", get(parse_permission_checks))
        // Clears the modules enabled cache [ClearModulesEnabledCache]
        .route(
            "/clear-modules-enabled-cache",
            post(clear_modules_enabled_cache),
        )
        // Executes a template on a Lua VM
        .route(
            "/template-exec/:guild_id/:user_id",
            post(templating_exec::execute_template),
        )
        // Executes an operation on a setting [SettingsOperation]
        .route(
            "/settings-operation/:guild_id/:user_id",
            post(settings_execute::settings_operation),
        );
    let router: Router<()> = router.with_state(AppData::new(data, ctx));
    router.into_make_service()
}

/// Returns a list of modules [Modules]
async fn modules(
    State(AppData { data, .. }): State<AppData>,
) -> Json<Vec<silverpelt::canonical_module::CanonicalModule>> {
    let mut modules = Vec::new();

    for idm in data.silverpelt_cache.canonical_module_cache.iter() {
        let module = idm.value();
        modules.push(module.clone());
    }

    Json(modules)
}

/// Given a list of guild ids, return a set of 0s and 1s indicating whether each guild exists in cache [GuildsExist]
#[axum::debug_handler]
async fn guilds_exist(
    State(AppData {
        data, cache_http, ..
    }): State<AppData>,
    Json(guilds): Json<Vec<serenity::all::GuildId>>,
) -> Response<Vec<i32>> {
    let mut guilds_exist = Vec::with_capacity(guilds.len());

    for guild in guilds {
        let has_guild = sandwich_driver::has_guild(&cache_http, &data.reqwest, guild)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        guilds_exist.push({
            if has_guild {
                1
            } else {
                0
            }
        });
    }

    Ok(Json(guilds_exist))
}

/// Returns basic user/guild information [BaseGuildUserInfo]
async fn base_guild_user_info(
    State(AppData {
        data, cache_http, ..
    }): State<AppData>,
    Path((guild_id, user_id)): Path<(serenity::all::GuildId, serenity::all::UserId)>,
) -> Response<crate::types::BaseGuildUserInfo> {
    let bot_user_id = cache_http.cache.current_user().id;
    let guild = sandwich_driver::guild(&cache_http, &data.reqwest, guild_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get guild: {:#?}", e),
            )
        })?;

    // Next fetch the member and bot_user
    let member: serenity::model::prelude::Member =
        match sandwich_driver::member_in_guild(&cache_http, &data.reqwest, guild_id, user_id).await
        {
            Ok(Some(member)) => member,
            Ok(None) => {
                return Err((StatusCode::NOT_FOUND, "User not found".into()));
            }
            Err(e) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to get member: {:#?}", e),
                ));
            }
        };

    let bot_user: serenity::model::prelude::Member =
        match sandwich_driver::member_in_guild(&cache_http, &data.reqwest, guild_id, bot_user_id)
            .await
        {
            Ok(Some(member)) => member,
            Ok(None) => {
                return Err((StatusCode::NOT_FOUND, "Bot user not found".into()));
            }
            Err(e) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to get bot user: {:#?}", e),
                ));
            }
        };

    // Fetch the channels
    let channels = sandwich_driver::guild_channels(&cache_http, &data.reqwest, guild_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get channels: {:#?}", e),
            )
        })?;

    let mut channels_with_permissions = Vec::with_capacity(channels.len());

    for channel in channels.iter() {
        channels_with_permissions.push(crate::types::GuildChannelWithPermissions {
            user: guild.user_permissions_in(channel, &member),
            bot: guild.user_permissions_in(channel, &bot_user),
            channel: channel.clone(),
        });
    }

    Ok(Json(crate::types::BaseGuildUserInfo {
        name: guild.name.to_string(),
        icon: guild.icon_url(),
        owner_id: guild.owner_id.to_string(),
        roles: guild.roles.into_iter().collect(),
        user_roles: member.roles.to_vec(),
        bot_roles: bot_user.roles.to_vec(),
        channels: channels_with_permissions,
    }))
}

/// Returns if the user has permission to run a command on a given guild [CheckCommandPermission]
async fn check_command_permission(
    State(AppData {
        data,
        serenity_context,
        ..
    }): State<AppData>,
    Path((guild_id, user_id)): Path<(serenity::all::GuildId, serenity::all::UserId)>,
    Json(req): Json<crate::types::CheckCommandPermissionRequest>,
) -> Response<crate::types::CheckCommandPermission> {
    let opts = req.opts;

    let flags = crate::types::RpcCheckCommandOptionsFlags::from_bits_truncate(opts.flags);

    let perm_res = permission_checks::check_command(
        &data.silverpelt_cache,
        &req.command,
        guild_id,
        user_id,
        &data.pool,
        &serenity_context,
        &data.reqwest,
        &None,
        permission_checks::CheckCommandOptions {
            ignore_module_disabled: flags
                .contains(crate::types::RpcCheckCommandOptionsFlags::IGNORE_MODULE_DISABLED),
            ignore_command_disabled: flags
                .contains(crate::types::RpcCheckCommandOptionsFlags::IGNORE_COMMAND_DISABLED),
            custom_resolved_kittycat_perms: opts.custom_resolved_kittycat_perms.map(|crkp| {
                crkp.iter()
                    .map(|x| kittycat::perms::Permission::from_string(x))
                    .collect::<Vec<kittycat::perms::Permission>>()
            }),
            custom_command_configuration: opts.custom_command_configuration.map(|x| *x),
            custom_module_configuration: opts.custom_module_configuration.map(|x| *x),
            channel_id: opts.channel_id,
        },
    )
    .await;

    let is_ok = perm_res.is_ok();

    Ok(Json(crate::types::CheckCommandPermission {
        perm_res,
        is_ok,
    }))
}

/// Verify/parse a permission check returning the parsed check [ParsePermissionChecks]
async fn parse_permission_checks(
    State(AppData { .. }): State<AppData>,
    Json(checks): Json<permissions::types::PermissionCheck>,
) -> Response<permissions::types::PermissionCheck> {
    let parsed_checks = permissions::parse::parse_permission_check(&checks)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to parse permission check: {:#?}", e),
            )
        })?;

    Ok(Json(parsed_checks))
}

// Clears the modules enabled cache [ClearModulesEnabledCache]
async fn clear_modules_enabled_cache(
    State(AppData { data, .. }): State<AppData>,
    Json(req): Json<crate::types::ClearModulesEnabledCacheRequest>,
) -> Response<crate::types::ClearModulesEnabledCacheResponse> {
    if let Some(guild_id) = req.guild_id {
        if let Some(module) = req.module {
            data.silverpelt_cache
                .module_enabled_cache
                .invalidate(&(guild_id, module))
                .await;
        } else {
            // Global enable/disable the module by iterating the entire cache
            for (k, _) in data.silverpelt_cache.module_enabled_cache.iter() {
                if k.0 == guild_id {
                    data.silverpelt_cache
                        .module_enabled_cache
                        .invalidate(&(k.0, k.1.clone()))
                        .await;
                }
            }
        }
    } else {
        data.silverpelt_cache.module_enabled_cache.invalidate_all()
    }

    Ok(Json(crate::types::ClearModulesEnabledCacheResponse {}))
}

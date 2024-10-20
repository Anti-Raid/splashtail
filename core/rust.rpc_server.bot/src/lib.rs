pub mod settings_execute;
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
        .route(
            "/parse-permission-checks/:guild_id",
            get(parse_permission_checks),
        )
        // Dispatches a TrustedWebEvent
        .route(
            "/dispatch-trusted-web-event",
            post(dispatch_trusted_web_event),
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
        data, cache_http, ..
    }): State<AppData>,
    Path((guild_id, user_id)): Path<(serenity::all::GuildId, serenity::all::UserId)>,
    Json(req): Json<crate::types::CheckCommandPermissionRequest>,
) -> Response<crate::types::CheckCommandPermission> {
    let opts = req.opts;

    let flags = crate::types::RpcCheckCommandOptionsFlags::from_bits_truncate(opts.flags);

    let perm_res = silverpelt::cmd::check_command(
        &data.silverpelt_cache,
        &req.command,
        guild_id,
        user_id,
        &data.pool,
        &cache_http,
        &data.reqwest,
        &None,
        silverpelt::cmd::CheckCommandOptions {
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
            skip_custom_resolved_fit_checks: flags.contains(
                crate::types::RpcCheckCommandOptionsFlags::SKIP_CUSTOM_RESOLVED_FIT_CHECKS,
            ),
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

/// Verify/parse a set of permission checks returning the parsed checks [ParsePermissionChecks]
async fn parse_permission_checks(
    State(AppData {
        data,
        serenity_context,
        ..
    }): State<AppData>,
    Path(guild_id): Path<serenity::all::GuildId>,
    Json(checks): Json<permissions::types::PermissionChecks>,
) -> Response<permissions::types::PermissionChecks> {
    let parsed_checks = silverpelt::validators::parse_permission_checks(
        guild_id,
        data.pool.clone(),
        botox::cache::CacheHttpImpl::from_ctx(&serenity_context),
        data.reqwest.clone(),
        &checks,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to parse permission checks: {:#?}", e),
        )
    })?;

    Ok(Json(parsed_checks))
}

// Dispatches a TrustedWebEvent
async fn dispatch_trusted_web_event(
    State(AppData {
        data,
        serenity_context,
        ..
    }): State<AppData>,
    Json(req): Json<crate::types::DispatchTrustedWebEventRequest>,
) -> Response<crate::types::DispatchTrustedWebEventResponse> {
    silverpelt::ar_event::dispatch_event_to_modules_errflatten(Arc::new(
        silverpelt::ar_event::EventHandlerContext {
            guild_id: req
                .guild_id
                .unwrap_or(silverpelt::ar_event::SYSTEM_GUILD_ID),
            data: data.clone(),
            event: silverpelt::ar_event::AntiraidEvent::TrustedWebEvent((req.event_name, req.args)),
            serenity_context: serenity_context.clone(),
        },
    ))
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to dispatch event: {:#?}", e),
        )
    })?;

    Ok(Json(crate::types::DispatchTrustedWebEventResponse {}))
}

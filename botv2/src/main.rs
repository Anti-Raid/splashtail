mod config;
mod impls;
mod ipc;
mod jobserver;
mod modules;
mod silverpelt;
mod tasks;

use std::sync::Arc;

use log::{error, info};
use object_store::ObjectStore;
use serenity::all::HttpBuilder;
use poise::serenity_prelude::FullEvent;
use poise::CreateReply;
use sqlx::postgres::PgPoolOptions;
use std::io::Write;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// User data, which is stored and accessible in all command invocations
pub struct Data {
    pub pool: sqlx::PgPool,
    pub mewld_ipc: Arc<ipc::mewld::MewldIpcClient>,
    pub object_store: Arc<Box<dyn ObjectStore>>,
    pub animus_magic_ipc: Arc<ipc::animus_magic::client::AnimusMagicClient>,
    pub shards_ready: Arc<dashmap::DashMap<u16, bool>>,
    pub surreal_cache: Surreal<Client>,
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Command { error, ctx, .. } => {
            error!("Error in command `{}`: {:?}", ctx.command().name, error,);
            let err = ctx
                .say(format!(
                    "There was an error running this command: {}",
                    error
                ))
                .await;

            if let Err(e) = err {
                error!("SQLX Error: {}", e);
            }
        }
        poise::FrameworkError::CommandCheckFailed { error, ctx, .. } => {
            error!(
                "[Possible] error in command `{}`: {:?}",
                ctx.command().name,
                error,
            );
            if let Some(error) = error {
                error!("Error in command `{}`: {:?}", ctx.command().name, error,);
                let err = ctx.say(format!("{}", error)).await;

                if let Err(e) = err {
                    error!("Error while sending error message: {}", e);
                }
            }
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                error!("Error while handling error: {}", e);
            }
        }
    }
}

async fn event_listener<'a>(
    ctx: poise::FrameworkContext<'a, Data, Error>,
    event: &FullEvent,
) -> Result<(), Error> {
    let user_data = ctx.serenity_context.data::<Data>();
    match event {
        FullEvent::InteractionCreate { interaction } => {
            info!("Interaction received: {:?}", interaction.id());

            let ic = match interaction {
                serenity::all::Interaction::Command(ic) => ic,
                _ => return Ok(()),
            };

            if !config::CONFIG
                .discord_auth
                .can_use_bot
                .contains(&ic.user.id)
            {
                let primary = poise::serenity_prelude::CreateEmbed::default()
                    .color(0xff0000)
                    .title("AntiRaid")
                    .url(&config::CONFIG.meta.support_server)
                    .description(
                        format!("Unfortunately, AntiRaid is currently unavailable due to poor code management and changes with the Discord API. We are currently in the works of V6, and hope to have it out by next month. All use of our services will not be available, and updates will be pushed here. We are extremely sorry for the inconvenience.\nFor more information you can also join our [Support Server]({})!", config::CONFIG.meta.support_server)
                    );

                let changes = ["We are working extremely hard on Antiraid v6, and have completed working on half of the bot. We should have this update out by Q1/Q2 2024! Delays may occur due to the sheer scope of the unique features we want to provide!"];

                let updates = poise::serenity_prelude::CreateEmbed::default()
                    .color(0x0000ff)
                    .title("Updates")
                    .description(changes.join("\t-"));

                let statistics = poise::serenity_prelude::CreateEmbed::default()
                    .color(0xff0000)
                    .description(format!(
                        "**Server Count:** {}\n**Shard Count:** {}\n**Cluster Count:** {}\n**Cluster ID:** {}\n**Cluster Name:** {}\n**Uptime:** {}",
                        user_data.mewld_ipc.cache.total_guilds(),
                        ipc::argparse::MEWLD_ARGS.shard_count,
                        ipc::argparse::MEWLD_ARGS.cluster_count,
                        ipc::argparse::MEWLD_ARGS.cluster_id,
                        ipc::argparse::MEWLD_ARGS.cluster_name,
                        {
                            let duration: std::time::Duration = std::time::Duration::from_secs((chrono::Utc::now().timestamp() - crate::config::CONFIG.bot_start_time) as u64);
                            let seconds = duration.as_secs() % 60;
                            let minutes = (duration.as_secs() / 60) % 60;
                            let hours = (duration.as_secs() / 60) / 60;
                            format!("{}h{}m{}s", hours, minutes, seconds)
                        }
                    ));

                ic.create_response(
                    &ctx.serenity_context.http,
                    serenity::all::CreateInteractionResponse::Message(
                        serenity::all::CreateInteractionResponseMessage::default()
                            .flags(serenity::all::InteractionResponseFlags::EPHEMERAL)
                            .content(&config::CONFIG.meta.support_server)
                            .add_embed(primary)
                            .add_embed(updates)
                            .add_embed(statistics),
                    ),
                )
                .await
                .map_err(|e| format!("Error sending reply: {}", e))?;
            }
        }
        FullEvent::Ready { data_about_bot } => {
            info!(
                "{} is ready on shard {}",
                data_about_bot.user.name, ctx.serenity_context.shard_id
            );

            tokio::task::spawn(crate::tasks::taskcat::start_all_tasks(
                user_data.pool.clone(),
                crate::impls::cache::CacheHttpImpl {
                    cache: ctx.serenity_context.cache.clone(),
                    http: ctx.serenity_context.http.clone(),
                },
                ctx.serenity_context.clone(),
            ));

            if ctx.serenity_context.shard_id.0 == *crate::ipc::argparse::MEWLD_ARGS.shards.first().unwrap() {
                info!("Starting IPC");
                
                let data = ctx.serenity_context.data::<Data>();
                let ipc_ref = data.mewld_ipc.clone();
                let ch = crate::impls::cache::CacheHttpImpl::from_ctx(ctx.serenity_context);
                let sm = ctx.shard_manager().clone();
                tokio::task::spawn(async move {
                    let ipc_ref = ipc_ref;
                    ipc_ref.start_ipc_listener(&ch, &sm).await;
                });

                // And for animus magic
                let ch = crate::impls::cache::CacheHttpImpl::from_ctx(ctx.serenity_context);
                let sm = ctx.shard_manager().clone();
                let am_ref = data.animus_magic_ipc.clone();
                tokio::task::spawn(async move {
                    let am_ref = am_ref;
                    am_ref.start_ipc_listener(ch, sm).await;
                });
            }

            if ctx.serenity_context.shard_id.0
                == *crate::ipc::argparse::MEWLD_ARGS.shards.last().unwrap()
            {
                info!("All shards ready, launching next cluster");
                if let Err(e) = user_data.mewld_ipc.publish_ipc_launch_next().await {
                    error!("Error publishing IPC launch next: {}", e);
                    return Err(e);
                }

                user_data
                    .shards_ready
                    .insert(ctx.serenity_context.shard_id.0, true);

                info!(
                    "Published IPC launch next to channel {}",
                    crate::ipc::argparse::MEWLD_ARGS.mewld_redis_channel
                );
            }
        }
        _ => {}
    }

    // Add all event listeners for key modules here
    for (module, evts) in silverpelt::SILVERPELT_CACHE
        .module_event_listeners_cache
        .iter()
    {
        log::debug!("Executing event handlers for {}", module);
        for evth in evts.iter() {
            if let Err(e) = evth(ctx.serenity_context, event).await {
                error!("Error in event handler: {}", e);
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    const POSTGRES_MAX_CONNECTIONS: u32 = 3; // max connections to the database, we don't need too many here
    const REDIS_MAX_CONNECTIONS: u32 = 10; // max connections to the redis

    // Setup logging
    let cluster_id = ipc::argparse::MEWLD_ARGS.cluster_id;
    let cluster_name = ipc::argparse::MEWLD_ARGS.cluster_name.clone();
    let cluster_count = ipc::argparse::MEWLD_ARGS.cluster_count;
    let shards = ipc::argparse::MEWLD_ARGS.shards.clone();
    let shard_count = ipc::argparse::MEWLD_ARGS.shard_count;

    let debug_mode = std::env::var("DEBUG").unwrap_or_default() == "true";

    let mut env_builder = env_logger::builder();

    env_builder
        .format(move |buf, record| {
            writeln!(
                buf,
                "[{} ({} of {})] {} - {}",
                cluster_name,
                cluster_id,
                cluster_count - 1,
                record.level(),
                record.args()
            )
        })
        .filter(Some("botv2"), log::LevelFilter::Info);

    if debug_mode {
        env_builder.filter(None, log::LevelFilter::Debug);
    } else {
        env_builder.filter(None, log::LevelFilter::Error);
    }

    env_builder.init();

    info!("{:#?}", ipc::argparse::MEWLD_ARGS);

    let proxy_url = config::CONFIG.meta.proxy.clone();

    info!("Proxy URL: {}", proxy_url);

    let http = Arc::new(HttpBuilder::new(&config::CONFIG.discord_auth.token)
        .proxy(proxy_url)
        .ratelimiter_disabled(true)
        .build());

    info!("HttpBuilder done");

    let mut intents = serenity::all::GatewayIntents::all();

    // Remove the really spammy intents
    intents.remove(serenity::all::GatewayIntents::GUILD_PRESENCES); // Don't even have the privileged gateway intent for this
    intents.remove(serenity::all::GatewayIntents::GUILD_MESSAGE_TYPING); // Don't care about typing
    intents.remove(serenity::all::GatewayIntents::DIRECT_MESSAGE_TYPING); // Don't care about typing
    intents.remove(serenity::all::GatewayIntents::DIRECT_MESSAGES); // Don't care about DMs

    let client_builder = serenity::all::ClientBuilder::new_with_http(http, intents);

    info!("Created ClientBuilder");

    let framework_opts = poise::FrameworkOptions {
        initialize_owners: true,
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("%".into()),
            ..poise::PrefixFrameworkOptions::default()
        },
        event_handler: |ctx, event| Box::pin(event_listener(ctx, event)),
        commands: {
            let mut cmds = Vec::new();

            for module in modules::modules() {
                log::info!("Loading module {}", module.id);
                for cmd in module.commands {
                    let mut cmd = cmd.0;
                    cmd.category = Some(module.id.to_string());
                    cmds.push(cmd);
                }
            }

            cmds
        },
        command_check: Some(|ctx| {
            Box::pin(async move {
                if !config::CONFIG
                    .discord_auth
                    .can_use_bot
                    .contains(&ctx.author().id)
                {
                    // We already send in the event handler
                    if let poise::Context::Application(_) = ctx {
                        return Ok(false);
                    }

                    let data = ctx.data();
                    let primary = poise::serenity_prelude::CreateEmbed::default()
                        .color(0xff0000)
                        .title("AntiRaid")
                        .url(&config::CONFIG.meta.support_server)
                        .description(
                            format!(
                                "Unfortunately, AntiRaid is currently unavailable due to poor code management and changes with the Discord API. We are currently in the works of V6, and hope to have it out by next month. All use of our services will not be available, and updates will be pushed here. We are extremely sorry for the inconvenience.\nFor more information you can also join our [Support Server]({})!", config::CONFIG.meta.support_server
                            )
                        );

                    let changes = ["We are working extremely hard on Antiraid v6, and have completed working on half of the bot. We should have this update out by Q1/Q2 2024! Delays may occur due to the sheer scope of the unique features we want to provide!"];

                    let updates = poise::serenity_prelude::CreateEmbed::default()
                        .color(0x0000ff)
                        .title("Updates")
                        .description(changes.join("\t-"));

                    let statistics = poise::serenity_prelude::CreateEmbed::default()
                        .color(0xff0000)
                        .description(format!(
                            "**Server Count:** {}\n**Shard Count:** {}\n**Cluster Count:** {}\n**Cluster ID:** {}\n**Cluster Name:** {}\n**Uptime:** {}",
                            data.mewld_ipc.cache.total_guilds(),
                            ipc::argparse::MEWLD_ARGS.shard_count,
                            ipc::argparse::MEWLD_ARGS.cluster_count,
                            ipc::argparse::MEWLD_ARGS.cluster_id,
                            ipc::argparse::MEWLD_ARGS.cluster_name,
                            {
                                let duration: std::time::Duration = std::time::Duration::from_secs((chrono::Utc::now().timestamp() - crate::config::CONFIG.bot_start_time) as u64);
                                let seconds = duration.as_secs() % 60;
                                let minutes = (duration.as_secs() / 60) % 60;
                                let hours = (duration.as_secs() / 60) / 60;
                                format!("{}h{}m{}s", hours, minutes, seconds)
                            }
                        ));
                    ctx.send(
                        CreateReply::default()
                            .content(&config::CONFIG.meta.support_server)
                            .embed(primary)
                            .embed(updates)
                            .embed(statistics),
                    )
                    .await
                    .map_err(|e| format!("Error sending reply: {}", e))?;

                    return Ok(false);
                }

                let command = ctx.command();

                // Check COMMAND_ID_MODULE_MAP
                if !silverpelt::SILVERPELT_CACHE
                    .command_id_module_map
                    .contains_key(&command.name)
                {
                    return Err(
                        "This command is not registered in the database, please contact support"
                            .into(),
                    );
                }

                let module = silverpelt::SILVERPELT_CACHE
                    .command_id_module_map
                    .get(&command.name)
                    .unwrap();

                if module == "root" {
                    if !crate::config::CONFIG
                        .discord_auth
                        .root_users
                        .contains(&ctx.author().id)
                    {
                        return Err("Root commands are off-limits unless you are a bot owner or otherwise have been granted authorization!".into());
                    }
                    return Ok(true);
                }

                // Look for guild
                if let Some(guild_id) = ctx.guild_id() {
                    if ["register"].contains(&ctx.command().name.as_str()) {
                        return Ok(true);
                    }

                    let data = ctx.data();

                    let guild = sqlx::query!(
                        "SELECT COUNT(*) FROM guilds WHERE id = $1",
                        guild_id.to_string()
                    )
                    .fetch_one(&data.pool)
                    .await?;

                    if guild.count.unwrap_or_default() == 0 {
                        // Guild not found, create it
                        sqlx::query!("INSERT INTO guilds (id) VALUES ($1)", guild_id.to_string())
                            .execute(&data.pool)
                            .await?;
                    }

                    let key = silverpelt::SILVERPELT_CACHE
                        .command_permission_cache
                        .get(&(guild_id, ctx.author().id))
                        .await;

                    if let Some(ref map) = key {
                        let cpr = map.get(&ctx.command().qualified_name);

                        if let Some(cpr) = cpr {
                            match cpr {
                                silverpelt::CachedPermResult::Ok => return Ok(true),
                                silverpelt::CachedPermResult::Err(e) => {
                                    return Err(e.to_string().into())
                                }
                            }
                        }
                    }

                    let Some(member) = ctx.author_member().await else {
                        return Err("You must be in a server to run this command".into());
                    };

                    let (cmd_data, command_config, module_config) =
                        silverpelt::get_command_configuration(
                            &data.pool,
                            guild_id.to_string().as_str(),
                            ctx.command().qualified_name.as_str(),
                        )
                        .await?;

                    let command_config = command_config.unwrap_or(silverpelt::GuildCommandConfiguration {
                        id: "".to_string(),
                        guild_id: guild_id.to_string(),
                        command: ctx.command().qualified_name.clone(),
                        perms: None,
                        disabled: false,
                    });

                    let module_config = module_config.unwrap_or(silverpelt::GuildModuleConfiguration {
                        id: "".to_string(),
                        guild_id: guild_id.to_string(),
                        module: module.clone(),
                        disabled: None,
                    });

                    let (is_owner, member_perms) = if let Some(guild) = ctx.guild() {
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
                    } else {
                        return Err("Your guild has not been cached yet? Please contact support after trying again as this should NEVER happen!".into());
                    };

                    if is_owner {
                        return Ok(true);
                    }

                    // Get kittycat perms of member (if they have any)
                    let kittycat_perms = {
                        let rec = sqlx::query!("SELECT resolved_perms_cache FROM guild_members WHERE guild_id = $1 AND user_id = $2", guild_id.to_string(), member.user.id.to_string())
                        .fetch_optional(&data.pool)
                        .await?;

                        if let Some(rec) = rec {
                            rec.resolved_perms_cache
                        } else {
                            Vec::new()
                        }
                    };

                    info!(
                        "Checking if user {} ({}) can run command {} with permissions {:?}",
                        member.user.name,
                        member.user.id,
                        ctx.command().qualified_name,
                        member_perms
                    );
                    if let Err(e) = silverpelt::permissions::can_run_command(
                        &cmd_data,
                        &command_config,
                        &module_config,
                        &ctx.command().qualified_name,
                        member_perms,
                        &kittycat_perms,
                    ) {
                        return Err(format!("{}\n\n**Code**: {}", e.1, e.0).into());
                    }

                    let mut key = silverpelt::SILVERPELT_CACHE
                        .command_permission_cache
                        .get(&(guild_id, ctx.author().id))
                        .await;
                    if let Some(ref mut map) = key {
                        map.insert(
                            ctx.command().qualified_name.clone(),
                            silverpelt::CachedPermResult::Ok,
                        );
                    } else {
                        let mut map = indexmap::IndexMap::new();
                        map.insert(
                            ctx.command().qualified_name.clone(),
                            silverpelt::CachedPermResult::Ok,
                        );
                        silverpelt::SILVERPELT_CACHE
                            .command_permission_cache
                            .insert((guild_id, ctx.author().id), map)
                            .await;
                    }

                    Ok(true)
                } else {
                    Err("This command can only be run from servers".into())
                }
            })
        }),
        pre_command: |ctx| {
            Box::pin(async move {
                info!(
                    "Executing command {} for user {} ({})...",
                    ctx.command().qualified_name,
                    ctx.author().name,
                    ctx.author().id
                );
            })
        },
        post_command: |ctx| {
            Box::pin(async move {
                info!(
                    "Done executing command {} for user {} ({})...",
                    ctx.command().qualified_name,
                    ctx.author().name,
                    ctx.author().id
                );
            })
        },
        on_error: |error| Box::pin(on_error(error)),
        ..Default::default()
    };

    let framework = poise::Framework::builder()
        .options(framework_opts)
        .build();

    info!("Connecting to redis");

    let pool = fred::prelude::Builder::from_config(
        fred::prelude::RedisConfig::from_url(&config::CONFIG.meta.bot_redis_url)
            .expect("Could not initialize Redis config"),
    )
    .build_pool(REDIS_MAX_CONNECTIONS.try_into().unwrap())
    .expect("Could not initialize Redis pool");

    info!("Connecting to surreal");

    let surreal_config = config::CONFIG.surreal.clone();
    let surreal_client = Surreal::new::<Ws>(surreal_config.url)
        .await
        .expect("Couldnt initialize surreal");
    let _ = surreal_client
        .signin(Root {
            username: surreal_config.username.as_str(),
            password: surreal_config.password.as_str(),
        })
        .await;
    surreal_client
        .use_ns("antiraid")
        .use_db("splashtail")
        .await
        .expect("Couldnt use namespace and database");

    let data = Data {
        mewld_ipc: Arc::new(ipc::mewld::MewldIpcClient {
            redis_pool: pool.clone(),
            cache: Arc::new(ipc::mewld::MewldIpcCache::default()),
        }),
        animus_magic_ipc: Arc::new(ipc::animus_magic::client::AnimusMagicClient {
            redis_pool: pool.clone(),
            rx_map: Arc::new(dashmap::DashMap::new()),
        }),
        object_store: Arc::new(
            config::CONFIG
                .object_storage
                .build()
                .expect("Could not initialize object store"),
        ),
        pool: PgPoolOptions::new()
            .max_connections(POSTGRES_MAX_CONNECTIONS)
            .connect(&config::CONFIG.meta.postgres_url)
            .await
            .expect("Could not initialize connection"),
        shards_ready: Arc::new(dashmap::DashMap::new()),
        surreal_cache: surreal_client,
    };

    info!("Initializing bot state");

    let mut client = client_builder
        .framework(framework)
        .data(Arc::new(data))
        .await
        .expect("Error creating client");

    let shard_range = std::ops::Range {
        start: shards[0],
        end: *shards.last().unwrap(),
    };

    info!("Starting shard range: {:?}", shard_range);

    if let Err(why) = client.start_shard_range(shard_range, shard_count).await {
        error!("Client error: {:?}", why);
    }

    std::process::exit(1); // Clean exit with status code of 1
}

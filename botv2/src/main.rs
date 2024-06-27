mod config;
mod ipc;
mod modules;
mod silverpelt;

use ipc::{
    animus_magic::client::{AnimusMagicClient, ClientData},
    mewld::MewldIpcClient,
};

use botox::cache::CacheHttpImpl;
use gwevent::core::get_event_guild_id;
use silverpelt::{
    module_config::is_module_enabled, proxysupport::ProxySupportData,
    silverpelt_cache::SILVERPELT_CACHE, EventHandlerContext,
};
use splashcore_rs::objectstore::ObjectStore;

use once_cell::sync::Lazy;
use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;

use log::{error, info, warn};
use serenity::all::{FullEvent, GuildId, HttpBuilder, UserId};
use sqlx::postgres::PgPoolOptions;
use std::io::Write;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub struct ConnectState {
    pub has_started_bgtasks: bool,
    pub has_started_ipc: bool,
    pub have_called_firstready: bool,
}

pub static CONNECT_STATE: Lazy<RwLock<ConnectState>> = Lazy::new(|| {
    RwLock::new(ConnectState {
        has_started_bgtasks: false,
        has_started_ipc: false,
        have_called_firstready: false,
    })
});

pub struct CanUseBotList {
    pub users: Vec<UserId>,
    pub guilds: Vec<GuildId>,
}

pub static CAN_USE_BOT_CACHE: Lazy<RwLock<CanUseBotList>> = Lazy::new(|| {
    RwLock::new(CanUseBotList {
        users: Vec::new(),
        guilds: Vec::new(),
    })
});

/// User data, which is stored and accessible in all command invocations
pub struct Data {
    pub pool: sqlx::PgPool,
    pub redis_pool: fred::prelude::RedisPool,
    pub reqwest: reqwest::Client,
    pub mewld_ipc: Arc<MewldIpcClient>,
    pub animus_magic_ipc: OnceLock<Arc<AnimusMagicClient>>, // a rwlock is needed as the cachehttp is only available after the client is started
    pub object_store: Arc<ObjectStore>,
    pub shards_ready: Arc<dashmap::DashMap<u16, bool>>,
    pub proxy_support_data: RwLock<Option<ProxySupportData>>, // Shard ID, WebsocketConfiguration
}

impl Data {
    /// Helper method to get the animus magic client
    fn get_animus_magic(&self) -> Result<Arc<AnimusMagicClient>, crate::Error> {
        let am = self.animus_magic_ipc.get();

        match am {
            Some(am) => Ok(am.clone()),
            None => Err("Animus Magic IPC not initialized".into()),
        }
    }
}

// TODO: allow root users to customize/set this in database later
pub fn maint_message<'a>(user_data: &crate::Data) -> poise::CreateReply<'a> {
    let primary = poise::serenity_prelude::CreateEmbed::default()
    .color(0xff0000)
    .title("AntiRaid")
    .url(&config::CONFIG.meta.support_server)
    .description(
        format!("Unfortunately, AntiRaid is currently unavailable due to poor code management and changes with the Discord API. We are currently in the works of V6, and hope to have it out by next month. All use of our services will not be available, and updates will be pushed here. We are extremely sorry for the inconvenience.\nFor more information you can also join our [Support Server]({})!", config::CONFIG.meta.support_server)
    );

    let changes = [
        "We are working extremely hard on Antiraid v6, and have completed working on half of the bot. We should have this update out by Q1/Q2 2024! Delays may occur due to the sheer scope of the unique features we want to provide!",
        "Yet another update: we are in the process of adding some MASSIVE new features including advanced permission management, server member limits, AI image classification, server member backups and custom customizable github webhook support (for developers)"
    ];

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

    poise::CreateReply::new()
        .ephemeral(true)
        .content(&config::CONFIG.meta.support_server)
        .embed(primary)
        .embed(updates)
        .embed(statistics)
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Command { error, ctx, .. } => {
            error!("Error in command `{}`: {:?}", ctx.command().name, error,);

            let err = ctx
                .send(
                    poise::CreateReply::new().embed(
                        serenity::all::CreateEmbed::new()
                            .color(serenity::all::Color::RED)
                            .title("An error has occurred")
                            .description(error.to_string()),
                    ),
                )
                .await;

            if let Err(e) = err {
                error!("Message send error for FrameworkError::Command: {}", e);
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
                    error!(
                        "Message send error for FrameworkError::CommandCheckFailed: {}",
                        e
                    );
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

            let allowed = config::CONFIG.discord_auth.public_bot || {
                let cub_cache = CAN_USE_BOT_CACHE.read().await;
                if let Some(ref guild_id) = ic.guild_id {
                    cub_cache.guilds.contains(guild_id) && cub_cache.users.contains(&ic.user.id)
                } else {
                    cub_cache.users.contains(&ic.user.id)
                }
            };

            if !allowed {
                ic.create_response(
                    &ctx.serenity_context.http,
                    serenity::all::CreateInteractionResponse::Message(
                        maint_message(&user_data).to_slash_initial_response(
                            serenity::all::CreateInteractionResponseMessage::default(),
                        ),
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

            if ctx.serenity_context.shard_id.0
                == *crate::ipc::argparse::MEWLD_ARGS.shards.first().unwrap()
            {
                if !CONNECT_STATE.read().await.has_started_bgtasks {
                    info!("Starting background tasks");
                    // Get all tasks
                    let mut tasks = Vec::new();
                    for module in modules::modules() {
                        for task in module.background_tasks {
                            tasks.push(task);
                        }
                    }

                    tokio::task::spawn(botox::taskman::start_all_tasks(
                        tasks,
                        ctx.serenity_context.clone(),
                    ));
                }

                CONNECT_STATE.write().await.has_started_bgtasks = true;

                if !CONNECT_STATE.read().await.has_started_ipc {
                    info!("Starting IPC");

                    let data = ctx.serenity_context.data::<Data>();
                    let ipc_ref = data.mewld_ipc.clone();
                    let ch = CacheHttpImpl::from_ctx(ctx.serenity_context);
                    let sm = ctx.shard_manager().clone();
                    tokio::task::spawn(async move {
                        let ipc_ref = ipc_ref;
                        ipc_ref.start_ipc_listener(&ch, &sm).await;
                    });

                    // And for animus magic
                    let am = Arc::new(AnimusMagicClient::new(Arc::new(ClientData {
                        pool: data.pool.clone(),
                        redis_pool: data.redis_pool.clone(),
                        reqwest: data.reqwest.clone(),
                        cache_http: CacheHttpImpl::from_ctx(ctx.serenity_context),
                    })));

                    data.animus_magic_ipc.get_or_init(|| am.clone());

                    tokio::task::spawn(async move {
                        let am_ref = am.clone();
                        am_ref.listen().await;
                    });
                }

                CONNECT_STATE.write().await.has_started_ipc = true;
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

            if !CONNECT_STATE.read().await.have_called_firstready {
                for module in modules::modules() {
                    for on_ready in module.on_first_ready.iter() {
                        if let Err(e) = on_ready(ctx.serenity_context.clone(), &user_data).await {
                            error!("Error initializing module [on_first_ready]: {}", e);
                            panic!(
                                "CRITICAL: Error initializing module [on_first_ready]: {}",
                                e
                            );
                        }
                    }
                }

                CONNECT_STATE.write().await.have_called_firstready = true;
            }
        }
        _ => {}
    }

    // Add all event listeners for key modules here
    let event_guild_id = match get_event_guild_id(event) {
        Ok(guild_id) => guild_id,
        Err(None) => return Ok(()),
        Err(Some(e)) => {
            warn!("Error getting guild id for event: {}", e);
            return Err(e);
        }
    };

    // Check if whitelisted
    let allowed = config::CONFIG.discord_auth.public_bot || {
        let cub = CAN_USE_BOT_CACHE.read().await;
        cub.guilds.contains(&event_guild_id)
    };

    if !allowed {
        return Ok(()); // Ignore the event
    }

    // Create context for event handlers, this is done here and wrapped in an Arc to avoid useless clones
    let event_handler_context = Arc::new(EventHandlerContext {
        guild_id: event_guild_id,
        full_event: event.clone(),
        data: ctx.user_data(),
        serenity_context: ctx.serenity_context.clone(),
    });

    let mut set = tokio::task::JoinSet::new();
    for (module, evts) in SILVERPELT_CACHE.module_event_listeners_cache.iter() {
        let module_enabled =
            match is_module_enabled(&event_handler_context.data.pool, event_guild_id, module).await
            {
                Ok(enabled) => enabled,
                Err(e) => {
                    error!("Error getting module enabled status: {}", e);
                    continue;
                }
            };

        if !module_enabled {
            continue;
        }

        log::trace!("Executing event handlers for {}", module);

        for evth in evts.iter() {
            let event_handler_context = event_handler_context.clone();
            set.spawn(async move { evth(&event_handler_context).await });
        }
    }

    while let Some(res) = set.join_next().await {
        match res {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => {
                error!("Error in event handler [task]: {}", e);
            }
            Err(e) => {
                error!("Error in event handler [joinset]: {}", e);
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
    let debug_opts = std::env::var("DEBUG_OPTS").unwrap_or_default();

    let mut env_builder = env_logger::builder();

    env_builder
        .format(move |buf, record| {
            writeln!(
                buf,
                "[{} ({} of {})] ({}) {} - {}",
                cluster_name,
                cluster_id,
                cluster_count - 1,
                record.target(),
                record.level(),
                record.args()
            )
        })
        .filter(Some("botv2"), log::LevelFilter::Info)
        .filter(Some("botox"), log::LevelFilter::Info)
        .filter(Some("splashcore_rs"), log::LevelFilter::Info);

    // Set custom log levels
    for opt in debug_opts.split(',') {
        let opt = opt.trim();

        if opt.is_empty() {
            continue;
        }

        let (target, level) = if opt.contains('=') {
            let mut split = opt.split('=');
            let target = split.next().unwrap();
            let level = split.next().unwrap();
            (target, level)
        } else {
            (opt, "debug")
        };

        let level = match level {
            "trace" => log::LevelFilter::Trace,
            "debug" => log::LevelFilter::Debug,
            "info" => log::LevelFilter::Info,
            "warn" => log::LevelFilter::Warn,
            "error" => log::LevelFilter::Error,
            _ => {
                error!("Invalid log level: {}", level);
                continue;
            }
        };

        env_builder.filter(Some(target), level);
    }

    if debug_mode {
        env_builder.filter(None, log::LevelFilter::Debug);
    } else {
        env_builder.filter(None, log::LevelFilter::Error);
    }

    env_builder.init();

    info!("{:#?}", ipc::argparse::MEWLD_ARGS);

    let proxy_url = config::CONFIG.meta.proxy.get().clone();

    info!("Proxy URL: {}", proxy_url);

    let http = Arc::new(
        HttpBuilder::new(&config::CONFIG.discord_auth.token)
            .proxy(proxy_url)
            .ratelimiter_disabled(true)
            .build(),
    );

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

            let mut _cmd_names = Vec::new();
            for module in modules::modules() {
                log::info!("Loading module {}", module.id);

                if !module.is_parsed() {
                    panic!("Module {} is not parsed", module.id);
                }

                if module.virtual_module {
                    continue;
                }

                for (mut cmd, extended_data) in module.commands {
                    let root_is_virtual = match extended_data.get("") {
                        Some(root) => root.virtual_command,
                        None => false,
                    };

                    if root_is_virtual {
                        continue;
                    }

                    cmd.category = Some(module.id.to_string());

                    let mut subcommands = Vec::new();
                    // Ensure subcommands are also linked to a category
                    for subcommand in cmd.subcommands {
                        let ext_data =
                            extended_data
                                .get(subcommand.name.as_str())
                                .unwrap_or_else(|| {
                                    panic!(
                                        "Subcommand {} does not have extended data",
                                        subcommand.name
                                    )
                                });

                        if ext_data.virtual_command {
                            continue;
                        }

                        subcommands.push(poise::Command {
                            category: Some(module.id.to_string()),
                            ..subcommand
                        });
                    }

                    cmd.subcommands = subcommands;

                    // Check for duplicate command names
                    if _cmd_names.contains(&cmd.name) {
                        error!("Duplicate command name: {:#?}", cmd);
                        panic!("Duplicate command name: {}", cmd.qualified_name);
                    }

                    _cmd_names.push(cmd.name.clone());

                    // Check for duplicate command aliases
                    for alias in cmd.aliases.iter() {
                        if _cmd_names.contains(alias) {
                            panic!(
                                "Duplicate command alias: {} from command {}",
                                alias, cmd.name
                            );
                        }

                        _cmd_names.push(alias.clone());
                    }

                    // Good to go
                    cmds.push(cmd);
                }
            }

            cmds
        },
        command_check: Some(|ctx| {
            Box::pin(async move {
                let allowed = config::CONFIG.discord_auth.public_bot || {
                    let cub_cache = CAN_USE_BOT_CACHE.read().await;
                    if let Some(ref guild_id) = ctx.guild_id() {
                        cub_cache.guilds.contains(guild_id)
                            && cub_cache.users.contains(&ctx.author().id)
                    } else {
                        cub_cache.users.contains(&ctx.author().id)
                    }
                };

                if !allowed {
                    // We already send in the event handler
                    if let poise::Context::Application(_) = ctx {
                        return Ok(false);
                    }

                    ctx.send(maint_message(&ctx.data()))
                        .await
                        .map_err(|e| format!("Error sending reply: {}", e))?;

                    return Ok(false);
                }

                let Some(guild_id) = ctx.guild_id() else {
                    return Err("This command can only be run from servers".into());
                };

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

                let command = ctx.command();

                let res = silverpelt::cmd::check_command(
                    command.name.as_str(),
                    &command.qualified_name,
                    guild_id,
                    ctx.author().id,
                    &data.pool,
                    &CacheHttpImpl::from_ctx(ctx.serenity_context()),
                    &Some(ctx),
                    silverpelt::cmd::CheckCommandOptions::default(),
                )
                .await;

                if res.is_ok() {
                    return Ok(true);
                }

                ctx.send(
                    poise::CreateReply::new().embed(
                        serenity::all::CreateEmbed::new()
                            .color(serenity::all::Color::RED)
                            .title("You don't have permission to use this command?")
                            .description(res.to_markdown())
                            .field("Code", format!("`{}`", res.code()), false),
                    ),
                )
                .await?;

                Ok(false)
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

    let framework = poise::Framework::builder().options(framework_opts).build();

    info!("Connecting to redis");

    let pool = fred::prelude::Builder::from_config(
        fred::prelude::RedisConfig::from_url(&config::CONFIG.meta.bot_redis_url)
            .expect("Could not initialize Redis config"),
    )
    .build_pool(REDIS_MAX_CONNECTIONS.try_into().unwrap())
    .expect("Could not initialize Redis pool");

    let pg_pool = PgPoolOptions::new()
        .max_connections(POSTGRES_MAX_CONNECTIONS)
        .connect(&config::CONFIG.meta.postgres_url)
        .await
        .expect("Could not initialize connection");

    // Fetch can_use_bot list
    let rec = sqlx::query!("SELECT id, type FROM can_use_bot")
        .fetch_all(&pg_pool)
        .await
        .expect("Could not fetch the users who are allowed to use the bot");

    // Save to CAN_USE_BOT_CACHE
    let mut cub = CAN_USE_BOT_CACHE.write().await;
    for item in rec {
        match item.r#type.as_str() {
            "user" => {
                let id = item
                    .id
                    .parse::<UserId>()
                    .unwrap_or_else(|_| panic!("Failed to parse user id: {}", item.id));
                cub.users.push(id);
            }
            "guild" => {
                let id = item
                    .id
                    .parse::<GuildId>()
                    .unwrap_or_else(|_| panic!("Failed to parse guild id: {}", item.id));
                cub.guilds.push(id);
            }
            _ => panic!("Unsupported type: {}", item.r#type),
        }
    }

    for root_user in config::CONFIG.discord_auth.root_users.iter() {
        cub.users.push(*root_user);
    }

    drop(cub);

    let reqwest = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(30))
        .timeout(std::time::Duration::from_secs(90))
        .build()
        .expect("Could not initialize reqwest client");

    let data = Data {
        mewld_ipc: Arc::new(ipc::mewld::MewldIpcClient {
            redis_pool: pool.clone(),
            cache: Arc::new(ipc::mewld::MewldIpcCache::default()),
            pool: pg_pool.clone(),
        }),
        redis_pool: pool.clone(),
        animus_magic_ipc: OnceLock::new(),
        object_store: Arc::new(
            config::CONFIG
                .object_storage
                .build()
                .expect("Could not initialize object store"),
        ),
        pool: pg_pool,
        reqwest,
        shards_ready: Arc::new(dashmap::DashMap::new()),
        proxy_support_data: RwLock::new(None),
    };

    info!("Initializing bot state");

    for module in modules::modules() {
        for init in module.on_startup.iter() {
            if let Err(e) = init(&data).await {
                error!("Error initializing module: {}", e);
                panic!("CRITICAL: Error initializing module: {}", e);
            }
        }
    }

    let mut client = client_builder
        .framework(framework)
        .data(Arc::new(data))
        .await
        .expect("Error creating client");

    client.cache.set_max_messages(10000);

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

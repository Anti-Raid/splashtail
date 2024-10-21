mod ipc;

use botox::cache::CacheHttpImpl;
use gwevent::core::get_event_guild_id;
use silverpelt::ar_event::{AntiraidEvent, EventHandlerContext};

use std::sync::{Arc, LazyLock};
use tokio::sync::RwLock;

use cap::Cap;
use clap::Parser;
use log::{error, info, warn};
use serenity::all::{FullEvent, HttpBuilder};
use silverpelt::{data::Data, Error};
use sqlx::postgres::PgPoolOptions;
use std::alloc;
use std::io::Write;

pub fn modules() -> Vec<Box<dyn silverpelt::module::Module>> {
    bot_modules_default::modules()
}

#[global_allocator]
static ALLOCATOR: Cap<alloc::System> = Cap::new(alloc::System, usize::MAX);

pub struct ConnectState {
    pub started_tasks: std::sync::atomic::AtomicBool,
    pub ready: dashmap::DashMap<serenity::all::ShardId, bool>,
    pub ready_lock: tokio::sync::Mutex<()>,
}

pub static CONNECT_STATE: LazyLock<ConnectState> = LazyLock::new(|| ConnectState {
    started_tasks: std::sync::atomic::AtomicBool::new(false),
    ready: dashmap::DashMap::new(),
    ready_lock: tokio::sync::Mutex::new(()),
});

/// Props
pub struct Props {
    pub pool: sqlx::PgPool,
    pub cmd_args: Arc<ipc::argparse::CmdArgs>,
    pub cache_http: Arc<RwLock<Option<CacheHttpImpl>>>,
    pub shard_manager: Arc<RwLock<Option<Arc<serenity::all::ShardManager>>>>,
}

#[async_trait::async_trait]
impl silverpelt::data::Props for Props {
    /// Converts the props to std::any::Any
    fn as_any(&self) -> &(dyn std::any::Any + Send + Sync) {
        self
    }

    fn extra_description(&self) -> String {
        "...".to_string()
    }

    async fn shards(&self) -> Result<Vec<u16>, Error> {
        if let Some(ref shards) = self.cmd_args.shards {
            return Ok(shards.clone());
        };

        let guard = self.shard_manager.read().await;

        if let Some(shard_manager) = guard.as_ref() {
            let mut shards = Vec::new();

            for (id, _) in shard_manager.runners.lock().await.iter() {
                shards.push(id.0);
            }

            Ok(shards)
        } else {
            Ok(Vec::new())
        }
    }

    async fn shard_count(&self) -> Result<u16, Error> {
        let guard = self.cache_http.read().await;

        if let Some(cache_http) = guard.as_ref() {
            Ok(cache_http.cache.shard_count().get())
        } else {
            if let Some(ref shards) = self.cmd_args.shards {
                return Ok(shards.len() as u16);
            }

            Ok(1)
        }
    }

    async fn total_guilds(&self) -> Result<u64, Error> {
        let guard = self.cache_http.read().await;

        if let Some(cache_http) = guard.as_ref() {
            Ok(cache_http.cache.guilds().len() as u64)
        } else {
            Ok(0)
        }
    }

    async fn total_users(&self) -> Result<u64, Error> {
        let guard = self.cache_http.read().await;

        if let Some(cache_http) = guard.as_ref() {
            let mut count = 0;

            for guild in cache_http.cache.guilds() {
                {
                    let guild = guild.to_guild_cached(&cache_http.cache);

                    if let Some(guild) = guild {
                        count += guild.member_count;
                    }
                }

                tokio::task::yield_now().await;
            }

            Ok(count)
        } else {
            Ok(0)
        }
    }
}

async fn event_listener<'a>(
    ctx: poise::FrameworkContext<'a, Data, Error>,
    event: &FullEvent,
) -> Result<(), Error> {
    match event {
        FullEvent::InteractionCreate { interaction } => {
            if !CONNECT_STATE
                .ready
                .contains_key(&ctx.serenity_context.shard_id)
            {
                return Ok(()); // Ignore interactions if the bot is not ready
            }

            info!("Interaction received: {:?}", interaction.id());
        }
        FullEvent::Ready { data_about_bot } => {
            let _lock = CONNECT_STATE.ready_lock.lock().await; // Lock to ensure that we don't have multiple ready events at the same time

            info!(
                "{} is ready on shard {}",
                data_about_bot.user.name, ctx.serenity_context.shard_id
            );

            // Set props
            let data = ctx.serenity_context.data::<Data>();
            let props = data.props.as_any().downcast_ref::<Props>().unwrap();

            let cache_http = CacheHttpImpl::from_ctx(ctx.serenity_context);
            let mut guard = props.cache_http.write().await;
            *guard = Some(cache_http);
            drop(guard);

            let shard_manager = ctx.shard_manager.clone();
            let mut guard = props.shard_manager.write().await;
            *guard = Some(shard_manager);
            drop(guard);

            // We don't really care which shard runs this, we just need one to run it
            if !CONNECT_STATE
                .started_tasks
                .load(std::sync::atomic::Ordering::SeqCst)
            {
                info!("Starting background tasks");
                // Get all tasks
                let tasks = bot_binutils::get_tasks(ctx.serenity_context, &data);
                tokio::task::spawn(botox::taskman::start_all_tasks(
                    tasks,
                    ctx.serenity_context.clone(),
                ));

                info!("Starting IPC");
                let data = ctx.serenity_context.data::<Data>();

                // Create a new rpc server
                let rpc_server =
                    rust_rpc_server_bot::create_bot_rpc_server(data.clone(), ctx.serenity_context);

                // Start the rpc server
                tokio::task::spawn(async move {
                    log::info!("Starting RPC server");
                    let opts = rust_rpc_server::CreateRpcServerOptions {
                        bind: rust_rpc_server::CreateRpcServerBind::Address(format!(
                            "{}:{}",
                            config::CONFIG.base_ports.bot_bind_addr,
                            config::CONFIG.base_ports.bot
                        )),
                    };

                    match rust_rpc_server::start_rpc_server(opts, rpc_server).await {
                        Ok(_) => {}
                        Err(e) => {
                            error!("Error starting RPC server: {}", e);
                            std::process::exit(1);
                        }
                    }
                });

                CONNECT_STATE
                    .started_tasks
                    .store(true, std::sync::atomic::Ordering::SeqCst);
            }

            CONNECT_STATE
                .ready
                .insert(ctx.serenity_context.shard_id, true);

            drop(_lock);
        }
        _ => {}
    }

    // Ignore all other events if the bot is not ready
    if !CONNECT_STATE
        .ready
        .contains_key(&ctx.serenity_context.shard_id)
    {
        return Ok(());
    }

    // Get guild id
    let event_guild_id = match get_event_guild_id(event) {
        Ok(guild_id) => guild_id,
        Err(None) => return Ok(()),
        Err(Some(e)) => {
            warn!("Error getting guild id for event: {}", e);
            return Err(e);
        }
    };

    // Create context for event handlers, this is done here and wrapped in an Arc to avoid useless clones
    let event_handler_context = Arc::new(EventHandlerContext {
        guild_id: event_guild_id,
        data: ctx.user_data(),
        event: AntiraidEvent::Discord(event.clone()),
        serenity_context: ctx.serenity_context.clone(),
    });

    if let Err(e) = silverpelt::ar_event::dispatch_event_to_modules(event_handler_context).await {
        error!(
            "Error dispatching event to modules: {}",
            e.into_iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    };

    Ok(())
}

#[tokio::main]
async fn main() {
    // Initially set allocator limit to 5GB, while this is quite high, it does ensure that the bot doesn't go down during normal operation
    ALLOCATOR.set_limit(5 * 1024 * 1024 * 1024).unwrap();

    const POSTGRES_MAX_CONNECTIONS: u32 = 70; // max connections to the database, we don't need too many here

    // Setup logging
    let cmd_args = Arc::new(ipc::argparse::CmdArgs::parse());

    let debug_mode = std::env::var("DEBUG").unwrap_or_default() == "true";
    let debug_opts = std::env::var("DEBUG_OPTS").unwrap_or_default();

    let mut env_builder = env_logger::builder();

    let mut default_filter =
        "serenity=error,fred=error,rust_bot=info,bot_binutils=info,rust_rpc_server=info,rust_rpc_server_bot=info,botox=info,templating=debug,sqlx=error".to_string();

    for module in modules() {
        let module_id = module.id();
        let module_filter = format!("bot_modules_{}=info", module_id);
        default_filter.push(',');
        default_filter.push_str(module_filter.as_str());
    }

    env_builder
        .format(move |buf, record| {
            writeln!(
                buf,
                "({}) {} - {}",
                record.target(),
                record.level(),
                record.args()
            )
        })
        .parse_filters(&default_filter)
        .filter(None, log::LevelFilter::Info);

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

    info!("{:#?}", cmd_args);

    let proxy_url = config::CONFIG.meta.proxy.clone();

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

    info!("Creating SilverpeltCache");

    let silverpelt_cache = {
        let mut silverpelt_cache = silverpelt::cache::SilverpeltCache::default();

        for module in modules() {
            silverpelt_cache.add_module(module);
        }

        Arc::new(silverpelt_cache)
    };

    let framework_opts = poise::FrameworkOptions {
        initialize_owners: true,
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("%".into()),
            ..poise::PrefixFrameworkOptions::default()
        },
        event_handler: |ctx, event| Box::pin(event_listener(ctx, event)),
        commands: bot_binutils::get_commands(&silverpelt_cache),
        command_check: Some(|ctx| Box::pin(bot_binutils::command_check(ctx))),
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
        on_error: |error| Box::pin(bot_binutils::on_error(error)),
        ..Default::default()
    };

    let framework = poise::Framework::builder().options(framework_opts).build();

    info!("Connecting to database");

    let pg_pool = PgPoolOptions::new()
        .max_connections(POSTGRES_MAX_CONNECTIONS)
        .connect(&config::CONFIG.meta.postgres_url)
        .await
        .expect("Could not initialize connection");

    let reqwest = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(30))
        .timeout(std::time::Duration::from_secs(90))
        .build()
        .expect("Could not initialize reqwest client");

    let props = Arc::new(Props {
        pool: pg_pool.clone(),
        cmd_args: cmd_args.clone(),
        cache_http: Arc::new(RwLock::new(None)),
        shard_manager: Arc::new(RwLock::new(None)),
    });

    let data = Data {
        object_store: Arc::new(
            config::CONFIG
                .object_storage
                .build()
                .expect("Could not initialize object store"),
        ),
        pool: pg_pool.clone(),
        reqwest,
        extra_data: dashmap::DashMap::new(),
        props: props.clone(),
        silverpelt_cache,
    };

    let mut client = client_builder
        .framework(framework)
        .data(Arc::new(data))
        .await
        .expect("Error creating client");

    client.cache.set_max_messages(10000);

    if let Some(shard_count) = cmd_args.shard_count {
        if let Some(ref shards) = cmd_args.shards {
            let shard_range = std::ops::Range {
                start: shards[0],
                end: *shards.last().unwrap(),
            };

            info!("Starting shard range: {:?}", shard_range);

            if let Err(why) = client.start_shard_range(shard_range, shard_count).await {
                error!("Client error: {:?}", why);
                std::process::exit(1); // Clean exit with status code of 1
            }

            return;
        } else {
            info!("Starting shard count: {}", shard_count);

            if let Err(why) = client.start_shards(shard_count).await {
                error!("Client error: {:?}", why);
                std::process::exit(1); // Clean exit with status code of 1
            }

            return;
        }
    }

    info!("Starting using autosharding");

    if let Err(why) = client.start_autosharded().await {
        error!("Client error: {:?}", why);
        std::process::exit(1); // Clean exit with status code of 1
    }
}

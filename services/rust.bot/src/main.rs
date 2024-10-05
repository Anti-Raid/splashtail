mod ipc;
mod test;

use ipc::mewld::MewldIpcClient;

use botox::cache::CacheHttpImpl;
use gwevent::core::get_event_guild_id;
use silverpelt::ar_event::{AntiraidEvent, EventHandlerContext};

use std::sync::{Arc, LazyLock};
use tokio::sync::RwLock;

use cap::Cap;
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
    pub mewld_ipc: Arc<MewldIpcClient>,
    pub proxy_support_data: RwLock<Option<Arc<proxy_support::ProxySupportData>>>,
}

#[async_trait::async_trait]
impl silverpelt::data::Props for Props {
    /// Converts the props to std::any::Any
    fn as_any(&self) -> &(dyn std::any::Any + Send + Sync) {
        self
    }

    fn name(&self) -> String {
        "bot".to_string()
    }

    async fn shards(&self) -> Result<Vec<u16>, Error> {
        Ok(crate::ipc::argparse::MEWLD_ARGS.shards.clone())
    }

    async fn shard_count(&self) -> Result<u16, Error> {
        Ok(crate::ipc::argparse::MEWLD_ARGS.shard_count)
    }

    fn cluster_id(&self) -> u16 {
        crate::ipc::argparse::MEWLD_ARGS.cluster_id
    }

    fn cluster_name(&self) -> String {
        crate::ipc::argparse::MEWLD_ARGS.cluster_name.clone()
    }

    fn cluster_count(&self) -> u16 {
        crate::ipc::argparse::MEWLD_ARGS.cluster_count
    }

    fn available_clusters(&self) -> usize {
        self.mewld_ipc.cache.cluster_healths.len()
    }

    async fn total_guilds(&self) -> Result<u64, Error> {
        Ok(self.mewld_ipc.cache.total_guilds())
    }

    async fn total_users(&self) -> Result<u64, Error> {
        Ok(self.mewld_ipc.cache.total_users())
    }

    /// Proxy support data
    async fn get_proxysupport_data(&self) -> Option<Arc<proxy_support::ProxySupportData>> {
        let guard = self.proxy_support_data.read().await;

        match guard.as_ref() {
            Some(data) => {
                return Some(data.clone());
            }
            None => {
                return None;
            }
        }
    }

    /// Set the proxy support data
    async fn set_proxysupport_data(
        &self,
        data: proxy_support::ProxySupportData,
    ) -> Result<(), silverpelt::Error> {
        let mut guard = self.proxy_support_data.write().await;
        *guard = Some(Arc::new(data));

        Ok(())
    }
}

async fn event_listener<'a>(
    ctx: poise::FrameworkContext<'a, Data, Error>,
    event: &FullEvent,
) -> Result<(), Error> {
    let user_data = ctx.serenity_context.data::<Data>();
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

            // We don't really care which shard runs this, we just need one to run it
            if !CONNECT_STATE
                .started_tasks
                .load(std::sync::atomic::Ordering::SeqCst)
            {
                info!("Starting background tasks");
                let tasks = bot_binutils::get_tasks(ctx.serenity_context, &user_data);
                tokio::task::spawn(botox::taskman::start_all_tasks(
                    tasks,
                    ctx.serenity_context.clone(),
                ));

                info!("Starting IPC");

                let data = ctx.serenity_context.data::<Data>();
                let props = data.props.as_any().downcast_ref::<Props>().unwrap();
                let ipc_ref = props.mewld_ipc.clone();
                let ch = CacheHttpImpl::from_ctx(ctx.serenity_context);
                let sm = ctx.shard_manager().clone();
                tokio::task::spawn(async move {
                    let ipc_ref = ipc_ref;
                    ipc_ref.start_ipc_listener(&ch, &sm).await;
                });

                // Create a new rpc server
                let rpc_server =
                    rust_rpc_server_bot::create_bot_rpc_server(data.clone(), ctx.serenity_context);

                // Start the rpc server
                tokio::task::spawn(async move {
                    log::info!(
                        "Starting RPC server on cluster {}",
                        ipc::argparse::MEWLD_ARGS.cluster_id
                    );
                    let opts = rust_rpc_server::CreateRpcServerOptions {
                        bind: rust_rpc_server::CreateRpcServerBind::Address(format!(
                            "{}:{}",
                            config::CONFIG.base_ports.bot_bind_addr.get(),
                            config::CONFIG.base_ports.bot.get()
                                + ipc::argparse::MEWLD_ARGS.cluster_id
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

            if ctx.serenity_context.shard_id.0
                == *crate::ipc::argparse::MEWLD_ARGS.shards.last().unwrap()
            {
                info!("All shards ready, launching next cluster");
                let data = ctx.serenity_context.data::<Data>();
                let props = data.props.as_any().downcast_ref::<Props>().unwrap();
                if let Err(e) = props.mewld_ipc.publish_ipc_launch_next().await {
                    error!("Error publishing IPC launch next: {}", e);
                    return Err(e);
                }

                info!(
                    "Published IPC launch next to channel {}",
                    crate::ipc::argparse::MEWLD_ARGS.mewld_redis_channel
                );
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

    let mut default_filter =
        "serenity=error,fred=error,rust_bot=info,bot_binutils=info,rust_rpc_server=info,rust_rpc_server_bot=info,botox=info,templating=debug"
            .to_string();

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
                "[{} ({} of {})] ({}) {} - {}",
                cluster_name,
                cluster_id,
                cluster_count - 1,
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

    info!("{:#?}", ipc::argparse::MEWLD_ARGS);

    let proxy_url = config::CONFIG.meta.proxy.get().clone();

    info!("Proxy URL: {}", proxy_url);

    let http = Arc::new(
        HttpBuilder::new(&config::CONFIG.discord_auth.token.get())
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

    info!("Connecting to redis");

    let pool = fred::prelude::Builder::from_config(
        fred::prelude::RedisConfig::from_url(&config::CONFIG.meta.bot_redis_url)
            .expect("Could not initialize Redis config"),
    )
    .build_pool(REDIS_MAX_CONNECTIONS.try_into().unwrap())
    .expect("Could not initialize Redis pool");

    let pg_pool = PgPoolOptions::new()
        .max_connections(POSTGRES_MAX_CONNECTIONS)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect(&config::CONFIG.meta.postgres_url)
        .await
        .expect("Could not initialize connection");

    let reqwest = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(30))
        .timeout(std::time::Duration::from_secs(90))
        .build()
        .expect("Could not initialize reqwest client");

    let props = Arc::new(Props {
        mewld_ipc: Arc::new(ipc::mewld::MewldIpcClient {
            redis_pool: pool.clone(),
            cache: Arc::new(ipc::mewld::MewldIpcCache::default()),
            pool: pg_pool.clone(),
        }),
        pool: pg_pool.clone(),
        proxy_support_data: RwLock::new(None),
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

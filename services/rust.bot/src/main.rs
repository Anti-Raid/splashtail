mod ext_generate;
mod ipc;

use ipc::{
    animus_magic::client::{AnimusMagicClient, ClientData},
    mewld::MewldIpcClient,
};

use botox::cache::CacheHttpImpl;
use gwevent::core::get_event_guild_id;
use modules::silverpelt::{
    module_config::is_module_enabled, silverpelt_cache::SILVERPELT_CACHE, EventHandlerContext,
};
use splashcore_rs::value::Value;

use once_cell::sync::Lazy;
use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;

use base_data::{Data, Error};
use cap::Cap;
use log::{error, info, warn};
use serenity::all::{FullEvent, GuildId, HttpBuilder, UserId};
use sqlx::postgres::PgPoolOptions;
use std::alloc;
use std::io::Write;

#[global_allocator]
static ALLOCATOR: Cap<alloc::System> = Cap::new(alloc::System, usize::MAX);

pub struct ConnectState {
    pub started_tasks: std::sync::atomic::AtomicBool,
    pub ready: dashmap::DashMap<serenity::all::ShardId, bool>,
    pub ready_lock: tokio::sync::Mutex<()>,
}

pub static CONNECT_STATE: Lazy<ConnectState> = Lazy::new(|| ConnectState {
    started_tasks: std::sync::atomic::AtomicBool::new(false),
    ready: dashmap::DashMap::new(),
    ready_lock: tokio::sync::Mutex::new(()),
});

/// Props
pub struct Props {
    pub pool: sqlx::PgPool,
    pub mewld_ipc: Arc<MewldIpcClient>,
    pub animus_magic_ipc: OnceLock<Arc<AnimusMagicClient>>, // a rwlock is needed as the cachehttp is only available after the client is started
}

#[async_trait::async_trait]
impl base_data::Props for Props {
    fn underlying_am_client(
        &self,
    ) -> Result<
        Box<dyn splashcore_rs::animusmagic::client::AnimusMagicRequestClient>,
        base_data::Error,
    > {
        let am = self.animus_magic_ipc.get();

        match am {
            Some(am) => Ok(Box::new(am.underlying_client.clone())),
            None => Err("Animus Magic IPC not initialized".into()),
        }
    }

    fn permodule_executor(&self) -> Box<dyn base_data::permodule::PermoduleFunctionExecutor> {
        Box::new(PermoduleFunctionExecutor {})
    }

    fn add_permodule_function(
        &self,
        module: &str,
        function: &str,
        func: base_data::permodule::ToggleFunc,
    ) {
        PERMODULE_FUNCTIONS.insert((module.to_string(), function.to_string()), func);
    }

    fn name(&self) -> String {
        "bot".to_string()
    }

    fn shards(&self) -> Vec<u16> {
        crate::ipc::argparse::MEWLD_ARGS.shards.clone()
    }

    fn shard_count(&self) -> u16 {
        crate::ipc::argparse::MEWLD_ARGS.shard_count
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

    fn total_guilds(&self) -> u64 {
        self.mewld_ipc.cache.total_guilds()
    }

    fn total_users(&self) -> u64 {
        self.mewld_ipc.cache.total_users()
    }

    async fn reset_can_use_bot(&self) -> Result<(), base_data::Error> {
        load_can_use_bot_whitelist(&self.pool).await?;
        Ok(())
    }
}

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

// In order to allow modules to implement their own internal caches/logic without polluting the animus magic protocol,
// we implement PERMODULE_FUNCTIONS which any module can register/add on to
//
// Format of a permodule toggle is (module_name, toggle)
pub static PERMODULE_FUNCTIONS: Lazy<
    dashmap::DashMap<(String, String), base_data::permodule::ToggleFunc>,
> = Lazy::new(dashmap::DashMap::new);

pub struct PermoduleFunctionExecutor {}

#[async_trait::async_trait]
impl base_data::permodule::PermoduleFunctionExecutor for PermoduleFunctionExecutor {
    async fn execute_permodule_function(
        &self,
        cache_http: &botox::cache::CacheHttpImpl,
        module: &str,
        function: &str,
        arguments: &indexmap::IndexMap<String, Value>,
    ) -> Result<(), crate::Error> {
        let key = (module.to_string(), function.to_string());
        let func = PERMODULE_FUNCTIONS.get(&key);

        let Some(func) = func else {
            return Err(format!("Function {} not found for module {}", function, module).into());
        };

        func(cache_http, arguments).await
    }
}

async fn load_can_use_bot_whitelist(pool: &sqlx::PgPool) -> Result<CanUseBotList, Error> {
    // Fetch can_use_bot list
    let rec = sqlx::query!("SELECT id, type FROM can_use_bot")
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Error fetching can_use_bot list: {}", e))?;

    let mut users = Vec::new();
    let mut guilds = Vec::new();

    for item in rec {
        match item.r#type.as_str() {
            "user" => {
                let id = item
                    .id
                    .parse::<UserId>()
                    .map_err(|e| format!("Failed to parse user id: {}", e))?;
                users.push(id);
            }
            "guild" => {
                let id = item
                    .id
                    .parse::<GuildId>()
                    .map_err(|e| format!("Failed to parse guild id: {}", e))?;
                guilds.push(id);
            }
            _ => {
                continue;
            }
        }
    }

    for root_user in config::CONFIG.discord_auth.root_users.iter() {
        users.push(*root_user);
    }

    Ok(CanUseBotList { users, guilds })
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

    let changes: [&str; 4] = [
        "We are working extremely hard on Antiraid v6, and have completed working on half of the bot. We should have this update out by Q1/Q2 2024! Delays may occur due to the sheer scope of the unique features we want to provide!",
        "Yet another update: we are in the process of adding some MASSIVE new features including advanced permission management, server member limits, AI image classification, server member backups and custom customizable github webhook support (for developers)",
        "Update (Tuesday, July 2nd 2024 Edition): We are still working on the bot. It is taking longer than expected due to the large amount of new features being added. You can also request specific features you want in Anti-Raid on our Discord Server!",
        "Update (July 15th): Our developers want feedback on what we should add to the bot! Please join our support server and give your wishlist now!"
    ];

    let updates = poise::serenity_prelude::CreateEmbed::default()
        .color(0x0000ff)
        .title("Updates")
        .description(changes.join("\t-"));

    let statistics = poise::serenity_prelude::CreateEmbed::default()
    .color(0xff0000)
    .description(format!(
        "**Server Count:** {}\n**Shard Count:** {}\n**Cluster Count:** {}\n**Cluster ID:** {}\n**Cluster Name:** {}\n**Uptime:** {}",
        user_data.props.total_guilds(),
        ipc::argparse::MEWLD_ARGS.shard_count,
        ipc::argparse::MEWLD_ARGS.cluster_count,
        ipc::argparse::MEWLD_ARGS.cluster_id,
        ipc::argparse::MEWLD_ARGS.cluster_name,
        {
            let duration: std::time::Duration = std::time::Duration::from_secs((chrono::Utc::now().timestamp() - config::CONFIG.start_time) as u64);
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
                ctx.command().qualified_name,
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
            if !CONNECT_STATE
                .ready
                .contains_key(&ctx.serenity_context.shard_id)
            {
                // Send maint message in response
                let ic = match interaction {
                    serenity::all::Interaction::Command(ic) => ic,
                    _ => return Ok(()),
                };

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
                // Get all tasks
                let mut tasks = Vec::new();
                for module in modules::modules::modules() {
                    for (task, confirm_task) in module.background_tasks {
                        let (confirmed, reason) = (confirm_task)(ctx.serenity_context);
                        if confirmed {
                            info!(
                                "Adding task {} with confirm_task reason: {}",
                                task.name, reason
                            );
                        } else {
                            info!(
                                "Skipping task {} as it is disabled for reason: {}",
                                task.name, reason
                            );
                            continue;
                        }

                        tasks.push(task);
                    }
                }

                tokio::task::spawn(botox::taskman::start_all_tasks(
                    tasks,
                    ctx.serenity_context.clone(),
                ));

                info!("Starting IPC");

                let data = ctx.serenity_context.data::<Data>();
                let props = data.extra_data::<Props>();
                let ipc_ref = props.mewld_ipc.clone();
                let ch = CacheHttpImpl::from_ctx(ctx.serenity_context);
                let sm = ctx.shard_manager().clone();
                tokio::task::spawn(async move {
                    let ipc_ref = ipc_ref;
                    ipc_ref.start_ipc_listener(&ch, &sm).await;
                });

                // And for animus magic
                let am = Arc::new(AnimusMagicClient::new(ClientData {
                    pool: data.pool.clone(),
                    redis_pool: data.redis_pool.clone(),
                    reqwest: data.reqwest.clone(),
                    cache_http: CacheHttpImpl::from_ctx(ctx.serenity_context),
                }));

                props.animus_magic_ipc.get_or_init(|| am.clone());

                tokio::task::spawn(async move {
                    let am_ref = am.clone();
                    am_ref.listen().await;
                });

                CONNECT_STATE
                    .started_tasks
                    .store(true, std::sync::atomic::Ordering::SeqCst);
            }

            if ctx.serenity_context.shard_id.0
                == *crate::ipc::argparse::MEWLD_ARGS.shards.last().unwrap()
            {
                info!("All shards ready, launching next cluster");
                let props = user_data.extra_data::<Props>();
                if let Err(e) = props.mewld_ipc.publish_ipc_launch_next().await {
                    error!("Error publishing IPC launch next: {}", e);
                    return Err(e);
                }

                info!(
                    "Published IPC launch next to channel {}",
                    crate::ipc::argparse::MEWLD_ARGS.mewld_redis_channel
                );
            }

            if !CONNECT_STATE
                .ready
                .contains_key(&ctx.serenity_context.shard_id)
            {
                for module in modules::modules::modules() {
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
        .parse_filters("serenity=error,fred=error,bot=info,modules=info,templating=debug")
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
            for module in modules::modules::modules() {
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
                let user_id = ctx.author().id;
                let allowed = config::CONFIG.discord_auth.public_bot || {
                    let cub_cache = CAN_USE_BOT_CACHE.read().await;
                    if let Some(ref guild_id) = ctx.guild_id() {
                        cub_cache.guilds.contains(guild_id) && cub_cache.users.contains(&user_id)
                    } else {
                        cub_cache.users.contains(&user_id)
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

                let user = sqlx::query!(
                    "SELECT COUNT(*) FROM users WHERE user_id = $1",
                    guild_id.to_string()
                )
                .fetch_one(&data.pool)
                .await?;

                if user.count.unwrap_or_default() == 0 {
                    // User not found, create it
                    sqlx::query!(
                        "INSERT INTO users (user_id) VALUES ($1)",
                        guild_id.to_string()
                    )
                    .execute(&data.pool)
                    .await?;
                }

                let command = ctx.command();

                let res = modules::silverpelt::cmd::check_command(
                    command.name.as_str(),
                    &command.qualified_name,
                    guild_id,
                    ctx.author().id,
                    &data.pool,
                    &CacheHttpImpl::from_ctx(ctx.serenity_context()),
                    &Some(ctx),
                    modules::silverpelt::cmd::CheckCommandOptions {
                        channel_id: Some(ctx.channel_id()),
                        ..Default::default()
                    },
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
    let cub_list = load_can_use_bot_whitelist(&pg_pool)
        .await
        .expect("Could not fetch the users who are allowed to use the bot");

    // Save to CAN_USE_BOT_CACHE
    let mut cub = CAN_USE_BOT_CACHE.write().await;
    *cub = cub_list;
    drop(cub);

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
        animus_magic_ipc: OnceLock::new(),
        pool: pg_pool.clone(),
    });

    let data = Data {
        redis_pool: pool.clone(),
        object_store: Arc::new(
            config::CONFIG
                .object_storage
                .build()
                .expect("Could not initialize object store"),
        ),
        pool: pg_pool.clone(),
        reqwest,
        proxy_support_data: RwLock::new(None),
        extra_data: props.clone(),
        props: props.clone(),
    };

    info!("Initializing bot state");

    for module in modules::modules::modules() {
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

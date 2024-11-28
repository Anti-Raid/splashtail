use log::{error, info};

#[allow(unused_imports)] // They aren't unused
use serenity::all::{FullEvent, HttpBuilder};
use silverpelt::{data::Data, Error};
use sqlx::postgres::PgPoolOptions;
use std::io::Write;
use std::sync::Arc;
use std::sync::LazyLock;

static TEST_ONE_TESTER: LazyLock<tokio::sync::Mutex<bool>> =
    LazyLock::new(|| tokio::sync::Mutex::new(false));

fn modules() -> Vec<Box<dyn silverpelt::module::Module>> {
    bot_modules_default::modules()
}

pub async fn run_tester() {
    test_module_parse();
    check_modules_test().await;
}

pub fn test_module_parse() {
    let _ = modules();
}

pub async fn check_modules_test() {
    const POSTGRES_MAX_CONNECTIONS: u32 = 3; // max connections to the database, we don't need too many here

    let debug_mode = std::env::var("DEBUG").unwrap_or_default() == "true";
    let debug_opts = std::env::var("DEBUG_OPTS").unwrap_or_default();

    let mut env_builder = env_logger::builder();

    let mut default_filter =
        "serenity=error,rust_assetgen=info,bot_binutils=info,botox=info,templating=debug,sqlx=error"
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

    info!("Starting check_modules_test");

    let proxy_url = config::CONFIG.meta.proxy.clone();

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
        event_handler: |ctx, event| Box::pin(start_testing(ctx, event)),
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

    info!("DB Connect [start]");

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
        _pool: pg_pool.clone(),
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

    // Call get gateway bot. This assumes the user has a proper antiraid setup with a proxy setup that makes this call cheap+indefinitely callable (like sandwich)
    info!("Getting bot gateway");
    let gateway = client.http.get_bot_gateway().await.unwrap();

    if let Err(why) = client.start_shard(0, gateway.shards.get()).await {
        panic!("Client error occurred during testing: {:?}", why);
    }
}

async fn start_testing<'a>(
    _ctx: poise::FrameworkContext<'a, Data, Error>,
    event: &FullEvent,
) -> Result<(), Error> {
    match event {
        FullEvent::Ready { .. } => {
            let lg = TEST_ONE_TESTER.lock().await;

            if *lg {
                return Ok(()); // Already running the test
            }

            drop(lg);

            info!("Starting check_modules_test_impl");

            match check_modules_test_impl().await {
                Ok(_) => {
                    info!("check_modules_test_impl passed");
                    std::process::exit(0);
                }
                Err(e) => {
                    error!("check_modules_test_impl failed: {:?}", e);
                    std::process::exit(1)
                }
            };
        }
        _ => {
            // Not used for now
        }
    };

    Ok(())
}

async fn check_modules_test_impl() -> Result<(), Error> {
    // Check for env var CHECK_MODULES_TEST_ENABLED
    if std::env::var("CHECK_MODULES_TEST_ENABLED").unwrap_or_default() == "true" {
        return Ok(());
    }

    // Set current directory to ../../
    let current_dir = std::env::current_dir().unwrap();

    if current_dir.ends_with("services/rust.bot") {
        std::env::set_current_dir("../../")?;
    }

    for module in modules() {
        module.validate()?;
    }

    Ok(())
}

// Boilerplate code
pub struct Props {
    pub _pool: sqlx::PgPool,
}

#[async_trait::async_trait]
impl silverpelt::data::Props for Props {
    /// Converts the props to std::any::Any
    fn as_any(&self) -> &(dyn std::any::Any + Send + Sync) {
        self
    }

    fn extra_description(&self) -> String {
        "Test Bot".to_string()
    }

    async fn shards(&self) -> Result<Vec<u16>, Error> {
        Ok(vec![])
    }

    /// Returns the shard messenger given the shard id
    async fn shard_messenger(
        &self,
        _shard_id: serenity::all::ShardId,
    ) -> Result<serenity::all::ShardMessenger, silverpelt::Error> {
        Err("Not implemented in test environment".into())
    }

    async fn shard_count(&self) -> Result<u16, Error> {
        Ok(0)
    }

    async fn total_guilds(&self) -> Result<u64, Error> {
        Ok(0)
    }

    async fn total_users(&self) -> Result<u64, Error> {
        Ok(0)
    }
}

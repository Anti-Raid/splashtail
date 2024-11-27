use log::{error, info};

#[allow(unused_imports)] // They aren't unused
use serenity::all::{FullEvent, HttpBuilder};
use std::io::Write;
use std::sync::Arc;

pub fn modules() -> Vec<Box<dyn silverpelt::module::Module>> {
    bot_modules_default::modules()
}

pub async fn register_poise_commands() {
    let debug_mode = std::env::var("DEBUG").unwrap_or_default() == "true";
    let debug_opts = std::env::var("DEBUG_OPTS").unwrap_or_default();

    let mut env_builder = env_logger::builder();

    let default_filter =
        "serenity=debug,rust_assetgen=info,bot_binutils=info,botox=info,templating=debug,sqlx=error"
            .to_string();

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

    info!("Registering poise commands");

    let silverpelt_cache = {
        let mut silverpelt_cache = silverpelt::cache::SilverpeltCache::default();

        for module in modules() {
            silverpelt_cache.add_module(module);
        }

        Arc::new(silverpelt_cache)
    };

    let commands = bot_binutils::get_commands(&silverpelt_cache);

    let http = Arc::new(
        HttpBuilder::new(&config::CONFIG.discord_auth.token)
            .proxy(config::CONFIG.meta.proxy.clone())
            .ratelimiter_disabled(true)
            .build(),
    );

    let app = http
        .get_current_application_info()
        .await
        .expect("Failed to get application info");
    http.set_application_id(app.id);

    let commands_builder = poise::builtins::create_application_commands(&commands);
    let num_commands = commands_builder.len();

    println!("Registering {} commands", num_commands);

    serenity::all::Command::set_global_commands(&http, &commands_builder)
        .await
        .expect("Failed to set global commands");
}

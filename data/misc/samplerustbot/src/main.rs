mod config;

use log::{error, info};
use serenity::{all::{RawEventHandler, ShardId}, async_trait};
use std::{io::Write, sync::Arc};

type Error = Box<dyn std::error::Error + Send + Sync>;

pub struct EventDispatch {}

#[async_trait]
impl RawEventHandler for EventDispatch {
    async fn raw_event(&self, ctx: serenity::all::client::Context, event: serenity::all::Event) {
        match event {
            serenity::all::Event::Ready(data_about_bot) => {
                info!("{} is now ready on shard {}", data_about_bot.ready.user.name, ctx.shard_id);
            },
            _ => {
                match event {
                    serenity::all::Event::InteractionCreate(_) => {},
                    serenity::all::Event::GuildAuditLogEntryCreate(_) => {},
                    serenity::all::Event::GuildCreate(_) => {},
                    _ => return
                }
                info!("Event: {:?}", event)
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // Setup logging
    let debug_mode = std::env::var("DEBUG").unwrap_or_default() == "true";

    let mut env_builder = env_logger::builder();

    env_builder
    .format(move |buf, record| writeln!(buf, "{} - {}", record.level(), record.args()))
    .filter(Some("samplerustbot"), log::LevelFilter::Info);

    if debug_mode {
        env_builder.filter(None, log::LevelFilter::Debug);
    } else {
        env_builder.filter(None, log::LevelFilter::Error);
    }

    env_builder.init();

    let proxy_url = std::env::var("PROXY_URL").unwrap_or_default();

    let mut http = serenity::all::HttpBuilder::new(&config::CONFIG.discord_auth.token);

    if !proxy_url.is_empty() {
        log::info!("Using proxy {}", proxy_url);
        http = http.ratelimiter_disabled(true).proxy(proxy_url);
    }

    let http = http.build();

    let mut intents = serenity::all::GatewayIntents::all();

    // The really spammy intents
    intents.remove(serenity::all::GatewayIntents::GUILD_PRESENCES); // Don't even have the privileged gateway intent for this
    intents.remove(serenity::all::GatewayIntents::GUILD_MESSAGE_TYPING); // Don't care about typing
    intents.remove(serenity::all::GatewayIntents::DIRECT_MESSAGE_TYPING); // Don't care about typing
    intents.remove(serenity::all::GatewayIntents::DIRECT_MESSAGES); // Don't care about DMs

    let client_builder = serenity::all::ClientBuilder::new_with_http(
        http,
        intents,
    );

    let mut client = client_builder
        .raw_event_handler(EventDispatch {})
        .await
        .expect("Error creating client");

    if let Err(why) = client.start_autosharded().await {
        error!("Client error: {:?}", why);
    }

    std::process::exit(1); // Clean exit with status code of 1
}

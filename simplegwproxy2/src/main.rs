mod impls;
mod config;
mod ws;
mod models;

use log::{error, info};
use serenity::{all::RawEventHandler, async_trait};
use std::{io::Write, sync::Arc};

type Error = Box<dyn std::error::Error + Send + Sync>;

pub struct EventDispatch {}

#[async_trait]
impl RawEventHandler for EventDispatch {
    async fn raw_event(&self, ctx: serenity::client::Context, event: serenity::all::Event) {
        match event {
            serenity::all::Event::Ready(data_about_bot) => {
                info!("{} is now ready on shard {}", data_about_bot.ready.user.name, ctx.shard_id);
            }
            _ => {
                // Reserialize the event back to its raw value form
                let Ok(raw) = serde_json::to_value(&event) else {
                    error!("Failed to serialize event: {:?}", event);
                    return;
                };
                
                for sess in ws::SESSIONS.iter() {
                    let session = sess.value();

                    if session.dispatcher.is_closed() {
                        // Push to missed_events
                        let mut missed_events = session.missed_events.lock().await;
                        missed_events.push_back(raw.clone());
                        drop(missed_events);
                    }

                    if session.state == crate::models::SessionState::Unidentified {
                        continue;
                    }

                    let sess_shard = serenity::all::ShardId(session.shard[0]);
                    
                    if sess_shard != ctx.shard_id {
                        continue;
                    }

                    if let Err(e) = session.dispatcher.send(crate::models::QueuedEvent::DispatchValue(Arc::new(raw.clone()))).await {
                        error!("Failed to send event to session: {}", e);
                    }
                }
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
    .filter(Some("simplegwproxy2"), log::LevelFilter::Info);

    if debug_mode {
        env_builder.filter(None, log::LevelFilter::Debug);
    } else {
        env_builder.filter(None, log::LevelFilter::Error);
    }

    env_builder.init();

    let http = serenity::all::HttpBuilder::new(&config::CONFIG.discord_auth.token).build();

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

    let cache = client.cache.clone();
    let http = client.http.clone();
    let shard_manager = client.shard_manager.clone();
    tokio::spawn(async move {
        ws::start_ws(impls::cache::CacheHttpImpl { 
            cache,
            http,
            shard_manager
        }).await.expect("Failed to start websocket");
    });

    if let Err(why) = client.start_autosharded().await {
        error!("Client error: {:?}", why);
    }

    std::process::exit(1); // Clean exit with status code of 1
}

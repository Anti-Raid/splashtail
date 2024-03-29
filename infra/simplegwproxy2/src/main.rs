mod impls;
mod config;
mod ws;
mod models;

use log::{error, info};
use serenity::{all::{RawEventHandler, ShardId}, async_trait};
use std::{io::Write, sync::Arc};

type Error = Box<dyn std::error::Error + Send + Sync>;

pub static IS_READY: once_cell::sync::Lazy<tokio::sync::RwLock<dashmap::DashMap<ShardId, bool>>> = once_cell::sync::Lazy::new(|| tokio::sync::RwLock::new(dashmap::DashMap::new()));

pub struct EventDispatch {}

#[async_trait]
impl RawEventHandler for EventDispatch {
    async fn raw_event(&self, ctx: serenity::client::Context, event: serenity::all::Event) {
        match event {
            serenity::all::Event::Ready(data_about_bot) => {
                info!("{} is now ready on shard {}", data_about_bot.ready.user.name, ctx.shard_id);

                let is_ready = IS_READY.write().await;

                is_ready.insert(ctx.shard_id, true);
            },
            serenity::all::Event::Resumed(_) => {},
            _ => {
                {
                    let is_ready = IS_READY.read().await;

                    if is_ready.contains_key(&ctx.shard_id) {
                        if !*is_ready.get(&ctx.shard_id).unwrap().value() {
                            return;
                        }
                    } else {
                        return;
                    }
                }

                // Reserialize the event back to its raw value form
                let Ok(raw) = serde_json::to_value(&event) else {
                    error!("Failed to serialize event: {:?}", event);
                    return;
                };
                
                for sess in ws::SESSIONS.iter() {
                    let session_id = sess.key();
                    let session = sess.value();

                    if session.dispatcher.is_closed() {
                        // Only some events should not be counted under missing events
                        match event {
                            serenity::all::Event::InteractionCreate(_) => continue, // Useless, by the time the session is back up, the interaction will have expired
                            serenity::all::Event::GuildAuditLogEntryCreate(_) => continue, // By the time we resume, the audit log create event is no longer really useful
                            _ => {},
                        }

                        // Push to SESSION_MISSING_EVENTS
                        let Some(mes) = ws::SESSION_MISSING_EVENTS.get(session_id) else {
                            log::warn!("No missing events queue for shard {}", session.shard[0]);
                            continue;
                        };

                        let mut me_queue = mes.lock().await;

                        me_queue.push_back(raw.clone());

                        drop(me_queue);

                        continue;
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

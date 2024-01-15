use dashmap::DashMap;
use once_cell::sync::Lazy;
use serenity::all::UnavailableGuild;
use tokio::net::{TcpListener, TcpStream};
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use tokio_websockets::{CloseCode, Message, WebSocketStream};
use crate::impls::cache::CacheHttpImpl;

static SESSIONS: Lazy<DashMap<String, Session>> = Lazy::new(DashMap::new);

const GATEWAY_VERSION: u8 = 10;
const HEARTBEAT_INTERVAL: u128 = 4000;

pub async fn start_ws(cache_http: CacheHttpImpl) -> Result<(), crate::Error> {
    let listener = TcpListener::bind(
        format!("127.0.0.1:{}", crate::config::CONFIG.simple_gateway_proxy.port)
    ).await?;

    while let Ok((stream, _)) = listener.accept().await {
        let ws_stream = tokio_websockets::ServerBuilder::new()
        .accept(stream)
        .await;

        if let Err(e) = ws_stream {
            log::error!("Failed to accept client: {}", e);
            continue;
        }

        let ws_stream = ws_stream.unwrap();

        let ch = cache_http.clone();
        tokio::spawn(async move {
            // Just an echo server, really
            if let Err(e) = connection(ws_stream, ch).await {
                log::error!("Failed to handle connection: {}", e);
            }
        });
    }

    Ok(())
}

pub enum QueuedEvent {
    SendHeartbeat,
    Stop,
    Dispatch(serde_json::Value),
    Close(CloseCode, String)
}

#[derive(PartialEq)]
pub enum SessionState {
    Unidentified,
    Authorized,
}

pub struct Session {
    last_heartbeat: std::time::Instant,
    shard: [u16; 2],
    dispatcher: tokio::sync::mpsc::Sender<QueuedEvent>,
    state: SessionState,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Hello {
    heartbeat_interval: u128,
}

#[derive(serde::Serialize, serde::Deserialize)]
/// Nothing else matters
pub struct Identify {
    token: String,
    shard: [u16; 2],
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Event<T: Sized> {
    op: u8,
    d: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    t: Option<String>,
}

pub async fn dispatch_manager(
    session_id: String, 
    mut dispatcher_recv: tokio::sync::mpsc::Receiver<QueuedEvent>,
    mut ws_sender: SplitSink<WebSocketStream<TcpStream>, Message>
) {
    // Read the channel
    let mut seq_no = 0;
    while let Some(event) = dispatcher_recv.recv().await {
        match event {
            QueuedEvent::SendHeartbeat => {
                // No-op for now
            },
            QueuedEvent::Stop => {
                // Stop the sender entirely
                break;
            },
            QueuedEvent::Close(code, reason) => {
                // Close the connection
                if let Err(e) = ws_sender.send(Message::close(Some(code), &reason)).await {
                    log::error!("Failed to close websocket [send]: {}", e);
                }
                if let Err(e) = ws_sender.close().await {
                    log::error!("Failed to close websocket [close]: {}", e);
                }
                break;
            },
            QueuedEvent::Dispatch(mut event) => {
                // Set the sequence number
                event["s"] = serde_json::Value::from(seq_no);

                // Send the event
                let Ok(evt_json) = serde_json::to_string(&event) else {
                    log::error!("Failed to serialize event");
                    continue;
                };

                if let Err(e) = ws_sender.send(Message::text(evt_json)).await {
                    log::error!("Failed to send event: {}", e);
                }

                seq_no += 1;
            }
        }
    }
}

pub async fn connection(ws_stream: tokio_websockets::WebSocketStream<tokio::net::TcpStream>, cache_http: CacheHttpImpl) -> Result<(), crate::Error> {
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    let (sender, recv) = tokio::sync::mpsc::channel::<QueuedEvent>(32);

    let session_id = crate::impls::crypto::gen_random(32);

    SESSIONS.insert(session_id.clone(), Session {
        last_heartbeat: std::time::Instant::now(),
        shard: [0, 0],
        dispatcher: sender,
        state: SessionState::Unidentified,
    });

    // Send the hello message
    let hello_event = Event {
        op: 10,
        d: Hello {
            heartbeat_interval: HEARTBEAT_INTERVAL,
        },
        t: None,
    };

    ws_sender.send(Message::text(serde_json::to_string(&hello_event)?)).await?;

    // Start the dispatch manager
    let mut curr_tasks: Vec<tokio::task::JoinHandle<()>> = Vec::new();
    let dpm = tokio::spawn(dispatch_manager(session_id.clone(), recv, ws_sender));

    curr_tasks.push(dpm);

    while let Some(Ok(msg)) = ws_receiver.next().await {
        if msg.is_binary() {
            continue;
        }

        #[allow(clippy::collapsible_if)]
        if msg.is_text() {
            let Some(session) = SESSIONS.get(&session_id) else {
                for task in curr_tasks {
                    task.abort();
                }
                return Ok(());
            };
        
            if session.state == SessionState::Unidentified {
                // Try reading an identify message
                let text = msg.as_text().unwrap();

                let identify = serde_json::from_str::<Event<Identify>>(text);

                if let Err(e) = identify {
                    log::error!("Failed to parse identify message: {}", e);
                    continue;
                }

                let identify = identify.unwrap();

                if identify.op != 2 {
                    session.dispatcher.send(QueuedEvent::Close(CloseCode::PROTOCOL_ERROR, "Expected identify message".to_string())).await?;
                    break;
                }

                if identify.d.token != crate::config::CONFIG.discord_auth.token {
                    session.dispatcher.send(QueuedEvent::Close(CloseCode::PROTOCOL_ERROR, "Invalid token".to_string())).await?;
                    break;
                }

                drop(session); // Kill the reference to the session

                let Some(mut session) = SESSIONS.get_mut(&session_id) else {
                    for task in curr_tasks {
                        task.abort();
                    }
                    return Ok(());
                };

                session.shard = identify.d.shard;
                session.state = SessionState::Authorized;

                // Send Ready event to client
                let ready = Event {
                    op: 0,
                    d: crate::models::Ready {
                        version: GATEWAY_VERSION,
                        user: cache_http.cache.current_user().clone(),
                        session_id: session_id.clone(),
                        shard: Some(session.shard),
                        guilds: cache_http.cache.guilds().iter().map(|gid| {
                            crate::models::UnavailableGuild {
                                id: *gid,
                                unavailable: true,
                            }
                        }).collect(),
                        resume_gateway_url: format!("http://{}", crate::config::CONFIG.simple_gateway_proxy.url),
                        application: crate::models::PartialCurrentApplicationInfo {
                            id: cache_http.http.application_id().unwrap(),
                            flags: serenity::all::ApplicationFlags::empty(),
                        }
                    },
                    t: Some("READY".to_string()),
                };

                // Dispatch event
                session.dispatcher.send(QueuedEvent::Dispatch(serde_json::to_value(ready)?)).await?;

                // Create task in which we dispatch the current cache to the client
                let ch = cache_http.clone();
                let sid = session_id.clone();
                let sc = session.shard.clone();

                drop(session); // Kill the reference to the session

                let guild_fan_task = tokio::spawn(async move {
                    let dispatcher = {
                        let Some(sess) = SESSIONS.get(&sid) else {
                            log::error!("Failed to get session");
                            return;
                        };

                        sess.dispatcher.clone()
                    };

                    for guild in ch.cache.guilds() {
                        if serenity::utils::shard_id(guild, sc[1]) != sc[0] {
                            continue;
                        }

                        let guild_create_json = {
                            if let Some(guild) = ch.cache.guild(guild) {
                                // Send GUILD_CREATE
                                let guild_create = Event {
                                    op: 0,
                                    d: guild.clone(),
                                    t: Some("GUILD_CREATE".to_string()),
                                };
    
                                let Ok(guild_create_json) = serde_json::to_value(&guild_create) else {
                                    log::error!("Failed to serialize GUILD_CREATE");
                                    continue;
                                };

                                guild_create_json
                            } else {
                                continue;
                            }    
                        };

                        if let Err(e) = dispatcher.send(QueuedEvent::Dispatch(guild_create_json)).await {
                            log::error!("Failed to send GUILD_CREATE fanout: {}", e);
                            continue;
                        }
                    }
                });

                curr_tasks.push(guild_fan_task);

                continue;
            }

            let text = msg.as_text().unwrap();

            let event = serde_json::from_str::<serenity::all::GatewayEvent>(text);

            if let Err(e) = event {
                log::error!("Failed to parse event: {}", e);
                continue;
            }
        }
    }

    // Stop the dispatch manager
    for task in curr_tasks {
        task.abort();
    }

    Ok(())
}
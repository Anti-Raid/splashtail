use dashmap::DashMap;
use once_cell::sync::Lazy;
use tokio::net::{TcpListener, TcpStream};
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use tokio_websockets::{CloseCode, Message, WebSocketStream};
use crate::impls::cache::CacheHttpImpl;
use std::{sync::Arc, collections::VecDeque};
use tokio::sync::{Mutex, RwLock};
use splashcore_rs::crypto::gen_random;
use crate::models::{Event, EventOpCode, Identify, Session, SessionState, QueuedEvent, Hello};

pub static SESSIONS: Lazy<DashMap<String, Session>> = Lazy::new(DashMap::new);
pub static SESSION_SEQ_NO: Lazy<DashMap<String, RwLock<u64>>> = Lazy::new(DashMap::new);
pub static SESSION_MISSING_EVENTS: Lazy<DashMap<String, Mutex<VecDeque<serde_json::Value>>>> = Lazy::new(DashMap::new);

/// Increments the session sequence number and returns the new value
pub async fn incr_session_seq_no(session_id: &str) -> u64 {
    let seq_no: Option<dashmap::mapref::one::Ref<'_, String, RwLock<u64>>> = SESSION_SEQ_NO.get(session_id);

    if let Some(seq_no) = seq_no {
        let mut seq_no = seq_no.write().await;
        *seq_no += 1;
        return *seq_no;
    }

    SESSION_SEQ_NO.insert(session_id.to_string(), RwLock::new(0));

    0
}

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
            // Start the connection
            if let Err(e) = connection(ws_stream, ch).await {
                log::error!("Failed to handle connection: {}", e);
            }
        });
    }

    Ok(())
}

pub async fn dispatch_manager(
    session_id: Arc<String>, 
    mut dispatcher_recv: tokio::sync::mpsc::Receiver<QueuedEvent>,
    ws_sender: Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>
) {
    // Read the channel
    while let Some(event) = dispatcher_recv.recv().await {
        match event {
            QueuedEvent::Ping => {
                // Send heartbeat ACK
                log::info!("Heartbeat ACK on session {} at time {:#?}", session_id, std::time::SystemTime::now());
                let mut ws_sender = ws_sender.lock().await;

                if let Err(e) = ws_sender.send(Message::text("{\"op\":11}".to_string())).await {
                    log::error!("Failed to send heartbeat ACK: {}", e);
                }
            },
            QueuedEvent::Close(code, reason) => {
                let mut ws_sender = ws_sender.lock().await;
                // Close the connection
                if let Err(e) = ws_sender.send(Message::close(Some(code), &reason)).await {
                    log::error!("Failed to close websocket [send]: {}", e);
                }
                if let Err(e) = ws_sender.close().await {
                    log::error!("Failed to close websocket [close]: {}", e);
                }
                drop(ws_sender);
                break;
            },
            QueuedEvent::Dispatch(event) => {
                let ws_sender = ws_sender.clone();

                // Move event out of Arc
                let mut event = Arc::into_inner(event).unwrap();

                if event.op == crate::models::EventOpCode::Dispatch {
                    event.s = Some(incr_session_seq_no(&session_id).await); // Increment the sequence number
                }

                // Send the event
                let Ok(evt_json) = serde_json::to_string(&event) else {
                    log::error!("Failed to serialize event");
                    return;
                };

                let mut ws_sender = ws_sender.lock().await;

                if let Err(e) = ws_sender.send(Message::text(evt_json)).await {
                    // Hacky solution to prevent sending when closed
                    if e.to_string().contains("attempted to send message after closing connection") {
                        return;
                    }

                    log::error!("Failed to send event: {}", e);
                }

                drop(ws_sender);
            },
            QueuedEvent::DispatchValue(event) => {
                let ws_sender = ws_sender.clone();
                
                // Move event out of Arc
                let mut event = Arc::into_inner(event).unwrap();

                event["op"] = serde_json::Value::from(0); // Ensure op is set to Dispatch
                event["s"] = incr_session_seq_no(&session_id).await.into(); // Increment the sequence number

                // Send the event
                let Ok(evt_json) = serde_json::to_string(&event) else {
                    log::error!("Failed to serialize event");
                    return;
                };

                let mut ws_sender = ws_sender.lock().await;

                if let Err(e) = ws_sender.send(Message::text(evt_json)).await {
                    // Hacky solution to prevent sending when closed
                    if e.to_string().contains("attempted to send message after closing connection") {
                        return;
                    }

                    log::error!("Failed to send event: {}", e);
                }

                drop(ws_sender);
            },
        }
    }
}

pub async fn connection(ws_stream: tokio_websockets::WebSocketStream<tokio::net::TcpStream>, cache_http: CacheHttpImpl) -> Result<(), crate::Error> {
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    let (sender, recv) = tokio::sync::mpsc::channel::<QueuedEvent>(512);

    let mut session_id = gen_random(32);

    SESSIONS.insert(session_id.clone(), Session {
        last_heartbeat: std::time::Instant::now(),
        shard: [0, 0],
        dispatcher: sender,
        state: SessionState::Unidentified,
    });
    SESSION_MISSING_EVENTS.insert(session_id.clone(), Mutex::new(VecDeque::new()));

    // Send the hello message
    let hello_event = Event {
        op: EventOpCode::Hello,
        d: serde_json::to_value(Hello {
            heartbeat_interval: HEARTBEAT_INTERVAL,
        })?,
        t: None,
        s: None,
    };

    ws_sender.send(Message::text(serde_json::to_string(&hello_event)?)).await?;

    // Start the dispatch manager
    let ws_sender = Arc::new(Mutex::new(ws_sender));

    let mut curr_tasks: Vec<tokio::task::JoinHandle<()>> = Vec::new();
    let dpm = tokio::task::spawn(
        dispatch_manager(Arc::new(session_id.clone()), recv, ws_sender.clone())
    );

    curr_tasks.push(dpm);

    while let Some(Ok(msg)) = ws_receiver.next().await {
        if msg.is_binary() {
            continue;
        }

        #[allow(clippy::collapsible_if)]
        if msg.is_text() {
            let Some(session) = SESSIONS.get(&session_id) else {
                log::error!("Failed to get session {}", session_id);
                for task in curr_tasks {
                    task.abort();
                }
                return Ok(());
            };

            // Try reading an identify message
            let text = msg.as_text().unwrap();
            let event = serde_json::from_str::<Event>(text);

            if let Err(e) = event {
                log::error!("Failed to parse event: {}", e);
                continue;
            }

            let event = event.unwrap();

            match event.op {
                EventOpCode::Dispatch => {}, // Recieve only
                EventOpCode::Heartbeat => {
                    // Send PING to dispatcher
                    let dispatcher = session.dispatcher.clone();
                    tokio::spawn(
                        async move {
                            tokio::time::sleep(
                                std::time::Duration::from_micros(100) // Give 100us for the application to see the sent ping
                            )
                            .await;

                            if let Err(e) = dispatcher.send(QueuedEvent::Ping).await {
                                log::error!("Failed to send PING to dispatcher: {}", e);
                            }
                        }
                    );
                },
                EventOpCode::Identify => {
                    // Ensure we're unidentified
                    if session.state == SessionState::Unidentified {  
                        let identify = serde_json::from_value::<Identify>(event.d);
    
                        if let Err(e) = identify {
                            log::error!("Failed to parse identify message: {}", e);
                            continue;
                        }
    
                        let identify = identify.unwrap();
    
                        if identify.token != crate::config::CONFIG.discord_auth.token && format!("Bot {}", crate::config::CONFIG.discord_auth.token) != identify.token {
                            log::info!("Invalid token, got {}", identify.token);
                            session.dispatcher.send(QueuedEvent::Close(CloseCode::PROTOCOL_ERROR, "Invalid token".to_string())).await?;
                            break;
                        }
        
                        drop(session); // Kill the reference to the session
        
                        let Some(mut session) = SESSIONS.get_mut(&session_id) else {
                            log::error!("Failed to get session");
                            for task in curr_tasks {
                                task.abort();
                            }
                            return Ok(());
                        };
        
                        session.shard = identify.shard;
                        session.state = SessionState::Authorized;

                        log::info!("Authorized shard {}", session.shard[0]);
        
                        // Send Ready event to client
                        let ready = Arc::new(Event {
                            op: EventOpCode::Dispatch,
                            s: None,
                            d: serde_json::to_value(crate::models::Ready {
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
                            })?,
                            t: Some("READY".to_string()),
                        });
        
                        // Dispatch event
                        session.dispatcher.send(QueuedEvent::Dispatch(ready)).await?;
        
                        // Create task in which we dispatch the current cache to the client
                        let ch = cache_http.clone();
                        let sc = session.shard;
                        
                        let dispatcher = session.dispatcher.clone();
                        let guild_fan_task = tokio::spawn(async move {
                            for guild in ch.cache.guilds() {
                                // Avoid filling the channel with only guild fanouts to allow for other events to be sent
                                tokio::time::sleep(
                                    std::time::Duration::from_micros(12000) // 12ms
                                ).await;

                                if serenity::utils::shard_id(guild, sc[1]) != sc[0] {
                                    continue;
                                }

                                if dispatcher.is_closed() {
                                    log::warn!("Dispatcher is closed, ending guild fanout");
                                    return;
                                }
        
                                let guild_create = {
                                    if let Some(guild) = ch.cache.guild(guild) {
                                        // Send GUILD_CREATE
                                        Arc::new(Event {
                                            op: EventOpCode::Dispatch,
                                            s: None,
                                            d: serde_json::to_value(guild.clone()).unwrap(),
                                            t: Some("GUILD_CREATE".to_string()),
                                        })
                                    } else {
                                        continue;
                                    }    
                                };
        
                                if let Err(e) = dispatcher.send(QueuedEvent::Dispatch(guild_create)).await {
                                    log::error!("Failed to send GUILD_CREATE fanout: {}", e);
                                    continue;
                                }
                            }
                        });
            
                        curr_tasks.push(guild_fan_task);
            
                        continue;    
                    }       
                },
                EventOpCode::PresenceUpdate => {
                    // Send the event directly to Discord
                    let up = serde_json::from_value::<crate::models::GatewayUpdatePresence>(event.d);

                    if let Err(e) = up {
                        log::error!("Failed to parse presence update: {}", e);
                        continue;
                    }

                    let up = up.unwrap();

                    let runners = cache_http.shard_manager.runners.lock().await;

                    let shard_id = serenity::all::ShardId(session.shard[0]);
                    if let Some(runner) = runners.get(&shard_id) {
                        runner.runner_tx.set_presence({
                            if let Some(activity) = up.activities {
                                if activity.is_empty() {
                                    None
                                } else {
                                    Some(
                                        serenity::all::ActivityData {
                                            name: activity[0].name.clone(),
                                            kind: activity[0].kind,
                                            url: activity[0].url.clone(),
                                            state: activity[0].state.clone(),
                                        }
                                    )
                                }
                            } else {
                                None
                            }
                        }, up.status);
                    }
                },
                EventOpCode::VoiceStateUpdate => continue, // Not supported
                EventOpCode::Resume => {
                    if session.state == SessionState::Unidentified {
                        let resume = serde_json::from_value::<crate::models::GatewayResumeEvent>(event.d);

                        if let Err(e) = resume {
                            log::error!("Failed to parse resume message: {}", e);
                            continue;
                        }

                        let resume = resume.unwrap();

                        if resume.token != crate::config::CONFIG.discord_auth.token && format!("Bot {}", crate::config::CONFIG.discord_auth.token) != resume.token {
                            session.dispatcher.send(QueuedEvent::Close(CloseCode::PROTOCOL_ERROR, "Invalid token".to_string())).await?;
                            break;
                        }

                        // Check if session ID is in session
                        if !SESSIONS.contains_key(&resume.session_id) {
                            // Send Invalid Session event
                            let ise = Arc::new(Event {
                                op: EventOpCode::InvalidSession,
                                s: None,
                                d: serde_json::Value::Bool(false),
                                t: None,
                            });

                            session.dispatcher.send(QueuedEvent::Dispatch(ise)).await?;

                            continue
                        }

                        // Kill the old dispatcher
                        for task in curr_tasks.iter() {
                            task.abort();
                        }

                        drop(session); // Kill the reference to the session

                        // Delete the current session
                        let sid = session_id.clone();
                        tokio::spawn(async move {
                            SESSIONS.remove(&sid);
                            SESSION_SEQ_NO.remove(&sid);
                            SESSION_MISSING_EVENTS.remove(&sid);
                        });

                        // Set the session ID
                        session_id = resume.session_id.clone();

                        // Set the sequence number
                        let sno = SESSION_SEQ_NO.entry(session_id.clone()).or_insert(RwLock::new(resume.seq));
                        let mut sno = sno.write().await;
                        *sno = resume.seq;
                        drop(sno); // Kill the reference to the sequence number

                        // Start the dispatch manager
                        let (sender, recv) = tokio::sync::mpsc::channel::<QueuedEvent>(512);

                        let Some(mut new_session) = SESSIONS.get_mut(&resume.session_id) else {
                            log::error!("Failed to get session");
                            continue;
                        };

                        new_session.dispatcher = sender;

                        drop(new_session); // Kill the reference to the new session

                        let dpm = tokio::task::spawn(
                            dispatch_manager(Arc::new(session_id.clone()), recv, ws_sender.clone())
                        );

                        curr_tasks.push(dpm);

                        let session_id = session_id.clone();
                        let resume_fanout_task = tokio::spawn(async move {
                            let Some(sess) = SESSIONS.get(&session_id) else {
                                log::error!("Failed to get session");
                                return;
                            };
                            
                            let dispatcher = sess.dispatcher.clone();

                            let Some(mes) = SESSION_MISSING_EVENTS.get(&session_id) else {
                                log::warn!("No missing events queue for shard {}", sess.shard[0]);
                                return;
                            };

                            drop(sess); // Kill the reference to the session

                            let mut missing_events = mes.lock().await;

                            while let Some(event) = missing_events.pop_front() {
                                if dispatcher.is_closed() {
                                    log::warn!("Dispatcher is closed, ending resume fanout");
                                    return;
                                }

                                if let Err(e) = dispatcher.send(QueuedEvent::DispatchValue(Arc::new(event))).await {
                                    log::error!("Failed to send missed event: {}", e);
                                }
                            }

                            drop(missing_events); // Kill the reference to the missing events queue
                        });

                        curr_tasks.push(resume_fanout_task);
                    }
                },
                EventOpCode::Reconnect => continue, // Recieve only event
                EventOpCode::RequestGuildMembers => {
                    let req = serde_json::from_value::<crate::models::GatewayGuildRequestMembers>(event.d);
                    
                    if let Err(e) = req {
                        log::error!("Failed to parse RequestGuildMembers: {}", e);
                        continue;
                    }

                    let req = req.unwrap();

                    if req.query.is_some() && req.user_ids.is_some() {
                        log::error!("RequestGuildMembers query and user_ids are both set, refusing to continue");
                        continue;
                    }

                    let mut chunk_guild_filter = serenity::all::ChunkGuildFilter::None;

                    if req.query.is_some() {
                        chunk_guild_filter = serenity::all::ChunkGuildFilter::Query(req.query.unwrap());
                    }

                    if req.user_ids.is_some() {
                        chunk_guild_filter = serenity::all::ChunkGuildFilter::UserIds(req.user_ids.unwrap());
                    }

                    let runners = cache_http.shard_manager.runners.lock().await;

                    let shard_id = serenity::all::ShardId(session.shard[0]);
                    if let Some(runner) = runners.get(&shard_id) {
                        runner.runner_tx.chunk_guild(
                            req.guild_id, 
                            req.limit, 
                            req.presences.unwrap_or(false), 
                            chunk_guild_filter,
                            req.nonce
                        );
                    }   
                },
                EventOpCode::InvalidSession => continue, // Recieve only event
                EventOpCode::Hello => continue, // Recieve only event
                EventOpCode::HeartbeatAck => continue // Recieve only event
            }
        }
    }

    // Stop the dispatch manager
    for task in curr_tasks {
        task.abort();
    }

    Ok(())
}
/// Animus Magic is the internal redis IPC system for internal communications between the bot and the server
/// 
/// Format of payloads: <scope: u8><cluster id: u16><op: 8 bits><command id: alphanumeric string>/<cbor payload>
use std::sync::Arc;
#[allow(unused_imports)]
use fred::clients::SubscriberClient;
use fred::interfaces::{ClientLike, PubsubInterface};
use serde::{Serialize, Deserialize};
use serenity::all::{GuildId, UserId, RoleId};

#[derive(Serialize, Deserialize, PartialEq)]
pub enum AnimusScope {
    Bot
}

impl AnimusScope {
    pub fn to_byte(&self) -> u8 {
        match self {
            AnimusScope::Bot => 0x0
        }
    }

    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x0 => Some(AnimusScope::Bot),
            _ => None
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq)]
pub enum AnimusOp {
    Request,
    Response,
    Error,
}

impl AnimusOp {
    pub fn to_byte(&self) -> u8 {
        match self {
            AnimusOp::Request => 0x0,
            AnimusOp::Response => 0x1,
            AnimusOp::Error => 0x2,
        }
    }

    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x0 => Some(AnimusOp::Request),
            0x1 => Some(AnimusOp::Response),
            0x2 => Some(AnimusOp::Error),
            _ => None
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AnimusMagicRole {
    pub id: RoleId,
    pub name: String,
    pub position: i16
}

#[derive(Serialize, Deserialize)]
pub enum AnimusResponse {
    /// Modules event contains module related data
    Modules {
        modules: Vec<crate::silverpelt::canonical_repr::modules::CanonicalModule>
    },
    /// GuildsExist event contains a list of u8s, where 1 means the guild exists and 0 means it doesn't
    GuildsExist {
        guilds_exist: Vec<u8>
    },
    /// GetBaseGuildAndUserInfo event is described in AnimusMessage 
    GetBaseGuildAndUserInfo {
        owner_id: String,
        name: String,
        icon: Option<String>,
        /// Format: (role id, role name, index)
        roles: Vec<AnimusMagicRole>,
        /// Bot highest role, same format as roles
        bot_highest: AnimusMagicRole
    }
}

#[derive(Serialize, Deserialize)]
pub enum AnimusMessage {
    /// Ask bot for module data
    Modules {},
    /// Given a list of guild IDs, return whether or not they exist on the bot
    GuildsExist {
        guilds: Vec<GuildId>,
    },
    /// Given a guild ID and a user ID, check:
    /// - The server owner
    /// - The server name
    /// - The server icon
    /// - The users roles
    /// - The bots highest role
    GetBaseGuildAndUserInfo {
        guild_id: GuildId,
        user_id: UserId
    }
}

impl AnimusMessage {
    pub async fn response(&self, cache_http: &crate::impls::cache::CacheHttpImpl) -> Result<AnimusResponse, crate::Error> {
        match self {
            AnimusMessage::Modules {}  => {
                let mut modules = Vec::new();

                for idm in crate::silverpelt::SILVERPELT_CACHE.canonical_module_cache.iter() {
                    let module = idm.value();
                    modules.push(module.clone());
                }

                Ok(AnimusResponse::Modules {
                    modules
                })
            },
            AnimusMessage::GuildsExist { guilds } => {
                let mut guilds_exist = Vec::with_capacity(guilds.len());

                for guild in guilds {
                    guilds_exist.push({
                        if cache_http.cache.guild(*guild).is_some() {
                            1
                        } else {
                            0
                        }
                    });
                }

                Ok(AnimusResponse::GuildsExist {
                    guilds_exist
                })
            },
            AnimusMessage::GetBaseGuildAndUserInfo { guild_id, user_id } => {
                let (name, icon, owner, roles, bot_highest) = {                    
                    let guild = match cache_http.cache.guild(*guild_id) {
                        Some(guild) => guild,
                        None => return Err("Guild not found".into())
                    }.clone();

                    let role_ids = {
                        let mem = match guild.member(cache_http, *user_id).await {
                            Ok(member) => member,
                            Err(e) => return Err(format!("Failed to get member: {}", e).into())
                        };

                        mem.roles.clone()
                    };

                    let mut roles = Vec::new();

                    for role in role_ids.iter() {
                        // Get role from guild.roles
                        match guild.roles.get(role) {
                            Some(role) => {
                                roles.push(AnimusMagicRole {
                                    id: role.id,
                                    name: role.name.to_string(),
                                    position: role.position
                                });
                            }
                            None => {
                                roles.push(AnimusMagicRole {
                                    id: *role,
                                    name: "Unknown".to_string(),
                                    position: -1
                                });
                            }
                        }
                    }

                    let bot_user_id = cache_http.cache.current_user().id;
                    let bot_roles = guild.member(&cache_http, bot_user_id).await?;

                    let mut bot_highest = AnimusMagicRole {
                        id: RoleId::new(0),
                        name: "Unknown".to_string(),
                        position: -1
                    };

                    for role in bot_roles.roles.iter() {
                        // Get role from guild.roles
                        match guild.roles.get(role) {
                            Some(role) => {
                                if role.position > bot_highest.position {
                                    bot_highest = AnimusMagicRole {
                                        id: role.id,
                                        name: role.name.to_string(),
                                        position: role.position
                                    };
                                }
                            }
                            None => {
                                if -1 > bot_highest.position {
                                    bot_highest = AnimusMagicRole {
                                        id: *role,
                                        name: "Unknown".to_string(),
                                        position: -1
                                    };
                                }
                            }
                        }
                    }

                    (guild.name.to_string(), guild.icon_url(), guild.owner_id, roles, bot_highest)
                };

                Ok(AnimusResponse::GetBaseGuildAndUserInfo {
                    name,
                    icon,
                    owner_id: owner.to_string(),
                    roles,
                    bot_highest
                })
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AnimusErrorResponse {
    pub message: String,
    pub context: String
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnimusCreatePayload {
    Response(AnimusResponse),
    Error(AnimusErrorResponse)
}

pub struct AnimusMessageMetadata {
    pub scope: AnimusScope,
    pub cluster_id: u16,
    pub op: AnimusOp,
    pub command_id: String,
    pub payload_offset: usize
}

impl AnimusMessage {
    #[allow(dead_code)]
    pub fn new_command_id() -> String {
        crate::impls::crypto::gen_random(16)
    }

    pub fn create_payload(
        cmd_id: &str,
        scope: AnimusScope,
        op: AnimusOp,
        data: &AnimusCreatePayload
    ) -> Result<Vec<u8>, crate::Error> {
        let mut payload = Vec::new();

        // Push scope as 1 u8
        payload.push(scope.to_byte());

        // Push cluster id as 2 u8s
        let cluster_id = super::argparse::MEWLD_ARGS.cluster_id.to_be_bytes();

        for byte in cluster_id {
            payload.push(byte);
        }

        // Push the op byte
        payload.push(op.to_byte());

        // Push the command id
        for byte in cmd_id.as_bytes() {
            payload.push(*byte);
        }

        // Push seperator of '/'
        payload.push(0x2f);

        // Push the cbor payload
        let v = serde_cbor::to_vec(data)?;

        for byte in v {
            payload.push(byte);
        }

        Ok(payload)
    }

    pub fn get_payload_meta(payload: &[u8]) -> Result<AnimusMessageMetadata, crate::Error> {
        // Take out scope
        let scope = AnimusScope::from_byte(payload[0]).ok_or("Invalid scope byte")?;
       
        // Take out cluster id
        let cluster_id = u16::from_be_bytes([payload[1], payload[2]]);

        let op = AnimusOp::from_byte(payload[3]).ok_or("Invalid op byte")?;

        let mut cmd_id = String::new();

        let mut i = 4;
        loop {
            if payload[i] == 0x2f {
                break;
            }

            cmd_id.push(payload[i] as char);

            i += 1;
        }

        Ok(
            AnimusMessageMetadata {
                scope,
                cluster_id,
                op,
                command_id: cmd_id,
                payload_offset: i + 1
            }
        )
    }

    #[allow(dead_code)]
    pub fn from_payload(payload: &[u8]) -> Result<(Self, AnimusMessageMetadata), crate::Error> {
        let meta = Self::get_payload_meta(payload)?;

        let payload = &payload[meta.payload_offset..];

        // Pluck out json
        let msg = serde_json::from_slice(payload)?;

        Ok((msg, meta))
    }
}

pub struct AnimusMagicClient {
    pub redis_pool: fred::clients::RedisPool,
    pub rx_map: std::sync::Arc<dashmap::DashMap<String, tokio::sync::mpsc::Sender<AnimusResponse>>>,
}

impl AnimusMagicClient {
    /// Starts listening to mewld IPC messages
    /// 
    /// This function never quits once executed
    pub async fn start_ipc_listener(
        &self,
        cache_http: crate::impls::cache::CacheHttpImpl,

        #[allow(unused_variables)] // To be used in the future
        shard_manager: Arc<serenity::all::ShardManager>,    
    ) -> ! {
        // Subscribes to the redis IPC channels we need to subscribe to
        let cfg = self.redis_pool.client_config();

        let subscriber = fred::prelude::Builder::from_config(cfg).build_subscriber_client().unwrap();

        subscriber.connect();
        subscriber.wait_for_connect().await.unwrap();

        let mut message_stream = subscriber.on_message();

        subscriber.manage_subscriptions();

        let _: () = subscriber.subscribe(
            "animus_magic",
        ).await.unwrap();    

        while let Ok(message) = message_stream.recv().await {
            log::debug!("Got message on channel {}", message.channel);
            let binary = match message.value {
                fred::types::RedisValue::Bytes(s) => s,
                fred::types::RedisValue::String(s) => s.into(),
                _ => {
                    log::warn!("Invalid message recieved on channel [wanted binary, but got text] {}", message.channel);
                    continue;
                }
            };

            let cache_http = cache_http.clone();
            let redis_pool = self.redis_pool.clone();

            // Take out cluster ID just to check
            let Ok(meta) = AnimusMessage::get_payload_meta(&binary) else {
                log::warn!("Invalid message recieved on channel {} [metadata extract error]", message.channel);
                continue;
            };

            if meta.op == AnimusOp::Response && meta.cluster_id != super::argparse::MEWLD_ARGS.cluster_id {
                // We have something interesting
                let rx_map = self.rx_map.clone();

                tokio::task::spawn(async move {
                    let sender = rx_map.get(&meta.command_id).map(|s| s.value().clone());

                    if let Some(sender) = sender {
                        let payload = &binary[meta.payload_offset..];

                        // Pluck out json
                        let msg = match serde_json::from_slice::<AnimusResponse>(payload) {
                            Ok(msg) => msg,
                            Err(e) => {
                                log::warn!("Invalid message recieved on channel {} [json extract error] {}", message.channel, e);
                                // Send error
                                if let Err(e) = Self::error(redis_pool, &meta.command_id, AnimusErrorResponse {
                                    message: "Invalid payload, failed to unmarshal message".to_string(),
                                    context: e.to_string()
                                }).await {
                                    log::warn!("Failed to send error response: {}", e);
                                }

                                return;
                            }
                        };

                        if let Err(e) = sender.send(msg).await {
                            log::warn!("Failed to send response to receiver: {}", e);
                        }
                    }
                });
                continue;
            }

            // Ensure requeest op, and that the cluster id is either the same as ours or the wildcard u16::MAX
            if meta.scope != AnimusScope::Bot || meta.op != AnimusOp::Request || (meta.cluster_id != super::argparse::MEWLD_ARGS.cluster_id && meta.cluster_id != u16::MAX) {
                continue; // Not for us
            }

            tokio::spawn(async move {
                let payload = &binary[meta.payload_offset..];

                // Pluck out json
                let msg = match serde_cbor::from_slice::<AnimusMessage>(payload) {
                    Ok(msg) => msg,
                    Err(e) => {
                        log::warn!("Invalid message recieved on channel {} [json extract error] {}", message.channel, e);
                        // Send error
                        if let Err(e) = Self::error(redis_pool, &meta.command_id, AnimusErrorResponse {
                            message: "Invalid payload, failed to unmarshal message".to_string(),
                            context: e.to_string()
                        }).await {
                            log::warn!("Failed to send error response: {}", e);
                        }

                        return;
                    }
                };

                let data = match msg.response(&cache_http).await {
                    Ok(data) => data,
                    Err(e) => {
                        log::warn!("Failed to get response for message on channel {}", message.channel);
                        // Send error
                        if let Err(e) = Self::error(redis_pool, &meta.command_id, AnimusErrorResponse {
                            message: "Failed to create response".to_string(),
                            context: e.to_string()
                        }).await {
                            log::warn!("Failed to send error response: {}", e);
                        }

                        return;
                    }
                };

                let Ok(payload) = AnimusMessage::create_payload(&meta.command_id, AnimusScope::Bot, AnimusOp::Response, &AnimusCreatePayload::Response(data)) else {
                    log::warn!("Failed to create payload for message on channel {}", message.channel);
                    
                    // Send error
                    if let Err(e) = Self::error(redis_pool, &meta.command_id, AnimusErrorResponse {
                        message: "Failed to create response payload".to_string(),
                        context: "create_payload returned Err code".to_string()
                    }).await {
                        log::warn!("Failed to send error response: {}", e);
                    }

                    return;
                };

                // Convert payload to redis value, from here the Error op is useless to try and send
                let payload = fred::types::RedisValue::Bytes(payload.into());

                let conn = redis_pool.next();
                conn.connect();
                let Ok(()) = conn.wait_for_connect().await else {
                    log::warn!("Failed to connect to redis");
                    return;
                };
                match conn.publish("animus_magic", payload).await {
                    Ok(()) => {},
                    Err(e) => {
                        log::warn!("Failed to publish response to redis: {}", e);
                    }   
                }
            });
        }

        unreachable!("IPC listener exited");    
    }

    /// Helper method to send an error response
    pub async fn error(redis_pool: fred::clients::RedisPool, command_id: &str, data: AnimusErrorResponse) -> Result<(), crate::Error> {
        let Ok(payload) = AnimusMessage::create_payload(command_id, AnimusScope::Bot, AnimusOp::Error, &AnimusCreatePayload::Error(data)) else {
            return Err("Failed to create payload for error message".into());
        };

        // Convert payload to redis value
        let payload = fred::types::RedisValue::Bytes(payload.into());

        let conn = redis_pool.next();
        conn.connect();
        let Ok(()) = conn.wait_for_connect().await else {
            return Err("Failed to connect to redis".into());
        };
        match conn.publish("animus_magic", payload).await {
            Ok(()) => Ok(()),
            Err(e) => Err(format!("Failed to publish response to redis: {}", e).into())
        }
    }
}
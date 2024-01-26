/// Animus Magic is the internal redis IPC system for internal communications between the bot and the server
/// 
/// Format of payloads: <cluster id: u16><op: 8 bits><command id: alphanumeric string>/<json payload>
use std::sync::Arc;
#[allow(unused_imports)]
use fred::clients::SubscriberClient;
use fred::interfaces::{ClientLike, PubsubInterface};
use serde::{Serialize, Deserialize};
use serenity::all::GuildId;

#[derive(Serialize, Deserialize, PartialEq)]
pub enum AnimusOp {
    Request,
    Response
}

impl AnimusOp {
    pub fn to_byte(&self) -> u8 {
        match self {
            AnimusOp::Request => 0x0,
            AnimusOp::Response => 0x1
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum AnimusResponse {
    /// Modules event contains module related data
    Modules {
        modules: indexmap::IndexMap<String, crate::silverpelt::canonical_repr::CanonicalModule>
    },
    GuildsExist {
        guilds_exist: Vec<u8>
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
}

pub struct AnimusMessageMetadata {
    pub cluster_id: u16,
    pub op: AnimusOp,
    pub command_id: String,
    pub payload_offset: usize
}

impl AnimusMessage {
    pub fn new_command_id() -> String {
        crate::impls::crypto::gen_random(16)
    }

    pub fn create_payload(
        cmd_id: &str,
        op: AnimusOp,
        data: &serde_json::Value
    ) -> Result<Vec<u8>, crate::Error> {
        let mut payload = Vec::new();

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

        // Push the json payload
        let json_v = serde_json::to_vec(data)?;

        for byte in json_v {
            payload.push(byte);
        }

        Ok(payload)
    }

    pub fn get_payload_meta(payload: &[u8]) -> Result<AnimusMessageMetadata, crate::Error> {
        // Take out cluster id
        let cluster_id = u16::from_be_bytes([payload[0], payload[1]]);

        let op = match payload[2] {
            0x0 => AnimusOp::Request,
            0x1 => AnimusOp::Response,
            _ => return Err("Invalid op byte".into())
        };

        let mut cmd_id = String::new();

        let mut i = 3;
        loop {
            if payload[i] == 0x2f {
                break;
            }

            cmd_id.push(payload[i] as char);

            i += 1;
        }

        Ok(
            AnimusMessageMetadata {
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

            // Ensure requeest op, and that the cluster id is either the same as ours or the wildcard u16::MAX
            if meta.op != AnimusOp::Request || (meta.cluster_id != super::argparse::MEWLD_ARGS.cluster_id && meta.cluster_id != u16::MAX) {
                continue; // Not for us
            }

            tokio::spawn(async move {
                let payload = &binary[meta.payload_offset..];

                // Pluck out json
                let msg = match serde_json::from_slice::<AnimusMessage>(payload) {
                    Ok(msg) => msg,
                    Err(e) => {
                        log::warn!("Invalid message recieved on channel {} [json extract error] {}", message.channel, e);
                        return;
                    }
                };

                let data = match msg {
                    AnimusMessage::Modules {}  => {
                        let mut modules = indexmap::IndexMap::new();

                        for (id, module) in crate::silverpelt::CANONICAL_MODULE_CACHE.iter() {
                            modules.insert(id.to_string(), module.clone());
                        }

                        AnimusResponse::Modules {
                            modules
                        }
                    },
                    AnimusMessage::GuildsExist { guilds } => {
                        let mut guilds_exist = Vec::with_capacity(guilds.len());

                        for guild in guilds {
                            guilds_exist.push({
                                if cache_http.cache.guild(guild).is_some() {
                                    1
                                } else {
                                    0
                                }
                            });
                        }

                        AnimusResponse::GuildsExist {
                            guilds_exist
                        }
                    }
                };

                let Ok(data) = serde_json::to_value(data) else {
                    log::warn!("Failed to serialize response for message on channel {}", message.channel);
                    return;
                };

                let Ok(payload) = AnimusMessage::create_payload(&AnimusMessage::new_command_id(), AnimusOp::Response, &data) else {
                    log::warn!("Failed to create payload for message on channel {}", message.channel);
                    return;
                };

                let conn = redis_pool.next();
                let Ok(()) = conn.publish(super::argparse::MEWLD_ARGS.mewld_redis_channel.clone(), payload).await else {
                    log::warn!("Failed to publish response to channel {}", message.channel);
                    return;
                };
            });
        }

        unreachable!("IPC listener exited");    
    }
}
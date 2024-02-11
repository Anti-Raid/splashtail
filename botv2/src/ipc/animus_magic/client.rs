/// Animus Magic is the internal redis IPC system for internal communications between the bot and the server
/// 
/// Format of payloads: <target [from]: u8><target [to]: u8><cluster id: u16><op: 8 bits><command id: alphanumeric string>/<cbor payload>
use std::sync::Arc;
use fred::{types::RedisValue, clients::RedisPool, prelude::Builder, interfaces::{ClientLike, PubsubInterface}};
use serde::{Serialize, Deserialize};
use super::bot::{BotAnimusResponse, BotAnimusMessage};
use super::jobserver::{JobserverAnimusMessage, JobserverAnimusResponse};
use crate::{Error, ipc::argparse::MEWLD_ARGS};

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum AnimusTarget {
    Bot,
    Jobserver,
    Webserver,
    Wildcard,
}

impl AnimusTarget {
    pub fn to_byte(self) -> u8 {
        match self {
            AnimusTarget::Bot => 0x0,
            AnimusTarget::Jobserver => 0x1,
            AnimusTarget::Webserver => 0x2,
            AnimusTarget::Wildcard => u8::MAX,
        }
    }

    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x0 => Some(AnimusTarget::Bot),
            0x1 => Some(AnimusTarget::Jobserver),
            0x2 => Some(AnimusTarget::Webserver),
            u8::MAX => Some(AnimusTarget::Wildcard),
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
pub struct AnimusErrorResponse {
    pub message: String,
    pub context: String
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnimusResponse {
    Bot(BotAnimusResponse),
    Jobserver(JobserverAnimusResponse),
    Error(AnimusErrorResponse),
}

impl AnimusResponse {
    pub fn from_payload(payload: &[u8], op: AnimusOp, target: AnimusTarget) -> Result<Self, crate::Error> {
        if op == AnimusOp::Error {
            let msg = serde_cbor::from_slice::<AnimusErrorResponse>(payload)?;
            return Ok(Self::Error(msg));
        }
        
        match target {
            AnimusTarget::Bot => {
                let msg = serde_cbor::from_slice::<BotAnimusResponse>(payload)?;
                Ok(Self::Bot(msg))
            }

            AnimusTarget::Jobserver => {
                let msg = serde_cbor::from_slice::<JobserverAnimusResponse>(payload)?;
                Ok(Self::Jobserver(msg))
            }

            _ => Err("Invalid target for payload".into())
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnimusMessage {
    Bot(BotAnimusMessage),
    Jobserver(JobserverAnimusMessage),
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnimusPayload {
    Message(AnimusMessage),
    Response(AnimusResponse),   
}

impl From<AnimusMessage> for AnimusPayload {
    fn from(msg: AnimusMessage) -> Self {
        Self::Message(msg)
    }
}

impl From<AnimusResponse> for AnimusPayload {
    fn from(resp: AnimusResponse) -> Self {
        Self::Response(resp)
    }
}

impl AnimusMessage {
    pub fn from_payload(payload: &[u8], target: AnimusTarget) -> Result<Self, crate::Error> {        
        match target {
            AnimusTarget::Bot => {
                let msg = serde_cbor::from_slice::<BotAnimusMessage>(payload)?;
                Ok(Self::Bot(msg))
            }

            AnimusTarget::Jobserver => {
                let msg = serde_cbor::from_slice::<JobserverAnimusMessage>(payload)?;
                Ok(Self::Jobserver(msg))
            }

            _ => Err("Invalid target for payload".into())
        }
    }
}

pub struct AnimusMessageMetadata {
    pub from: AnimusTarget,
    pub to: AnimusTarget,
    pub cluster_id: u16,
    pub op: AnimusOp,
    pub command_id: String,
    pub payload_offset: usize
}

pub struct AnimusMagicClient {
    pub redis_pool: RedisPool,
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

        let subscriber = Builder::from_config(cfg).build_subscriber_client().unwrap();

        subscriber.connect();
        subscriber.wait_for_connect().await.unwrap();

        let mut message_stream = subscriber.on_message();

        subscriber.manage_subscriptions();

        let _: () = subscriber.subscribe(
            MEWLD_ARGS.animus_magic_channel.as_str(),
        ).await.unwrap();    

        while let Ok(message) = message_stream.recv().await {
            log::debug!("Got message on channel {}", message.channel);
            let binary = match message.value {
                RedisValue::Bytes(s) => s,
                RedisValue::String(s) => s.into(),
                _ => {
                    log::warn!("Invalid message recieved on channel [wanted binary, but got text] {}", message.channel);
                    continue;
                }
            };

            // Parse the payload
            let Ok(meta) = Self::get_payload_meta(&binary) else {
                log::warn!("Invalid message recieved on channel {} [metadata extract error]", message.channel);
                continue;
            };

            // Case of response
            match meta.op {
                AnimusOp::Response | AnimusOp::Error => {
                    if meta.from == AnimusTarget::Bot && (meta.cluster_id != MEWLD_ARGS.cluster_id && meta.cluster_id != u16::MAX) {
                        continue; // Not for us
                    }

                    let rx_map = self.rx_map.clone();

                    let redis_pool = self.redis_pool.clone();

                    tokio::task::spawn(async move {
                        let sender = rx_map.get(&meta.command_id).map(|s| s.value().clone());

                        if let Some(sender) = sender {
                            let payload = &binary[meta.payload_offset..];

                            let resp = match AnimusResponse::from_payload(payload, meta.op, meta.from) {
                                Ok(resp) => resp,
                                Err(e) => {
                                    log::warn!("Invalid message recieved on channel {} [response extract error] {}", message.channel, e);
                                    // Send error
                                    if let Err(e) = Self::error(&redis_pool, &meta.command_id, AnimusErrorResponse {
                                        message: "Invalid payload, failed to unmarshal message".to_string(),
                                        context: e.to_string()
                                    }, meta.from).await {
                                        log::warn!("Failed to send error response: {}", e);
                                    }

                                    return;
                                }
                            };

                            if let Err(e) = sender.send(resp).await {
                                rx_map.remove(&meta.command_id);
                                log::warn!("Failed to send response to receiver: {}", e);
                            }

                            rx_map.remove(&meta.command_id);
                        }
                    });
                }

                AnimusOp::Request => {
                    // Ensure requeest op, and that the cluster id is either the same as ours or the wildcard u16::MAX
                    if meta.to != AnimusTarget::Bot && meta.to != AnimusTarget::Wildcard {
                        continue; // Not for us, to != Bot and != wildcard
                    }

                    if meta.cluster_id == MEWLD_ARGS.cluster_id || meta.cluster_id != u16::MAX {
                        continue; // Not for us, cluster_id != ours and != wildcard
                    }

                    let cache_http = cache_http.clone();
                    let redis_pool = self.redis_pool.clone();

                    tokio::spawn(async move {
                        let payload = &binary[meta.payload_offset..];

                        // Pluck out json
                        let resp = match AnimusMessage::from_payload(payload, AnimusTarget::Bot) {
                            Ok(resp) => resp,
                            Err(e) => {
                                log::warn!("Invalid message recieved on channel {} [request extract error] {}", message.channel, e);
                                // Send error
                                if let Err(e) = Self::error(&redis_pool, &meta.command_id, AnimusErrorResponse {
                                    message: "Invalid payload, failed to unmarshal message".to_string(),
                                    context: e.to_string()
                                }, meta.from).await {
                                    log::warn!("Failed to send error response: {}", e);
                                }

                                return;
                            }
                        };
                        
                        let msg = match resp {
                            AnimusMessage::Bot(msg) => msg,
                            AnimusMessage::Jobserver(_) => {
                                log::warn!("Invalid message recieved on channel {} [invalid message type]", message.channel);
                                // Send error
                                if let Err(e) = Self::error(&redis_pool, &meta.command_id, AnimusErrorResponse {
                                    message: "Invalid payload, failed to unmarshal message".to_string(),
                                    context: "Invalid message type".to_string()
                                }, meta.from).await {
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
                                if let Err(e) = Self::error(&redis_pool, &meta.command_id, AnimusErrorResponse {
                                    message: "Failed to create response".to_string(),
                                    context: e.to_string()
                                }, meta.from).await {
                                    log::warn!("Failed to send error response: {}", e);
                                }

                                return;
                            }
                        };

                        let Ok(payload) = Self::create_payload(&meta.command_id, AnimusTarget::Bot, meta.from, AnimusOp::Response, &AnimusResponse::Bot(data).into()) else {
                            log::warn!("Failed to create payload for message on channel {}", message.channel);
                            
                            // Send error
                            if let Err(e) = Self::error(&redis_pool, &meta.command_id, AnimusErrorResponse {
                                message: "Failed to create response payload".to_string(),
                                context: "create_payload returned Err code".to_string()
                            }, meta.from).await {
                                log::warn!("Failed to send error response: {}", e);
                            }

                            return;
                        };

                        if let Err(e) = Self::publish(&redis_pool, payload).await {
                            log::warn!("Failed to publish response to redis: {}", e);

                            // Send error
                            if let Err(e) = Self::error(&redis_pool, &meta.command_id, AnimusErrorResponse {
                                message: "Failed to publish response to redis".to_string(),
                                context: e.to_string()
                            }, meta.from).await {
                                log::warn!("Failed to send error response: {}", e);
                            }
                        }
                    });
                }
            }      
        }

        unreachable!("IPC listener exited");    
    }

    /// Helper method to send an error response
    pub async fn error(redis_pool: &RedisPool, command_id: &str, data: AnimusErrorResponse, to: AnimusTarget) -> Result<(), crate::Error> {
        let Ok(payload) = Self::create_payload(command_id, AnimusTarget::Bot, to, AnimusOp::Error, &AnimusResponse::Error(data).into()) else {
            return Err("Failed to create payload for error message".into());
        };

        Self::publish(redis_pool, payload).await
    }

    /// Helper method to send a response
    pub async fn publish(redis_pool: &RedisPool, payload: Vec<u8>) -> Result<(), Error> {
        // Convert payload to redis value
        let payload = RedisValue::Bytes(payload.into());

        let conn = redis_pool.next();
        conn.connect();
        let Ok(()) = conn.wait_for_connect().await else {
            return Err("Failed to connect to redis".into());
        };
        match conn.publish(MEWLD_ARGS.animus_magic_channel.as_str(), payload).await {
            Ok(()) => Ok(()),
            Err(e) => Err(format!("Failed to publish response to redis: {}", e).into())
        }
    }

    /// request creates a new request and waits for a response until either timeout or response
    pub async fn request(&self, target: AnimusTarget, msg: AnimusMessage) -> Result<AnimusResponse, crate::Error> {
        let cmd_id = Self::new_command_id();

        let payload = match Self::create_payload(&cmd_id, AnimusTarget::Bot, target, AnimusOp::Request, &msg.into()) {
            Ok(payload) => payload,
            Err(e) => return Err(e)
        };

        let (tx, mut rx) = tokio::sync::mpsc::channel(1);

        self.rx_map.insert(cmd_id.clone(), tx);

        Self::publish(&self.redis_pool, payload).await?;

        let resp = match tokio::time::timeout(std::time::Duration::from_secs(10), rx.recv()).await {
            Ok(resp) => resp,
            Err(_) => return Err("Request timed out".into())
        };

        match resp {
            Some(resp) => Ok(resp),
            None => Err("Failed to get response".into())
        }
    }

    #[allow(dead_code)]
    pub fn new_command_id() -> String {
        crate::impls::crypto::gen_random(16)
    }

    /// Creates a payload 
    pub fn create_payload(
        cmd_id: &str,
        from: AnimusTarget,
        to: AnimusTarget,
        op: AnimusOp,
        data: &AnimusPayload
    ) -> Result<Vec<u8>, crate::Error> {
        let mut payload = Vec::new();

        // Push from as 1 u8
        payload.push(from.to_byte());

        // Push to as 1 u8
        payload.push(to.to_byte());

        // Push cluster id as 2 u8s
        let cluster_id = MEWLD_ARGS.cluster_id.to_be_bytes();

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

    // Parses the metadata of a payload
    pub fn get_payload_meta(payload: &[u8]) -> Result<AnimusMessageMetadata, crate::Error> {
        // Take out scope
        let from = AnimusTarget::from_byte(payload[0]).ok_or("Invalid from byte")?;
        
        // Take out scope
        let to = AnimusTarget::from_byte(payload[1]).ok_or("Invalid type byte")?;

        // Take out cluster id
        let cluster_id = u16::from_be_bytes([payload[2], payload[3]]);

        let op = AnimusOp::from_byte(payload[4]).ok_or("Invalid op byte")?;

        let mut cmd_id = String::new();

        let mut i = 5;
        loop {
            if payload[i] == 0x2f {
                break;
            }

            cmd_id.push(payload[i] as char);

            i += 1;
        }

        Ok(
            AnimusMessageMetadata {
                from,
                to,
                cluster_id,
                op,
                command_id: cmd_id,
                payload_offset: i + 1
            }
        )
    }
}
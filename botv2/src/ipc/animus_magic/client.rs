use super::bot::{BotAnimusMessage, BotAnimusResponse};
use super::jobserver::{JobserverAnimusMessage, JobserverAnimusResponse};
use crate::{ipc::argparse::MEWLD_ARGS, Error};
use splashcore_rs::animusmagic_protocol::{
    AnimusOp, 
    AnimusTarget, 
    AnimusErrorResponse,
    create_payload,
    from_payload,
    get_payload_meta,
};
use splashcore_rs::animusmagic_ext::AnimusMagicClientExt;
use fred::{
    clients::{RedisClient, RedisPool},
    interfaces::{ClientLike, EventInterface, PubsubInterface},
    prelude::Builder,
    types::RedisValue,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnimusResponse {
    Bot(BotAnimusResponse),
    Jobserver(JobserverAnimusResponse),
}

impl AnimusResponse {
    pub fn from_payload(payload: &[u8], target: AnimusTarget) -> Result<Self, crate::Error> {
        match target {
            AnimusTarget::Bot => {
                let bar = from_payload::<BotAnimusResponse>(payload);

                match bar {
                    Ok(bar) => Ok(AnimusResponse::Bot(bar)),
                    Err(e) => Err(format!("Failed to unmarshal message: {}", e).into()),
                }
            }
            AnimusTarget::Jobserver => {
                let jar = from_payload::<JobserverAnimusResponse>(payload);

                match jar {
                    Ok(jar) => Ok(AnimusResponse::Jobserver(jar)),
                    Err(e) => Err(format!("Failed to unmarshal message: {}", e).into()),
                }
            }
            _ => Err("Invalid target".into()),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnimusMessage {
    Bot(BotAnimusMessage),
    Jobserver(JobserverAnimusMessage),
}

impl AnimusMessage {
    pub fn from_payload(payload: &[u8], target: AnimusTarget) -> Result<Self, crate::Error> {
        match target {
            AnimusTarget::Bot => {
                let bar = from_payload::<BotAnimusMessage>(payload);

                match bar {
                    Ok(bar) => Ok(AnimusMessage::Bot(bar)),
                    Err(e) => Err(format!("Failed to unmarshal message: {}", e).into()),
                }
            }
            AnimusTarget::Jobserver => {
                let jar = from_payload::<JobserverAnimusMessage>(payload);

                match jar {
                    Ok(jar) => Ok(AnimusMessage::Jobserver(jar)),
                    Err(e) => Err(format!("Failed to unmarshal message: {}", e).into()),
                }
            }
            _ => Err("Invalid target".into()),
        }
    }
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

pub struct AnimusMagicClient {
    pub redis_pool: RedisPool,
    pub rx_map: std::sync::Arc<dashmap::DashMap<String, tokio::sync::mpsc::Sender<AnimusResponse>>>,
    pub err_map: std::sync::Arc<dashmap::DashMap<String, tokio::sync::mpsc::Sender<AnimusErrorResponse>>>,
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

        self.redis_pool.connect_pool();

        let mut message_stream = subscriber.message_rx();

        subscriber.manage_subscriptions();

        let _: () = subscriber
            .subscribe(MEWLD_ARGS.animus_magic_channel.as_str())
            .await
            .unwrap();

        while let Ok(message) = message_stream.recv().await {
            log::debug!("Got message on channel {}", message.channel);
            let binary = match message.value {
                RedisValue::Bytes(s) => s,
                RedisValue::String(s) => s.into(),
                _ => {
                    log::warn!(
                        "Invalid message recieved on channel [wanted binary, but got text] {}",
                        message.channel
                    );
                    continue;
                }
            };

            // Parse the payload
            let Ok(meta) = get_payload_meta(&binary) else {
                log::warn!(
                    "Invalid message recieved on channel {} [metadata extract error]",
                    message.channel
                );
                continue;
            };

            // Case of response
            match meta.op {
                AnimusOp::Error => {
                    if meta.from == AnimusTarget::Bot
                        && (meta.cluster_id != MEWLD_ARGS.cluster_id && meta.cluster_id != u16::MAX)
                    {
                        continue; // Not for us
                    }

                    let err_map = self.err_map.clone();

                    tokio::task::spawn(async move {
                        let sender = err_map.get(&meta.command_id).map(|s| s.value().clone());

                        if let Some(sender) = sender {
                            let payload = &binary[meta.payload_offset..];

                            let resp = match from_payload::<AnimusErrorResponse>(payload) {
                                Ok(resp) => resp,
                                Err(e) => {
                                    log::warn!(
                                        "Invalid message recieved on channel {} [response extract error] {}",
                                        message.channel,
                                        e
                                    );
                                    return;
                                }
                            };

                            if let Err(e) = sender.send(resp).await {
                                err_map.remove(&meta.command_id);
                                log::warn!("Failed to send response to receiver: {}", e);
                            }

                            err_map.remove(&meta.command_id);
                        }
                    });
                },
                AnimusOp::Response => {
                    if meta.from == AnimusTarget::Bot
                        && (meta.cluster_id != MEWLD_ARGS.cluster_id && meta.cluster_id != u16::MAX)
                    {
                        continue; // Not for us
                    }

                    let rx_map = self.rx_map.clone();

                    tokio::task::spawn(async move {
                        let sender = rx_map.get(&meta.command_id).map(|s| s.value().clone());

                        if let Some(sender) = sender {
                            let payload = &binary[meta.payload_offset..];

                            let resp = match AnimusResponse::from_payload(payload, meta.from) {
                                Ok(resp) => resp,
                                Err(e) => {
                                    log::warn!(
                                        "Invalid message recieved on channel {} [response extract error] {}",
                                        message.channel,
                                        e
                                    );
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

                AnimusOp::Request | AnimusOp::Probe => {
                    // Ensure requeest op, and that the cluster id is either the same as ours or the wildcard u16::MAX
                    if meta.to != AnimusTarget::Bot && meta.to != AnimusTarget::Wildcard {
                        continue; // Not for us, to != Bot and != wildcard
                    }

                    if meta.cluster_id != MEWLD_ARGS.cluster_id && meta.cluster_id != u16::MAX {
                        continue; // Not for us, cluster_id != ours and != wildcard
                    }

                    if meta.op == AnimusOp::Probe {
                        // Send probe response
                        let redis_pool = self.redis_pool.clone();

                        tokio::spawn(async move {
                            let Ok(payload) = create_payload::<AnimusErrorResponse>(
                                &meta.command_id,
                                AnimusTarget::Bot,
                                MEWLD_ARGS.cluster_id,
                                meta.from,
                                AnimusOp::Response,
                                &AnimusErrorResponse {
                                    message: "Pong".to_string(),
                                    context: "".to_string(),
                                },
                            ) else {
                                log::warn!(
                                    "Failed to create payload for message on channel {}",
                                    message.channel
                                );
                                return;
                            };

                            if let Err(e) = Self::publish(redis_pool.next(), payload).await {
                                log::warn!("Failed to publish response to redis: {}", e);
                            }
                        });

                        continue;
                    }

                    let cache_http = cache_http.clone();
                    let redis_pool = self.redis_pool.clone();

                    let client = AnimusMagicClient {
                        redis_pool: self.redis_pool.clone(),
                        rx_map: self.rx_map.clone(),
                        err_map: self.err_map.clone(),
                    };
                    
                    tokio::spawn(async move {
                        let payload = &binary[meta.payload_offset..];

                        // Pluck out json
                        let resp = match AnimusMessage::from_payload(payload, AnimusTarget::Bot) {
                            Ok(resp) => resp,
                            Err(e) => {
                                log::warn!("Invalid message recieved on channel {} [request extract error] {}", message.channel, e);
                                // Send error
                                if let Err(e) = client.error(
                                    &meta.command_id,
                                    AnimusErrorResponse {
                                        message: "Invalid payload, failed to unmarshal message"
                                            .to_string(),
                                        context: e.to_string(),
                                    },
                                    meta.from,
                                )
                                .await
                                {
                                    log::warn!("Failed to send error response: {}", e);
                                }

                                return;
                            }
                        };

                        let msg = match resp {
                            AnimusMessage::Bot(msg) => msg,
                            AnimusMessage::Jobserver(_) => {
                                log::warn!(
                                    "Invalid message recieved on channel {} [invalid message type]",
                                    message.channel
                                );
                                // Send error
                                if let Err(e) = client.error(
                                    &meta.command_id,
                                    AnimusErrorResponse {
                                        message: "Invalid payload, failed to unmarshal message"
                                            .to_string(),
                                        context: "Invalid message type".to_string(),
                                    },
                                    meta.from,
                                )
                                .await
                                {
                                    log::warn!("Failed to send error response: {}", e);
                                }

                                return;
                            }
                        };

                        let data = match msg.response(&cache_http).await {
                            Ok(data) => data,
                            Err(e) => {
                                log::warn!(
                                    "Failed to get response for message on channel {}",
                                    message.channel
                                );
                                // Send error
                                if let Err(e) = client.error(
                                    &meta.command_id,
                                    AnimusErrorResponse {
                                        message: "Failed to create response".to_string(),
                                        context: e.to_string(),
                                    },
                                    meta.from,
                                )
                                .await
                                {
                                    log::warn!("Failed to send error response: {}", e);
                                }

                                return;
                            }
                        };

                        let Ok(payload) = create_payload::<AnimusResponse>(
                            &meta.command_id,
                            AnimusTarget::Bot,
                            MEWLD_ARGS.cluster_id,
                            meta.from,
                            AnimusOp::Response,
                            &AnimusResponse::Bot(data),
                        ) else {
                            log::warn!(
                                "Failed to create payload for message on channel {}",
                                message.channel
                            );

                            // Send error
                            if let Err(e) = client.error(
                                &meta.command_id,
                                AnimusErrorResponse {
                                    message: "Failed to create response payload".to_string(),
                                    context: "create_payload returned Err code".to_string(),
                                },
                                meta.from,
                            )
                            .await
                            {
                                log::warn!("Failed to send error response: {}", e);
                            }

                            return;
                        };

                        if let Err(e) = Self::publish(redis_pool.next(), payload).await {
                            log::warn!("Failed to publish response to redis: {}", e);

                            // Send error
                            if let Err(e) = client.error(
                                &meta.command_id,
                                AnimusErrorResponse {
                                    message: "Failed to publish response to redis".to_string(),
                                    context: e.to_string(),
                                },
                                meta.from,
                            )
                            .await
                            {
                                log::warn!("Failed to send error response: {}", e);
                            }
                        }
                    });
                }
            }
        }

        unreachable!("IPC listener exited");
    }

    /// Helper method to send a response
    pub async fn publish(redis_conn: &RedisClient, payload: Vec<u8>) -> Result<(), Error> {
        // Convert payload to redis value
        let payload = RedisValue::Bytes(payload.into());

        match redis_conn
            .publish(MEWLD_ARGS.animus_magic_channel.as_str(), payload)
            .await
        {
            Ok(()) => Ok(()),
            Err(e) => Err(format!("Failed to publish response to redis: {}", e).into()),
        }
    }
}

impl AnimusMagicClientExt<AnimusResponse> for AnimusMagicClient {
    fn error_map(&self) -> Arc<dashmap::DashMap<String, tokio::sync::mpsc::Sender<AnimusErrorResponse>>> {
        self.err_map.clone()
    }

    fn rx_map(&self) -> Arc<dashmap::DashMap<String, tokio::sync::mpsc::Sender<AnimusResponse>>> {
        self.rx_map.clone()
    }

    fn from(&self) -> AnimusTarget {
        AnimusTarget::Bot
    }

    fn cluster_id(&self) -> u16 {
        MEWLD_ARGS.cluster_id
    }

    async fn publish_next(&self, payload: Vec<u8>) -> Result<(), Error> {
        Self::publish(self.redis_pool.next(), payload).await
    }
}
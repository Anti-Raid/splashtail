use bothelpers::cache::CacheHttpImpl;
#[allow(unused_imports)]
use fred::clients::SubscriberClient;
use fred::interfaces::{ClientLike, EventInterface, PubsubInterface};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// This is the fundemental primitive for mewld IPC
pub struct MewldIpcClient {
    pub redis_pool: fred::clients::RedisPool,
    pub cache: Arc<MewldIpcCache>,
    pub pool: sqlx::PgPool,
}

#[derive(Default)]
pub struct MewldIpcCache {
    /// Stores the health of all clusters
    pub cluster_healths: Arc<dashmap::DashMap<u16, Vec<MewldDiagShardHealth>>>,
    pub all_clusters_up: Arc<tokio::sync::RwLock<bool>>,
}

impl MewldIpcCache {
    /// Returns the true total number of servers the bot has access to
    pub fn total_guilds(&self) -> u64 {
        let mut total = 0;

        for mp in self.cluster_healths.iter() {
            for shard in mp.value() {
                total += shard.guilds;
            }
        }

        total
    }

    /// Returns the true total number of users the bot has access to
    pub fn total_users(&self) -> u64 {
        let mut total = 0;

        for mp in self.cluster_healths.iter() {
            for shard in mp.value() {
                total += shard.users;
            }
        }

        total
    }

    /// Returns whether or not all clusters are up
    pub async fn all_clusters_up(&self) -> bool {
        *self.all_clusters_up.read().await
    }
}

/*
Scope     string         `json:"scope"`
    Action    string         `json:"action"`
    Args      map[string]any `json:"args,omitempty"`
    CommandId string         `json:"command_id,omitempty"`
    Output    any            `json:"output,omitempty"`
    Data      map[string]any `json:"data,omitempty"` // Used in action logs */

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherCmd {
    scope: String,
    action: String,
    args: Option<serde_json::Value>,
    command_id: Option<String>,
    output: Option<serde_json::Value>,
    data: Option<serde_json::Value>,
}

/*

Mewld structures

// Internal payload for diagnostics
type diagPayload struct {
    ClusterID int    `json:"id"`    // The cluster ID
    Nonce     string `json:"nonce"` // Random nonce sent that is used to validate that a nonce comes from a specific diag request
    Diag      bool   `json:"diag"`  // Whether or not this is a diag request, is always true in this struct
}

type ShardHealth struct {
    ShardID uint64  `json:"shard_id"` // The shard ID
    Up      bool    `json:"up"`       // Whether or not the shard is up
    Latency float64 `json:"latency"`  // Latency of the shard (optional, send if possible)
    Guilds  uint64  `json:"guilds"`   // The number of guilds in the shard. Is optional
    Users   uint64  `json:"users"`    // The number of users in the shard. Is optional
}

type DiagResponse struct {
    Nonce string        // Random nonce used to validate that a nonce comes from a specific diag request
    Data  []ShardHealth // The shard health data
}
 */

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MewldDiagPayload {
    #[serde(rename = "id")]
    cluster_id: u16,
    nonce: String,
    diag: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MewldDiagResponse {
    cluster_id: u16,
    /// This is not part of the mewld response protocol, but its useful for statistics and mewld will ignore it anyways
    #[serde(rename = "Nonce")]
    nonce: String,
    #[serde(rename = "Data")]
    data: Vec<MewldDiagShardHealth>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MewldDiagShardHealth {
    pub shard_id: u16,
    pub up: bool,
    pub latency: f64,
    pub guilds: u64,
    pub users: u64,
}

impl MewldIpcClient {
    /// Starts listening to mewld IPC messages
    ///
    /// This function never quits once executed
    pub async fn start_ipc_listener(
        &self,
        serenity_cache: &CacheHttpImpl,
        shard_manager: &Arc<serenity::all::ShardManager>,
    ) -> ! {
        // Subscribes to the redis IPC channels we need to subscribe to
        let cfg = self.redis_pool.client_config();

        let subscriber = fred::prelude::Builder::from_config(cfg)
            .build_subscriber_client()
            .unwrap();

        subscriber.connect();
        subscriber.wait_for_connect().await.unwrap();

        let mut message_stream = subscriber.message_rx();

        // Subscribe to the mewld channel
        // There are two channels we need to subscribe to:
        // 1. The actual mewld channel
        // 2. A private 'mailbox' channel
        //
        // Note that jobserver will use HTTP for communication
        subscriber.manage_subscriptions();

        let _: () = subscriber
            .subscribe(super::argparse::MEWLD_ARGS.mewld_redis_channel.clone())
            .await
            .unwrap();
        let _: () = subscriber
            .subscribe(&format!(
                "{}:{}",
                super::argparse::MEWLD_ARGS.mewld_redis_channel,
                super::argparse::MEWLD_ARGS.cluster_id
            ))
            .await
            .unwrap();

        while let Ok(message) = message_stream.recv().await {
            log::debug!(
                "Recieved message {:#?} on channel {}",
                message.value,
                message.channel
            );

            let strvalue = match message.value {
                fred::types::RedisValue::String(s) => s,
                _ => {
                    log::warn!("Invalid message recieved on channel {}", message.channel);
                    continue;
                }
            };

            if strvalue.is_empty() || !strvalue.starts_with('{') {
                log::warn!(
                    "Invalid message recieved on channel {} [earlyopt: not a json object]",
                    message.channel
                );
                continue;
            }

            // Decode to a serde_json::Value
            let value: serde_json::Value = match serde_json::from_str(&strvalue) {
                Ok(v) => v,
                Err(e) => {
                    log::warn!(
                        "Invalid message recieved on channel {}: {} [cannot deserialize]",
                        message.channel,
                        e
                    );
                    continue;
                }
            };

            let obj = match value {
                serde_json::Value::Object(ref obj) => obj,
                _ => {
                    log::warn!(
                        "Invalid message recieved on channel {} [not an object]",
                        message.channel
                    );
                    continue;
                }
            };

            // Check if this is a mewld diag payload
            if obj.contains_key("diag") {
                let diag_payload = match serde_json::from_value::<MewldDiagPayload>(value) {
                    Ok(v) => v,
                    Err(e) => {
                        log::warn!("Invalid message recieved on channel {}: {} [diag key set but not a diag payload]", message.channel, e);
                        continue;
                    }
                };

                if diag_payload.cluster_id != super::argparse::MEWLD_ARGS.cluster_id {
                    // Not for us
                    continue;
                }

                if diag_payload.nonce.is_empty() {
                    log::warn!(
                        "Invalid diag payload recieved on channel {}: nonce is empty",
                        message.channel
                    );
                    continue;
                }

                // Collect shard health
                let mut guild_counts_per_shard = std::collections::HashMap::new();
                let mut user_counts_per_shard = std::collections::HashMap::new();

                for guild in serenity_cache.cache.guilds() {
                    let shard_id =
                        serenity::utils::shard_id(guild, super::argparse::MEWLD_ARGS.shard_count);

                    let count = guild_counts_per_shard.entry(shard_id).or_insert(0);
                    *count += 1;

                    {
                        let guild = guild.to_guild_cached(&serenity_cache.cache);

                        if let Some(guild) = guild {
                            let count = user_counts_per_shard.entry(shard_id).or_insert(0);
                            *count += guild.member_count;
                        }
                    }
                }

                let mut shard_healths = vec![];

                for (shard_id, shard) in shard_manager.runners.lock().await.iter() {
                    let shard_health = MewldDiagShardHealth {
                        shard_id: shard_id.0,
                        up: shard.stage == serenity::all::ConnectionStage::Connected,
                        latency: shard.latency.unwrap_or_default().as_millis() as f64,
                        guilds: guild_counts_per_shard
                            .get(&shard_id.0)
                            .copied()
                            .unwrap_or_default(),
                        users: user_counts_per_shard
                            .get(&shard_id.0)
                            .copied()
                            .unwrap_or_default(),
                    };

                    shard_healths.push(shard_health);
                }

                let diag_payload_str = match serde_json::to_string(&MewldDiagResponse {
                    cluster_id: super::argparse::MEWLD_ARGS.cluster_id,
                    nonce: diag_payload.nonce.clone(),
                    data: shard_healths.clone(),
                }) {
                    Ok(v) => v,
                    Err(e) => {
                        log::warn!("Error processing message recieved on channel {}: {} [cannot serialize]", message.channel, e);
                        continue;
                    }
                };

                let diag_response = LauncherCmd {
                    action: "diag".to_string(),
                    scope: "launcher".to_string(),
                    args: None,
                    command_id: None,
                    output: Some(serde_json::Value::String(diag_payload_str)),
                    data: None,
                };

                self.publish_ipc_launchercmd(diag_response).await.unwrap();
            } else {
                let lcmd: LauncherCmd = match serde_json::from_value(value) {
                    Ok(v) => v,
                    Err(e) => {
                        log::warn!(
                            "Invalid message recieved on channel {}: {} [not a launcher command]",
                            message.channel,
                            e
                        );
                        continue;
                    }
                };

                match lcmd.action.as_str() {
                    // Diag is guaranteed to only be a response to a diag request
                    "diag" => {
                        let Some(output) = lcmd.output else {
                            log::warn!("Invalid message recieved on channel {} [diag key set but not a diag payload]", message.channel);
                            continue;
                        };

                        let output_str = match output {
                            serde_json::Value::String(s) => s,
                            _ => {
                                log::warn!("Invalid message recieved on channel {} [diag key set but not a diag payload]", message.channel);
                                continue;
                            }
                        };

                        // Parse output to a diagresponse
                        let diag_response: MewldDiagResponse = match serde_json::from_str(
                            &output_str,
                        ) {
                            Ok(v) => v,
                            Err(e) => {
                                log::warn!("Invalid message recieved on channel {}: {} [diag key set but not a diag payload]", message.channel, e);
                                continue;
                            }
                        };

                        // We have recieved a diagnostic payload from other clusters, save it
                        self.cache
                            .cluster_healths
                            .insert(diag_response.cluster_id, diag_response.data);
                    }
                    "all_clusters_launched" => {
                        // All clusters have launched, set the flag
                        *self.cache.all_clusters_up.write().await = true;
                    }
                    "launch_next" => {} // Ignore
                    _ => {
                        log::warn!(
                            "Invalid message recieved on channel {} [not a launcher command]",
                            message.channel
                        );
                        continue;
                    }
                }
            }
        }

        unreachable!("IPC listener exited");
    }

    /// Publishes a message to the redis IPC channel via the standard launchercmd
    pub async fn publish_ipc_launchercmd(&self, cmd: LauncherCmd) -> Result<(), crate::Error> {
        let cmd = serde_json::to_string(&cmd)?;

        let conn = self.redis_pool.next();
        conn.connect();
        conn.wait_for_connect().await?;
        conn.publish(super::argparse::MEWLD_ARGS.mewld_redis_channel.clone(), cmd)
            .await?;

        Ok(())
    }

    /// Publishes a launch_next command
    pub async fn publish_ipc_launch_next(&self) -> Result<(), crate::Error> {
        let cmd = LauncherCmd {
            scope: "launcher".to_string(),
            action: "launch_next".to_string(),
            args: Some(serde_json::json!({
                "id": super::argparse::MEWLD_ARGS.cluster_id,
            })),
            command_id: None,
            output: None,
            data: None,
        };

        self.publish_ipc_launchercmd(cmd).await
    }
}

use serde::{Serialize, Deserialize};
use serde_repr::{Serialize_repr, Deserialize_repr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum ShardGroupStatus {
    Idle,
    Connecting,
    Connected,
    MarkedForClosure,
    Closing,
    Closed,
    Erroring,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatusEndpointResponse {
    pub uptime: i64,
    pub managers: Vec<StatusEndpointManager>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatusEndpointManager {
    pub display_name: String,
    pub shard_groups: Vec<StatusEndpointShardGroup>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatusEndpointShardGroup {
    pub shard_group_id: i32,
    pub shards: Vec<[i64; 6]>, // // ShardID, Status, Latency (in milliseconds), Guilds, Uptime (in seconds), Total Uptime (in seconds)
    pub status: ShardGroupStatus,
    pub uptime: i64,
}
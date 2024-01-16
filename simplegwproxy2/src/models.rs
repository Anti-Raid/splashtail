use serenity::all::{
    ActivityType,
    ApplicationId,
    ApplicationFlags,
    CurrentUser,
    GuildId,
    UserId,
    OnlineStatus
};
use small_fixed_array::FixedString;
use url::Url;
use std::sync::Arc;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Ready {
    pub version: u8,
    pub user: CurrentUser,
    pub guilds: Vec<UnavailableGuild>,
    pub session_id: String,
    pub resume_gateway_url: String,
    pub shard: Option<[u16; 2]>,
    pub application: PartialCurrentApplicationInfo,
} 

/// Data for an unavailable guild.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UnavailableGuild {
    pub id: GuildId,
    pub unavailable: bool,
} 

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PartialCurrentApplicationInfo {
    pub id: ApplicationId,
    pub flags: ApplicationFlags,
}

pub enum QueuedEvent {
    Ping,
    Dispatch(Arc<Event>),
    DispatchBulk(Arc<Vec<Event>>),
    DispatchValue(Arc<serde_json::Value>), // Temp
    Close(tokio_websockets::CloseCode, String)
}

#[derive(PartialEq)]
pub enum SessionState {
    Unidentified,
    Authorized,
}

pub struct Session {
    pub last_heartbeat: std::time::Instant,
    pub shard: [u16; 2],
    pub dispatcher: tokio::sync::mpsc::Sender<QueuedEvent>,
    pub state: SessionState,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Hello {
    pub heartbeat_interval: u128,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GatewayResumeEvent {
    pub token: String,
    pub session_id: String,
    pub seq: u64,
}

#[derive(serde::Serialize, serde::Deserialize)]
/// Nothing else matters
pub struct Identify {
    pub token: String,
    pub shard: [u16; 2],
}

/// Activity data of the current user.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ActivityData {
    /// The name of the activity
    pub name: FixedString<u32>,
    /// The type of the activity
    #[serde(rename = "type")]
    pub kind: ActivityType,
    /// The state of the activity, if the type is [`ActivityType::Custom`]
    pub state: Option<FixedString<u32>>,
    /// The url of the activity, if the type is [`ActivityType::Streaming`]
    pub url: Option<Url>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct GatewayUpdatePresence {
    pub since: Option<u64>,
    pub activities: Option<Vec<ActivityData>>,
    pub status: OnlineStatus,
    pub afk: bool,
}

/// Guild Request Members
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GatewayGuildRequestMembers {
    pub guild_id: GuildId,
    pub query: Option<String>,
    pub limit: Option<u16>,
    pub presences: Option<bool>,
    pub user_ids: Option<Vec<UserId>>,
    pub nonce: Option<String>,
}

#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr, PartialEq)]
#[repr(u8)]
pub enum EventOpCode {
    Dispatch = 0,
    Heartbeat = 1,
    Identify = 2,
    PresenceUpdate = 3,
    VoiceStateUpdate = 4,
    Resume = 6,
    Reconnect = 7,
    RequestGuildMembers = 8,
    InvalidSession = 9,
    Hello = 10,
    HeartbeatAck = 11,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Event {
    pub op: EventOpCode,
    pub s: Option<u64>,
    pub d: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t: Option<String>,
}

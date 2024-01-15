use serenity::all::{
    ApplicationId,
    ApplicationFlags,
    CurrentUser,
    GuildId
};

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
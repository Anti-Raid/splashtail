use dashmap::DashMap;
use serenity::all::{GuildId, UserId};
use sqlx::PgPool;
use std::sync::Arc;
use std::collections::HashMap;

pub type GCLimitCache = Arc<HashMap<String, super::core::Limit>>;
pub type GCTargetConfigCache = Arc<HashMap<super::core::UserLimitTypes, super::core::GuildUserTargetSettings>>;

#[derive(Clone)]
pub struct GuildCache {
    pub limits: GCLimitCache,
    pub target_config: GCTargetConfigCache
}

impl GuildCache {
    pub async fn from_guild(pool: &PgPool, guild_id: GuildId) -> Result<Self, crate::Error> {
        let limits = super::core::Limit::from_guild(pool, guild_id).await?;
        let target_config = super::core::GuildUserTargetSettings::from_guild(pool, guild_id).await?;
        Ok(Self {
            limits: Arc::new(limits.into_iter().map(|l| (l.limit_id.clone(), l)).collect()),
            target_config: Arc::new(target_config.into_iter().map(|t| (t.limit_type.clone(), t)).collect())
        })
    }
}

pub static GUILD_CACHE: once_cell::sync::Lazy<DashMap<GuildId, GuildCache>> = once_cell::sync::Lazy::new(|| {
    DashMap::new()
});

/// In order to properly handle and ignore already resolved/hit actions, 
/// we need to store a resolution of each action
#[derive(Debug, Clone)]
pub struct TimesResolution {
    /// The target affected
    pub target: String,
    /// Which limit IDs were hit
    pub limits: Vec<String>,
    /// Action data
    pub action_data: serde_json::Value,
}

// Guild Member Current Actions
#[derive(Debug, Clone)]
pub struct GuildMemberCurrentActions {
    /// The times the user has performed the action
    /// 
    /// The key is the timestamp and the value is the target
    pub times: indexmap::IndexMap<i64, TimesResolution>, 
}

/// Stores a map of the limit types to the actions peformed by a user of said type in a guild
pub type GuildMemberLimitTypesMap = HashMap<super::core::UserLimitTypes, GuildMemberCurrentActions>;

/// Stores a map of the user id to the GuildMemberLimitsCache for each user (see its comment for more info)
pub type GuildMemberLimitsUserMap = HashMap<UserId, GuildMemberLimitTypesMap>;

/// Stores a map of the guild id to the GuildMemberLimitsUserMap for each guild (see its comment for more info)
pub type GuildMemberCurrentActionsCache = DashMap<GuildId, GuildMemberLimitsUserMap>;

/// Stores the current actions of a user in a guild
pub static GUILD_MEMBER_ACTIONS_CACHE: once_cell::sync::Lazy<GuildMemberCurrentActionsCache> = once_cell::sync::Lazy::new(|| {
    DashMap::new()
});
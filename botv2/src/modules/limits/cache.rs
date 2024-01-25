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

#[derive(Debug, Clone)]
pub struct HitLimitsData {
    pub current_hit_limit: super::core::CurrentUserLimitsHit,
    pub hit_id: Option<String>,
    pub notes: Vec<String>
}

// Guild Member Current Actions
#[derive(Debug, Clone)]
pub struct GuildMemberCurrentActions {
    /// The times the user has performed the action
    /// 
    /// The key is the timestamp and the value is the target
    pub times: indexmap::IndexMap<i64, TimesResolution>, 
    /// Timestamp->Limit map
    pub time_action_map: DashMap<i64, String>,
    /// Hit limits
    pub hit_limits: DashMap<i64, Vec<HitLimitsData>>,
}

impl GuildMemberCurrentActions {
    /// Syncs with the database returning newly created action IDs
    pub async fn sync_with_db(&self, pool: &PgPool, limit_type: super::core::UserLimitTypes, user_id: UserId, guild_id: GuildId) -> Result<Vec<String>, crate::Error> {        
        let mut action_ids = Vec::new();
        for (ts, tr) in self.times.iter() {
            if self.time_action_map.contains_key(ts) {
                continue;
            }

            // Insert into limits__user_actions
            let action_id = crate::impls::crypto::gen_random(48);
            sqlx::query!(
                "
                INSERT INTO limits__user_actions 
                (action_id, guild_id, user_id, target, limit_type, action_data, created_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
            ",
                action_id.clone(),
                guild_id.to_string(),
                user_id.to_string(),
                tr.target.clone(),
                limit_type.to_string(),
                tr.action_data.clone(),
                sqlx::types::chrono::DateTime::from_timestamp(*ts, 0).unwrap()
            )
            .execute(pool)
            .await?;

            // Insert into time_action_map
            self.time_action_map.insert(*ts, action_id.clone());

            action_ids.push(action_id);
        }

        // Add in the hit limits
        for mut entry in self.hit_limits.iter_mut() {  
            let entries = entry.value_mut();
            for data in entries {
                if data.hit_id.is_some() {
                    continue;
                }
                
                let hit_id = crate::impls::crypto::gen_random(16);
                sqlx::query!(
                    "
                INSERT INTO limits__past_hit_limits
                (id, guild_id, user_id, limit_id, cause, notes)
                VALUES ($1, $2, $3, $4, $5, $6)",
                    &hit_id,
                    guild_id.to_string(),
                    user_id.to_string(),
                    data.current_hit_limit.limit_id.clone(),
                    &data.current_hit_limit
                    .cause
                    .iter()
                    .map(|a| a.action_id.clone())
                    .collect::<Vec<_>>(),
                    &data.notes
                )
                .execute(pool)
                .await?;

                data.hit_id = Some(hit_id.clone());
            }
        }

        Ok(action_ids)
    }
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

use std::collections::HashMap;
use log::info;
use num_traits::ToPrimitive;
use poise::serenity_prelude::{GuildId, UserId};
use serde::{Deserialize, Serialize};
use sqlx::{
    postgres::types::PgInterval,
    types::chrono::{DateTime, Utc},
    PgPool,
};
use strum_macros::{Display, EnumString, EnumVariantNames};
use surrealdb::engine::local::Db;
use surrealdb::Surreal;

use crate::Error;
use crate::impls::utils::pg_interval_to_secs;

#[derive(poise::ChoiceParameter)]
pub enum UserLimitTypesChoices {
    #[name = "Role Create"]
    RoleAdd,
    #[name = "Role Update"]
    RoleUpdate,
    #[name = "Role Remove"]
    RoleRemove,
    #[name = "Role Given To Member"]
    RoleGivenToMember,
    #[name = "Role Removed From Member"]
    RoleRemovedFromMember,
    #[name = "Member Roles Updated"]
    MemberRolesUpdated,
    #[name = "Channel Create"]
    ChannelAdd,
    #[name = "Channel Update"]
    ChannelUpdate,
    #[name = "Channel Remove"]
    ChannelRemove,
    #[name = "Kick"]
    Kick,
    #[name = "Ban"]
    Ban,
    #[name = "Unban"]
    Unban,
}

impl UserLimitTypesChoices {
    pub fn resolve(self) -> UserLimitTypes {
        match self {
            Self::RoleAdd => UserLimitTypes::RoleAdd,
            Self::RoleUpdate => UserLimitTypes::RoleUpdate,
            Self::RoleRemove => UserLimitTypes::RoleRemove,
            Self::RoleGivenToMember => UserLimitTypes::RoleGivenToMember,
            Self::RoleRemovedFromMember => UserLimitTypes::RoleRemovedFromMember,
            Self::MemberRolesUpdated => UserLimitTypes::MemberRolesUpdated,
            Self::ChannelAdd => UserLimitTypes::ChannelAdd,
            Self::ChannelUpdate => UserLimitTypes::ChannelUpdate,
            Self::ChannelRemove => UserLimitTypes::ChannelRemove,
            Self::Kick => UserLimitTypes::Kick,
            Self::Ban => UserLimitTypes::Ban,
            Self::Unban => UserLimitTypes::Unban,
        }
    }
}

#[derive(EnumString, Display, PartialEq, EnumVariantNames, Clone, Copy, Debug, Serialize, Hash, Eq, Deserialize)]
#[strum(serialize_all = "snake_case")]
pub enum    UserLimitTypes {
    RoleAdd,       // set
    RoleUpdate,    // set
    RoleRemove,    // set
    RoleGivenToMember, // set
    RoleRemovedFromMember, // set
    MemberRolesUpdated, // set
    ChannelAdd,    // set
    ChannelUpdate, // set
    ChannelRemove, //set
    Kick,
    Ban, 
    Unban,
}

impl UserLimitTypes {
    pub fn to_cond(self) -> String {
        match &self {
            Self::RoleAdd => "Roles Created".to_string(),
            Self::RoleUpdate => "Roles Updated".to_string(),
            Self::RoleRemove => "Role Removed".to_string(),
            Self::RoleGivenToMember => "Role Given To Member".to_string(),
            Self::RoleRemovedFromMember => "Role Removed From Member".to_string(),
            Self::MemberRolesUpdated => "Member Roles Updated".to_string(),
            Self::ChannelAdd => "Channels Created".to_string(),
            Self::ChannelUpdate => "Channels Updated".to_string(),
            Self::ChannelRemove => "Channels Removed".to_string(),
            Self::Kick => "Kicks".to_string(),
            Self::Ban => "Bans".to_string(),
            Self::Unban => "Unbans".to_string(),
        }
    }

    /// Returns the default repeat rate for a user-target pair
    /// 
    /// Within this rate, limits will be ignored if the same user-target pair has already been handled
    #[allow(dead_code)] // May be used in the future
    pub fn default_user_target_repeat_rate(&self) -> i64 {
        match self {
            Self::RoleGivenToMember => 2, // 2 seconds
            Self::RoleRemovedFromMember => 2, // 2 seconds
            Self::MemberRolesUpdated => 5, // 5 seconds
            _ => 0, // No repeat rate as other events are more accurate
        }
    }
}

#[derive(poise::ChoiceParameter)]
pub enum UserLimitActionsChoices {
    #[name = "Remove All Roles"]
    RemoveAllRoles,
    #[name = "Kick User"]
    KickUser,
    #[name = "Ban User"]
    BanUser,
}

impl UserLimitActionsChoices {
    pub fn resolve(self) -> UserLimitActions {
        match self {
            Self::RemoveAllRoles => UserLimitActions::RemoveAllRoles,
            Self::KickUser => UserLimitActions::KickUser,
            Self::BanUser => UserLimitActions::BanUser,
        }
    }
}

#[derive(EnumString, Display, PartialEq, EnumVariantNames, Clone, Debug, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
pub enum UserLimitActions {
    RemoveAllRoles,
    KickUser,
    BanUser,
}

impl UserLimitActions {
    pub fn to_cond(&self) -> String {
        match &self {
            Self::RemoveAllRoles => "Remove All Roles".to_string(),
            Self::KickUser => "Kick User".to_string(),
            Self::BanUser => "Ban User".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserAction {
    /// The ID of the action
    pub action_id: String,
    /// The limit type associated with this action performed
    pub limit_type: UserLimitTypes,
    /// The time the action was performed
    pub created_at: DateTime<Utc>,
    /// The ID of the user who performed the action
    pub user_id: UserId,
    /// The ID of the guild the action was performed in
    pub guild_id: GuildId,
    /// The data associated with the action (extra data etc.)
    pub action_data: serde_json::Value,
    /// The limits that have been hit for this action
    pub limits_hit: Vec<String>,
    /// The target the action was intended for
    pub target: String,
}

impl UserAction {
    /// Fetch user actions for a action id
    pub async fn by_id(
        pool: &PgPool,
        guild_id: GuildId,
        action_id: &str,
    ) -> Result<Self, Error> {
        let r = sqlx::query!(
            "
                SELECT user_id, limit_type, created_at, action_data, 
                limits_hit, target FROM limits__user_actions
                WHERE guild_id = $1
                AND action_id = $2
            ",
            guild_id.to_string(),
            action_id.to_string()
        )
        .fetch_one(pool)
        .await?;


    let actions = Self {
            guild_id,
            action_id: action_id.to_string(),
            user_id: r.user_id.parse()?,
            limit_type: r.limit_type.parse()?,
            created_at: r.created_at,
            action_data: r.action_data,
            limits_hit: r.limits_hit,
            target: r.target,
        };

        Ok(actions)
    }

    /// Fetch actions for a user in a guild
    pub async fn user(
        pool: &PgPool,
        guild_id: GuildId,
        user_id: UserId,
    ) -> Result<Vec<Self>, Error> {
        let rec = sqlx::query!(
            "
                SELECT action_id, limit_type, created_at, action_data, 
                limits_hit, target FROM limits__user_actions
                WHERE guild_id = $1
                AND user_id = $2
            ",
            guild_id.to_string(),
            user_id.to_string()
        )
        .fetch_all(pool)
        .await?;

        let mut actions = Vec::new();

        for r in rec {
            actions.push(Self {
                guild_id,
                user_id,
                action_id: r.action_id,
                limit_type: r.limit_type.parse()?,
                created_at: r.created_at,
                action_data: r.action_data,
                limits_hit: r.limits_hit,
                target: r.target,
            });
        }

        Ok(actions)
    }

    /// Fetch all user actions in a guild
    pub async fn guild(pool: &PgPool, guild_id: GuildId) -> Result<Vec<Self>, Error> {
        let rec = sqlx::query!(
            "
                SELECT action_id, limit_type, created_at, user_id, action_data, 
                limits_hit, target FROM limits__user_actions
                WHERE guild_id = $1
            ",
            guild_id.to_string()
        )
        .fetch_all(pool)
        .await?;

        let mut actions = Vec::new();

        for r in rec {
            actions.push(Self {
                guild_id,
                action_id: r.action_id,
                limit_type: r.limit_type.parse()?,
                created_at: r.created_at,
                user_id: r.user_id.parse()?,
                action_data: r.action_data,
                limits_hit: r.limits_hit,
                target: r.target,
            });
        }

        Ok(actions)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Limit {
    /// The ID of the guild this limit is for
    pub guild_id: GuildId,
    /// The ID of the limit
    pub limit_id: String,
    /// The name of the limit
    pub limit_name: String,
    /// The type of limit
    pub limit_type: UserLimitTypes,
    /// The action to take when the limit is hit
    pub limit_action: UserLimitActions,
    /// The number of times the limit can be hit
    pub limit_per: i32,
    /// The time frame, in seconds the limit can be hit in
    pub limit_time: i64,
}

impl Limit {
    pub async fn from_database(pool: &PgPool, guild_id: GuildId) -> Result<Vec<Self>, Error> {
        let rec = sqlx::query!(
            "
                SELECT limit_id, limit_name, limit_type, limit_action, limit_per, 
                limit_time AS limit_time FROM limits__guild_limits
                WHERE guild_id = $1
            ",
            guild_id.to_string()
        )
        .fetch_all(pool)
        .await;

        let rec = match rec {
            Ok(rec) => rec,
            Err(sqlx::Error::RowNotFound) => return Ok(Vec::new()),
            Err(e) => return Err(e.into()),
        };

        let mut limits = Vec::new();

        for r in rec {
            limits.push(Self {
                guild_id,
                limit_id: r.limit_id,
                limit_name: r.limit_name,
                limit_type: r.limit_type.parse()?,
                limit_action: r.limit_action.parse()?,
                limit_per: r.limit_per,
                limit_time: pg_interval_to_secs(r.limit_time),
            });
        }
        Ok(limits)
    }
    pub async fn from_cache(cache: &Surreal<Db>, guild_id: GuildId) -> Result<Vec<Self>, Error> {
        let mut request = cache
            .query("select guild_id, limit_id, limit_name, limit_type, limit_action, limit_per, limit_time from guild_limits where guild_id = type::string($guild_id)")
            .bind(("guild_id", guild_id.to_string()))
            .await?;

        let records: Vec<Limit> = request.take(0)?;
        Ok(records)
    }

    pub async fn fetch(cache: &Surreal<Db>, pool: &PgPool, guild_id: GuildId) -> Result<Vec<Self>, Error> {
        let cache = Self::from_cache(cache, guild_id).await?;
        if cache.is_empty() {
            let db = Self::from_database(pool, guild_id).await?;
            return Ok(db);
        }
        Ok(cache)
    }
}

#[derive(Debug, Serialize)]
pub struct PastHitLimits {
    pub id: String,
    pub user_id: UserId,
    pub guild_id: GuildId,
    pub limit_id: String,
    pub cause: Vec<UserAction>,
    pub notes: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl PastHitLimits {
    /// Fetch actions for guild
    pub async fn guild(pool: &PgPool, guild_id: GuildId) -> Result<Vec<Self>, Error> {
        let rec = sqlx::query!(
            "
                SELECT id, user_id, limit_id, cause, notes, created_at FROM limits__past_hit_limits
                WHERE guild_id = $1
            ",
            guild_id.to_string()
        )
        .fetch_all(pool)
        .await?;

        let mut hits = Vec::new();

        for r in rec {
            let mut cause = vec![];

            for action in r.cause {
                cause.push(UserAction::by_id(pool, guild_id, &action).await?);
            }

            hits.push(Self {
                guild_id,
                id: r.id,
                limit_id: r.limit_id,
                created_at: r.created_at,
                user_id: r.user_id.parse()?,
                notes: r.notes,
                cause,
            });
        }

        Ok(hits)
    }    
}
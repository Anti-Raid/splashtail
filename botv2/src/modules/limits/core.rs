use poise::serenity_prelude::{GuildId, UserId};
use serde::Serialize;
use sqlx::{
    postgres::types::PgInterval,
    types::chrono::{DateTime, Utc},
    PgPool,
};
use strum_macros::{Display, EnumString, EnumVariantNames};

use crate::Error;

#[derive(poise::ChoiceParameter)]
pub enum UserLimitTypesChoices {
    #[name = "Role Create"]
    RoleAdd,
    #[name = "Role Update"]
    RoleUpdate,
    #[name = "Role Remove"]
    RoleRemove,
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
            Self::ChannelAdd => UserLimitTypes::ChannelAdd,
            Self::ChannelUpdate => UserLimitTypes::ChannelUpdate,
            Self::ChannelRemove => UserLimitTypes::ChannelRemove,
            Self::Kick => UserLimitTypes::Kick,
            Self::Ban => UserLimitTypes::Ban,
            Self::Unban => UserLimitTypes::Unban,
        }
    }
}

#[derive(EnumString, Display, PartialEq, EnumVariantNames, Clone, Debug, Serialize)]
#[strum(serialize_all = "snake_case")]
pub enum UserLimitTypes {
    RoleAdd,       // set
    RoleUpdate,    // set
    RoleRemove,    // set
    ChannelAdd,    // set
    ChannelUpdate, // set
    ChannelRemove, //set
    Kick,
    Ban,
    Unban,
}

impl UserLimitTypes {
    pub fn to_cond(&self) -> String {
        match &self {
            Self::RoleAdd => "Roles Created".to_string(),
            Self::RoleUpdate => "Roles Updated".to_string(),
            Self::RoleRemove => "Role Removed".to_string(),
            Self::ChannelAdd => "Channels Created".to_string(),
            Self::ChannelUpdate => "Channels Updated".to_string(),
            Self::ChannelRemove => "Channels Removed".to_string(),
            Self::Kick => "Kicks".to_string(),
            Self::Ban => "Bans".to_string(),
            Self::Unban => "Unbans".to_string(),
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

#[derive(EnumString, Display, PartialEq, EnumVariantNames, Clone, Debug, Serialize)]
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

#[derive(Clone, Debug, Serialize)]
pub struct Action {
    pub action_id: String,
    pub limit_type: UserLimitTypes,
    pub created_at: DateTime<Utc>,
    pub user_id: UserId,
    pub guild_id: GuildId,
    pub action_target: String,
    pub limits_hit: Vec<String>,
}

impl Action {
    /// Fetch actions for a action id
    pub async fn by_id(
        pool: &PgPool,
        guild_id: GuildId,
        action_id: &str,
    ) -> Result<Self, Error> {
        let r = sqlx::query!(
            "
                SELECT user_id, limit_type, created_at, action_target, limits_hit
                FROM user_actions
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
            action_target: r.action_target.parse()?,
            limits_hit: r.limits_hit,
        };

        Ok(actions)
    }

    /// Fetch actions for user
    pub async fn user(
        pool: &PgPool,
        guild_id: GuildId,
        user_id: UserId,
    ) -> Result<Vec<Self>, Error> {
        let rec = sqlx::query!(
            "
                SELECT action_id, limit_type, created_at, action_target, limits_hit
                FROM user_actions
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
                action_target: r.action_target.parse()?,
                limits_hit: r.limits_hit,
            });
        }

        Ok(actions)
    }

    /// Fetch actions for guild
    pub async fn guild(pool: &PgPool, guild_id: GuildId) -> Result<Vec<Self>, Error> {
        let rec = sqlx::query!(
            "
                SELECT action_id, limit_type, created_at, user_id, action_target, limits_hit
                FROM user_actions
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
                action_target: r.action_target.parse()?,
                limits_hit: r.limits_hit,
            });
        }

        Ok(actions)
    }
}

#[derive(Debug)]
pub struct Limit {
    pub guild_id: GuildId,
    pub limit_id: String,
    pub limit_name: String,
    pub limit_type: UserLimitTypes,
    pub limit_action: UserLimitActions,
    pub limit_per: i32,
    pub limit_time: PgInterval,
}

impl Limit {
    pub async fn from_guild(pool: &PgPool, guild_id: GuildId) -> Result<Vec<Self>, Error> {
        let rec = sqlx::query!(
            "
                SELECT limit_id, limit_name, limit_type, limit_action, limit_per, limit_time
                FROM limits
                WHERE guild_id = $1
            ",
            guild_id.to_string()
        )
        .fetch_all(pool)
        .await?;

        let mut limits = Vec::new();

        for r in rec {
            limits.push(Self {
                guild_id,
                limit_id: r.limit_id,
                limit_name: r.limit_name,
                limit_type: r.limit_type.parse()?,
                limit_action: r.limit_action.parse()?,
                limit_per: r.limit_per,
                limit_time: r.limit_time,
            });
        }

        Ok(limits)
    }
}

#[derive(Debug)]
pub struct CurrentUserLimitsHit {
    pub limit: Limit,
    pub cause: Vec<Action>,
}

impl CurrentUserLimitsHit {
    /// Returns a list of all limits that have been hit for a specific guild
    pub async fn hit(guild_id: GuildId, pool: &PgPool) -> Result<Vec<Self>, Error> {
        let limits = Limit::from_guild(pool, guild_id).await?;

        let mut hits = Vec::new();

        for limit in limits {
            let mut cause = Vec::new();

            // Find all actions that apply to this limit
            let rec = sqlx::query!(
                "
                    SELECT action_id, created_at, user_id, action_target, limits_hit
                    FROM user_actions
                    WHERE guild_id = $1
                    AND NOT($4 = ANY(limits_hit)) -- Not already handled
                    AND NOW() - created_at < $2
                    AND limit_type = $3
                ",
                guild_id.to_string(),
                limit.limit_time,
                limit.limit_type.to_string(),
                limit.limit_id
            )
            .fetch_all(pool)
            .await?;

            for r in rec {
                cause.push(Action {
                    guild_id,
                    limit_type: limit.limit_type.clone(),
                    created_at: r.created_at,
                    user_id: r.user_id.parse()?,
                    action_target: r.action_target.parse()?,
                    action_id: r.action_id,
                    limits_hit: r.limits_hit,
                });
            }

            if cause.len() >= limit.limit_per as usize {
                hits.push(Self { limit, cause });
            }
        }

        Ok(hits)
    }
}

#[derive(Debug, Serialize)]
pub struct PastHitLimits {
    pub id: String,
    pub user_id: UserId,
    pub guild_id: GuildId,
    pub limit_id: String,
    pub cause: Vec<Action>,
    pub notes: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl PastHitLimits {
    /// Fetch actions for guild
    pub async fn guild(pool: &PgPool, guild_id: GuildId) -> Result<Vec<Self>, Error> {
        let rec = sqlx::query!(
            "
                SELECT id, user_id, limit_id, cause, notes, created_at FROM past_hit_limits
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
                cause.push(Action::by_id(pool, guild_id, &action).await?);
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
use serde::{Deserialize, Serialize};
use serenity::all::{GuildId, UserId};
use silverpelt::Error;
use splashcore_rs::utils::pg_interval_to_secs;
use sqlx::PgPool;
use strum_macros::{Display, EnumString, VariantNames};

#[derive(
    EnumString,
    Display,
    PartialEq,
    VariantNames,
    Clone,
    Copy,
    Debug,
    Serialize,
    Hash,
    Eq,
    Deserialize,
)]
#[strum(serialize_all = "snake_case")]
pub enum LimitTypes {
    MemberAdd,
    RoleAdd,               // set
    RoleUpdate,            // set
    RoleRemove,            // set
    RoleGivenToMember,     // set
    RoleRemovedFromMember, // set
    MemberRolesUpdated,    // set
    ChannelAdd,            // set
    ChannelUpdate,         // set
    ChannelRemove,         // set
    Kick,                  // set
    Ban,                   // set
    Unban,                 // set
    MessageCreate,         // set
    PruneMembers,          // set
    Custom(u8),            // unused, for future use
}

#[allow(dead_code)] // Will be used later
impl LimitTypes {
    pub fn to_cond(self) -> String {
        match &self {
            Self::MemberAdd => "Member Added to Server".to_string(),
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
            Self::MessageCreate => "Messages Created".to_string(),
            Self::PruneMembers => "Members Pruned".to_string(),
            Self::Custom(c) => format!("Custom {}", c),
        }
    }

    pub fn from_std_events(typ: std_events::limit::LimitTypes) -> Self {
        match typ {
            std_events::limit::LimitTypes::MemberAdd => Self::MemberAdd,
            std_events::limit::LimitTypes::RoleAdd => Self::RoleAdd,
            std_events::limit::LimitTypes::RoleUpdate => Self::RoleUpdate,
            std_events::limit::LimitTypes::RoleRemove => Self::RoleRemove,
            std_events::limit::LimitTypes::RoleGivenToMember => Self::RoleGivenToMember,
            std_events::limit::LimitTypes::RoleRemovedFromMember => Self::RoleRemovedFromMember,
            std_events::limit::LimitTypes::MemberRolesUpdated => Self::MemberRolesUpdated,
            std_events::limit::LimitTypes::ChannelAdd => Self::ChannelAdd,
            std_events::limit::LimitTypes::ChannelUpdate => Self::ChannelUpdate,
            std_events::limit::LimitTypes::ChannelRemove => Self::ChannelRemove,
            std_events::limit::LimitTypes::Kick => Self::Kick,
            std_events::limit::LimitTypes::Ban => Self::Ban,
            std_events::limit::LimitTypes::Unban => Self::Unban,
            std_events::limit::LimitTypes::MessageCreate => Self::MessageCreate,
            std_events::limit::LimitTypes::PruneMembers => Self::PruneMembers,
            std_events::limit::LimitTypes::Custom(c) => Self::Custom(c),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// Stores the configuration for a guild
pub struct LimitGuild {
    pub guild_id: GuildId,

    /// Which strategy to use for this guild
    pub strategy: String,
}

impl LimitGuild {
    pub fn default_for_guild(guild_id: GuildId) -> Self {
        Self {
            guild_id,
            strategy: "in-memory".to_string(),
        }
    }

    pub async fn get(pool: &PgPool, guild_id: GuildId) -> Result<Self, Error> {
        let rec = sqlx::query!(
            "SELECT guild_id, strategy FROM limits__guilds WHERE guild_id = $1",
            guild_id.to_string()
        )
        .fetch_one(pool)
        .await;

        let rec = match rec {
            Ok(rec) => rec,
            Err(sqlx::Error::RowNotFound) => {
                return Ok(Self::default_for_guild(guild_id));
            }
            Err(e) => return Err(e.into()),
        };

        Ok(Self {
            guild_id,
            strategy: rec.strategy,
        })
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
    pub limit_type: LimitTypes,
    /// The number of stings to give when the limit is hit
    pub stings: i32,
    /// The number of times the limit can be hit
    pub limit_per: i32,
    /// The time frame, in seconds the limit can be hit in
    pub limit_time: i64,
}

impl Limit {
    pub async fn guild(pool: &PgPool, guild_id: GuildId) -> Result<Vec<Self>, Error> {
        let rec = sqlx::query!(
            "
                SELECT limit_id, limit_name, limit_type, stings, limit_per, 
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
                limit_id: r.limit_id.to_string(),
                limit_name: r.limit_name,
                limit_type: r.limit_type.parse()?,
                stings: r.stings,
                limit_per: r.limit_per,
                limit_time: pg_interval_to_secs(r.limit_time),
            });
        }
        Ok(limits)
    }
}

pub struct HandleModAction {
    /// Guild ID
    pub guild_id: GuildId,
    /// User ID
    pub user_id: UserId,
    /// Limit to handle for the User ID in question
    pub limit: LimitTypes,
    /// Target of the action
    pub target: Option<String>,
    /// Extra data for the action
    pub action_data: serde_json::Value,
}

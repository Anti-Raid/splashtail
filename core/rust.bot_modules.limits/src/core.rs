use serde::{Deserialize, Serialize};
use serenity::all::{GuildId, UserId};
use serenity::async_trait;
use silverpelt::{sting_sources, Error};
use splashcore_rs::utils::pg_interval_to_secs;
use sqlx::PgPool;
use strum_macros::{Display, EnumString, VariantNames};

pub(crate) struct LimitsUserStingsStingSource;

#[async_trait]
impl sting_sources::StingSource for LimitsUserStingsStingSource {
    fn id(&self) -> String {
        "limits__user_stings".to_string()
    }

    fn description(&self) -> String {
        "Limits User Stings".to_string()
    }

    fn flags(&self) -> sting_sources::StingSourceFlags {
        sting_sources::StingSourceFlags::SUPPORTS_DURATIONS
            | sting_sources::StingSourceFlags::SUPPORTS_DELETE
    }

    async fn fetch(
        &self,
        data: &sting_sources::StingSourceData,
        filters: sting_sources::StingFetchFilters,
    ) -> Result<Vec<sting_sources::FullStingEntry>, silverpelt::Error> {
        let rows = sqlx::query!(
            "
            SELECT id, user_id, guild_id, stings, expiry, created_at FROM limits__user_stings
            WHERE 
                ($1::TEXT IS NULL OR guild_id = $1::TEXT) AND 
                ($2::TEXT IS NULL OR user_id = $2::TEXT) AND (
                $3::BOOL IS NULL OR 
                ($3 = true AND expiry < NOW()) OR
                ($3 = false AND expiry > NOW())
            )",
            filters.guild_id.map(|g| g.to_string()),
            filters.user_id.map(|u| u.to_string()),
            filters.expired,
        )
        .fetch_all(&data.pool)
        .await?;

        let mut entries = Vec::new();
        for row in rows {
            // As limits does not support StingState, we emulate it using expiry here
            let (duration, state) = {
                let delta = row.expiry - row.created_at;

                // Convert to std::time::Duration
                let dur = std::time::Duration::from_secs(delta.num_seconds() as u64);

                if row.expiry < chrono::Utc::now() {
                    (Some(dur), sting_sources::StingState::Handled)
                } else {
                    (Some(dur), sting_sources::StingState::Active)
                }
            };

            entries.push(sting_sources::FullStingEntry {
                entry: sting_sources::StingEntry {
                    user_id: row.user_id.parse()?,
                    guild_id: row.guild_id.parse()?,
                    stings: row.stings,
                    reason: None,
                    void_reason: None,
                    action: sting_sources::Action::None,
                    state,
                    duration,
                    creator: sting_sources::StingCreator::System,
                },
                created_at: row.created_at,
                id: row.id.to_string(),
            });
        }

        Ok(filters.client_side_apply_filters(entries))
    }

    // No-op
    async fn create_sting_entry(
        &self,
        _data: &sting_sources::StingSourceData,
        entry: sting_sources::StingEntry,
    ) -> Result<sting_sources::FullStingEntry, silverpelt::Error> {
        Ok(sting_sources::FullStingEntry {
            entry,
            created_at: chrono::Utc::now(),
            id: "".to_string(),
        })
    }

    // No-op
    async fn update_sting_entry(
        &self,
        _data: &sting_sources::StingSourceData,
        _id: String,
        _entry: sting_sources::UpdateStingEntry,
    ) -> Result<(), silverpelt::Error> {
        Ok(())
    }

    async fn delete_sting_entry(
        &self,
        data: &sting_sources::StingSourceData,
        id: String,
    ) -> Result<(), silverpelt::Error> {
        sqlx::query!(
            "DELETE FROM limits__user_stings WHERE id = $1",
            id.parse::<sqlx::types::uuid::Uuid>()?
        )
        .execute(&data.pool)
        .await?;

        Ok(())
    }
}

#[derive(poise::ChoiceParameter)]
pub enum LimitTypesChoices {
    #[name = "Member Added to Server"]
    MemberAdd,
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
    #[name = "Message Create"]
    MessageCreate,
}

impl LimitTypesChoices {
    pub fn resolve(self) -> LimitTypes {
        match self {
            Self::MemberAdd => LimitTypes::MemberAdd,
            Self::RoleAdd => LimitTypes::RoleAdd,
            Self::RoleUpdate => LimitTypes::RoleUpdate,
            Self::RoleRemove => LimitTypes::RoleRemove,
            Self::RoleGivenToMember => LimitTypes::RoleGivenToMember,
            Self::RoleRemovedFromMember => LimitTypes::RoleRemovedFromMember,
            Self::MemberRolesUpdated => LimitTypes::MemberRolesUpdated,
            Self::ChannelAdd => LimitTypes::ChannelAdd,
            Self::ChannelUpdate => LimitTypes::ChannelUpdate,
            Self::ChannelRemove => LimitTypes::ChannelRemove,
            Self::Kick => LimitTypes::Kick,
            Self::Ban => LimitTypes::Ban,
            Self::Unban => LimitTypes::Unban,
            Self::MessageCreate => LimitTypes::MessageCreate,
        }
    }
}

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
    ChannelRemove,         //set
    Kick,
    Ban,
    Unban,
    MessageCreate,
}

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
        }
    }
}

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
pub enum LimitStrategy {
    InMemory,
    // WARNING: Persist is MUCH SLOWER than InMemory and will only be available for servers with less than 1500 members
    Persist,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// Stores the configuration for a guild
pub struct LimitGuild {
    pub guild_id: GuildId,

    /// Whether to persist actions in postgres or just use governor/in-memory limiting
    ///
    /// Note that using persist_actions is MUCH SLOWER than in-memory limiting
    pub strategy: LimitStrategy,
}

impl LimitGuild {
    pub fn default_for_guild(guild_id: GuildId) -> Self {
        Self {
            guild_id,
            strategy: LimitStrategy::InMemory,
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
            strategy: rec.strategy.parse()?,
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

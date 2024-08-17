use poise::serenity_prelude::GuildId;
use serde::{Deserialize, Serialize};
use serenity::async_trait;
use silverpelt::{sting_sources, Error};
use splashcore_rs::utils::pg_interval_to_secs;
use sqlx::PgPool;
use strum_macros::{Display, EnumString, VariantNames};

pub(crate) struct LimitsUserActionsStingSource;

#[async_trait]
impl sting_sources::StingSource for LimitsUserActionsStingSource {
    fn id(&self) -> String {
        "limits__user_actions".to_string()
    }

    fn description(&self) -> String {
        "Limits (User Action) Punishments".to_string()
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
            SELECT stings, stings_expiry, created_at, user_id, guild_id, action_id FROM limits__user_actions
            WHERE 
                ($1::TEXT IS NULL OR guild_id = $1::TEXT) AND 
                ($2::TEXT IS NULL OR user_id = $2::TEXT) AND (
                $3::BOOL IS NULL OR 
                ($3 = true AND stings_expiry < NOW()) OR
                ($3 = false AND stings_expiry > NOW())
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
                let delta = row.stings_expiry - row.created_at;

                // Convert to std::time::Duration
                let dur = std::time::Duration::from_secs(delta.num_seconds() as u64);

                if row.stings_expiry < chrono::Utc::now() {
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
                id: row.action_id,
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
        sqlx::query!("DELETE FROM limits__user_actions WHERE action_id = $1", id)
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
    /// The action to take when the limit is hit
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

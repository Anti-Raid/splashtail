use futures_util::future::FutureExt;
use poise::serenity_prelude::GuildId;
use serde::{Deserialize, Serialize};
use silverpelt::Error;
use splashcore_rs::utils::pg_interval_to_secs;
use sqlx::PgPool;
use strum_macros::{Display, EnumString, VariantNames};

/// Punishment sting source
pub async fn register_punishment_sting_source(
    _data: &silverpelt::data::Data,
) -> Result<(), silverpelt::Error> {
    async fn sting_entries(
        ctx: &serenity::all::Context,
        guild_id: serenity::all::GuildId,
        user_id: serenity::all::UserId,
    ) -> Result<Vec<crate::punishments::sting_source::StingEntry>, silverpelt::Error> {
        let data = ctx.data::<silverpelt::data::Data>();
        let pool = &data.pool;

        let mut entries = vec![];

        // Fetch all entries
        let moderation_entries = sqlx::query!(
                "SELECT stings, (NOW() > stings_expiry) AS expired, created_at FROM limits__user_actions WHERE user_id = $1 AND guild_id = $2",
                user_id.to_string(),
                guild_id.to_string(),
            )
            .fetch_all(pool)
            .await?;

        for entry in moderation_entries {
            entries.push(crate::punishments::sting_source::StingEntry {
                user_id,
                guild_id,
                stings: entry.stings,
                reason: None, // TODO: Add reason (if possible)
                created_at: entry.created_at,
                expired: entry.expired.unwrap_or(false),
            });
        }

        Ok(entries)
    }

    let source = crate::punishments::sting_source::StingSource {
        id: "limits__user_actions".to_string(),
        description: "Limits (User Action) Punishments".to_string(),
        fetch: Box::new(|ctx, guild_id, user_id| sting_entries(ctx, *guild_id, *user_id).boxed()),
    };

    crate::punishments::sting_source::add_sting_source(source);
    Ok(())
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

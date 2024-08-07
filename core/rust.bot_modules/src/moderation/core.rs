use crate::punishments::sting_source::{add_sting_source, StingEntry, StingSource};
use crate::temporary_punishments::source::{
    add_source, Action as TemporaryPunishmentsAction, Entry as TemporaryPunishmentsEntry,
    Source as TemporaryPunishmentsSource,
};
use futures_util::future::FutureExt;
use serenity::all::User;
use splashcore_rs::utils::pg_interval_to_secs;

/// Punishment sting source
pub async fn register_punishment_sting_source(
    _data: &base_data::Data,
) -> Result<(), base_data::Error> {
    async fn sting_entries(
        ctx: &serenity::all::Context,
        guild_id: serenity::all::GuildId,
        user_id: serenity::all::UserId,
    ) -> Result<Vec<StingEntry>, base_data::Error> {
        let data = ctx.data::<base_data::Data>();
        let pool = &data.pool;

        let mut entries = vec![];

        // Fetch all moderation actions of the user in moderation__actions
        let moderation_entries = sqlx::query!(
                "SELECT user_id, guild_id, stings, reason, expired, created_at FROM moderation__actions WHERE user_id = $1 AND guild_id = $2",
                user_id.to_string(),
                guild_id.to_string(),
            )
            .fetch_all(pool)
            .await?;

        for entry in moderation_entries {
            entries.push(StingEntry {
                user_id,
                guild_id,
                stings: entry.stings,
                reason: entry.reason,
                created_at: entry.created_at,
                expired: entry.expired,
            });
        }

        Ok(entries)
    }

    let source = StingSource {
        id: "moderation__actions".to_string(),
        description: "Moderation Actions".to_string(),
        fetch: Box::new(|ctx, guild_id, user_id| sting_entries(ctx, *guild_id, *user_id).boxed()),
    };

    add_sting_source(source);
    Ok(())
}

pub fn username(m: &User) -> String {
    if let Some(ref global_name) = m.global_name {
        global_name.to_string()
    } else {
        m.tag()
    }
}

pub fn to_log_format(moderator: &User, member: &User, reason: &str) -> String {
    format!(
        "{} | Handled '{}' for reason '{}'",
        username(moderator),
        username(member),
        reason
    )
}

/// Temp punishments sting source
pub async fn register_temporary_punishment_source(
    _data: &base_data::Data,
) -> Result<(), base_data::Error> {
    async fn entries(
        ctx: &serenity::all::Context,
    ) -> Result<Vec<TemporaryPunishmentsEntry>, base_data::Error> {
        let data = ctx.data::<base_data::Data>();
        let pool = &data.pool;

        let mut entries = vec![];

        // Fetch all moderation actions of the user in moderation__actions
        let moderation_entries = sqlx::query!(
                "SELECT id, user_id, moderator, guild_id, stings, reason, action, duration, created_at FROM moderation__actions WHERE handled = false AND (expired = true OR (duration IS NOT NULL AND duration + created_at < NOW()))",
            )
            .fetch_all(pool)
            .await?;

        for entry in moderation_entries {
            entries.push(TemporaryPunishmentsEntry {
                id: entry.id.to_string(),
                user_id: entry.user_id.parse::<serenity::all::UserId>()?,
                moderator: entry.moderator.parse::<serenity::all::UserId>()?,
                guild_id: entry.guild_id.parse::<serenity::all::GuildId>()?,
                stings: entry.stings,
                reason: entry.reason,
                created_at: entry.created_at,
                duration: match entry.duration {
                    Some(d) => {
                        std::time::Duration::from_secs(u64::try_from(pg_interval_to_secs(d))?)
                    }
                    None => continue,
                },
                action: {
                    match entry.action.as_str() {
                        "ban" => TemporaryPunishmentsAction::Ban,
                        "remove-all-roles" => TemporaryPunishmentsAction::RemoveAllRoles,
                        _ => continue,
                    }
                },
            });
        }

        Ok(entries)
    }

    async fn log_error(
        ctx: &serenity::all::Context,
        entry: &TemporaryPunishmentsEntry,
        error: Option<String>,
    ) -> Result<(), base_data::Error> {
        let data = ctx.data::<base_data::Data>();
        let pool = &data.pool;

        let id = entry.id.parse::<sqlx::types::Uuid>()?;

        match error {
            Some(error) => {
                sqlx::query!(
                    "UPDATE moderation__actions SET handled = true, handle_errors = $1 WHERE id = $2",
                    error.to_string(),
                    id
                )
                .execute(pool)
                .await?;
            }
            None => {
                sqlx::query!(
                    "UPDATE moderation__actions SET handled = true WHERE guild_id = $1 AND user_id = $2 AND action = $3",
                    entry.guild_id.to_string(),
                    entry.user_id.to_string(),
                    match entry.action {
                        TemporaryPunishmentsAction::Ban => "ban".to_string(),
                        TemporaryPunishmentsAction::RemoveAllRoles => "remove-all-roles".to_string()
                    }
                )
                .execute(pool)
                .await?;
            }
        }

        Ok(())
    }

    let source = TemporaryPunishmentsSource {
        id: "moderation__actions".to_string(),
        description: "Moderation Actions".to_string(),
        fetch: Box::new(|ctx| entries(ctx).boxed()),
        log_error: Box::new(|ctx, entry, error| log_error(ctx, entry, error.clone()).boxed()),
    };

    add_source(source);
    Ok(())
}

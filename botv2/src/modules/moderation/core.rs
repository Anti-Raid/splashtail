use futures_util::FutureExt;
use serenity::all::User;

/// Punishment sting source
pub async fn register_punishment_sting_source(_data: &crate::Data) -> Result<(), crate::Error> {
    async fn sting_entries(
        ctx: &serenity::all::Context,
        guild_id: serenity::all::GuildId,
        user_id: serenity::all::UserId,
    ) -> Result<Vec<crate::modules::punishments::sting_source::StingEntry>, crate::Error> {
        let data = ctx.data::<crate::Data>();
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
            entries.push(crate::modules::punishments::sting_source::StingEntry {
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

    let source = crate::modules::punishments::sting_source::StingSource {
        id: "moderation__actions".to_string(),
        description: "Moderation Actions".to_string(),
        fetch: Box::new(|ctx, guild_id, user_id| sting_entries(ctx, *guild_id, *user_id).boxed()),
    };

    crate::modules::punishments::sting_source::add_sting_source(source);
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

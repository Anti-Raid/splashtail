use futures_util::future::FutureExt;

/// Punishment sting source
pub async fn register_punishment_sting_source(
    _data: &silverpelt::data::Data,
) -> Result<(), silverpelt::Error> {
    async fn sting_entries(
        ctx: &serenity::all::Context,
        guild_id: serenity::all::GuildId,
        user_id: serenity::all::UserId,
    ) -> Result<Vec<bot_modules_punishments::sting_source::StingEntry>, silverpelt::Error> {
        let data = ctx.data::<silverpelt::data::Data>();
        let pool = &data.pool;

        let mut entries = vec![];

        let opts = super::cache::get_config(pool, guild_id).await?;

        // Delete old entries
        sqlx::query!(
            "DELETE FROM inspector__punishments WHERE created_at < $1",
            chrono::Utc::now() - chrono::Duration::seconds(opts.sting_retention as i64),
        )
        .execute(pool)
        .await?;

        // Fetch all entries
        let ba_entries = sqlx::query!(
                "SELECT stings, created_at FROM inspector__punishments WHERE user_id = $1 AND guild_id = $2",
                user_id.to_string(),
                guild_id.to_string(),
            )
            .fetch_all(pool)
            .await?;

        for entry in ba_entries {
            entries.push(bot_modules_punishments::sting_source::StingEntry {
                user_id,
                guild_id,
                stings: entry.stings,
                reason: None, // TODO: Add reason (if possible)
                created_at: entry.created_at,
                expired: false,
            });
        }

        Ok(entries)
    }

    let source = bot_modules_punishments::sting_source::StingSource {
        id: "inspector__punishments".to_string(),
        description: "Inspector Punishments".to_string(),
        fetch: Box::new(|ctx, guild_id, user_id| sting_entries(ctx, *guild_id, *user_id).boxed()),
    };

    bot_modules_punishments::sting_source::add_sting_source(source);
    Ok(())
}

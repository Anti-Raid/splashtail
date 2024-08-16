use serenity::async_trait;
use silverpelt::sting_sources;
use sqlx::Row;

pub(crate) struct InspectorPunishmentsStingSource;

#[async_trait]
impl sting_sources::StingSource for InspectorPunishmentsStingSource {
    fn id(&self) -> String {
        "inspector__punishments".to_string()
    }

    fn description(&self) -> String {
        "Inspector Punishments".to_string()
    }

    fn flags(&self) -> sting_sources::StingSourceFlags {
        sting_sources::StingSourceFlags::SUPPORTS_DELETE
            | sting_sources::StingSourceFlags::REQUIRES_GUILD_ID_IN_FILTER
    }

    async fn fetch(
        &self,
        ctx: &serenity::all::Context,
        filters: sting_sources::StingFetchFilters,
    ) -> Result<Vec<sting_sources::FullStingEntry>, silverpelt::Error> {
        let Some(guild_id) = filters.guild_id else {
            return Err("Guild ID is required for this sting source".into());
        };

        let data = ctx.data::<silverpelt::data::Data>();

        let opts = super::cache::get_config(&data.pool, guild_id).await?;

        // Delete old entries
        sqlx::query!(
            "DELETE FROM inspector__punishments WHERE created_at < $1",
            chrono::Utc::now() - chrono::Duration::seconds(opts.sting_retention as i64),
        )
        .execute(&data.pool)
        .await?;

        let base_query = "SELECT id, user_id, guild_id, stings, created_at FROM inspector__punishments WHERE guild_id = $1";

        let mut where_filters = Vec::new();

        // User ID filter
        if filters.user_id.is_some() {
            where_filters.push(format!("user_id = ${}", where_filters.len() + 2));
        }

        let query = if where_filters.is_empty() {
            base_query.to_string()
        } else {
            format!("{} WHERE {}", base_query, where_filters.join(" AND "))
        };

        let query = sqlx::query(&query);

        // Bind filters
        let query = query.bind(guild_id.to_string());

        let query = if let Some(user_id) = filters.user_id {
            query.bind(user_id.to_string())
        } else {
            query
        };

        let rows = query.fetch_all(&data.pool).await?;

        let mut entries = Vec::new();
        for row in rows {
            let id = row.try_get::<sqlx::types::Uuid, _>("id")?;
            let user_id = row.try_get::<String, _>("user_id")?;
            let guild_id = row.try_get::<String, _>("guild_id")?;
            let stings = row.try_get::<i32, _>("stings")?;
            let created_at = row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at")?;

            entries.push(sting_sources::FullStingEntry {
                entry: sting_sources::StingEntry {
                    user_id: user_id.to_string().parse()?,
                    guild_id: guild_id.to_string().parse()?,
                    stings,
                    reason: None,
                    void_reason: None,
                    action: sting_sources::Action::None,
                    state: sting_sources::StingState::Active,
                    duration: None,
                    creator: sting_sources::StingCreator::System,
                },
                created_at,
                id: id.to_string(),
            });
        }

        Ok(entries)
    }

    // No-op
    async fn create_sting_entry(
        &self,
        _ctx: &serenity::all::Context,
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
        ctx: &serenity::all::Context,
        id: String,
        entry: sting_sources::UpdateStingEntry,
    ) -> Result<(), silverpelt::Error> {
        // We only support editting stings for this source
        let Some(stings) = entry.stings else {
            return Ok(());
        };

        let data = ctx.data::<silverpelt::data::Data>();

        sqlx::query!(
            "UPDATE inspector__punishments SET stings = $1 WHERE id = $2",
            stings,
            id.parse::<sqlx::types::Uuid>()?,
        )
        .execute(&data.pool)
        .await?;

        Ok(())
    }

    async fn delete_sting_entry(
        &self,
        ctx: &serenity::all::Context,
        id: String,
    ) -> Result<(), silverpelt::Error> {
        sqlx::query!(
            "DELETE FROM inspector__punishments WHERE id = $1",
            id.parse::<sqlx::types::Uuid>()?
        )
        .execute(&ctx.data::<silverpelt::data::Data>().pool)
        .await?;

        Ok(())
    }
}

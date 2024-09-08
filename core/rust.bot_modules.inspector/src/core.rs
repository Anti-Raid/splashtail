use serenity::async_trait;
use silverpelt::sting_sources;

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
        sting_sources::StingSourceFlags::SUPPORTS_UPDATE
            | sting_sources::StingSourceFlags::SUPPORTS_DELETE
    }

    async fn count(
        &self,
        data: &sting_sources::StingSourceData,
        filters: sting_sources::StingCountFilters,
    ) -> Result<usize, silverpelt::Error> {
        let row = sqlx::query!(
            "SELECT COUNT(*) FROM inspector__punishments 
            WHERE ($1::TEXT IS NULL OR guild_id = $1::TEXT)
            AND ($2::TEXT IS NULL OR user_id = $2::TEXT)
            AND (
                $3::BOOL IS NULL OR 
                ($3 = true AND stings_expiry < NOW()) OR
                ($3 = false AND stings_expiry > NOW())
            )",
            filters.guild_id.map(|g| g.to_string()),
            filters.user_id.map(|u| u.to_string()),
            filters.expired,
        )
        .fetch_one(&data.pool)
        .await?;

        Ok(row.count.unwrap_or(0) as usize)
    }

    async fn fetch(
        &self,
        data: &sting_sources::StingSourceData,
        filters: sting_sources::StingFetchFilters,
    ) -> Result<Vec<sting_sources::FullStingEntry>, silverpelt::Error> {
        let rows = sqlx::query!(
            "SELECT id, user_id, guild_id, stings, created_at FROM inspector__punishments 
            WHERE ($1::TEXT IS NULL OR guild_id = $1::TEXT)
            AND ($2::TEXT IS NULL OR user_id = $2::TEXT)
            AND (
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
            entries.push(sting_sources::FullStingEntry {
                entry: sting_sources::StingEntry {
                    user_id: row.user_id.to_string().parse()?,
                    guild_id: row.guild_id.to_string().parse()?,
                    stings: row.stings,
                    reason: None,
                    void_reason: None,
                    action: sting_sources::Action::None,
                    state: sting_sources::StingState::Active,
                    duration: None,
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

    async fn update_sting_entry(
        &self,
        data: &sting_sources::StingSourceData,
        id: String,
        entry: sting_sources::UpdateStingEntry,
    ) -> Result<(), silverpelt::Error> {
        // We only support editting stings for this source
        let Some(stings) = entry.stings else {
            return Ok(());
        };

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
        data: &sting_sources::StingSourceData,
        id: String,
    ) -> Result<(), silverpelt::Error> {
        sqlx::query!(
            "DELETE FROM inspector__punishments WHERE id = $1",
            id.parse::<sqlx::types::Uuid>()?
        )
        .execute(&data.pool)
        .await?;

        Ok(())
    }
}

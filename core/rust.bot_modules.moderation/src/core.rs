use serenity::all::User;
use serenity::async_trait;
use silverpelt::sting_sources;
use splashcore_rs::utils::pg_interval_to_secs;
use std::str::FromStr;

pub(crate) struct ModerationActionsStingSource;

#[async_trait]
impl sting_sources::StingSource for ModerationActionsStingSource {
    fn id(&self) -> String {
        "moderation__actions".to_string()
    }

    fn description(&self) -> String {
        "Moderation Actions".to_string()
    }

    fn flags(&self) -> sting_sources::StingSourceFlags {
        sting_sources::StingSourceFlags::SUPPORTS_MANUAL_VOIDING
            | sting_sources::StingSourceFlags::SUPPORTS_DURATIONS
            | sting_sources::StingSourceFlags::SUPPORTS_ACTIONS
            | sting_sources::StingSourceFlags::SUPPORTS_UPDATE
            | sting_sources::StingSourceFlags::SUPPORTS_DELETE
    }

    async fn fetch(
        &self,
        data: &sting_sources::StingSourceData,
        filters: sting_sources::StingFetchFilters,
    ) -> Result<Vec<sting_sources::FullStingEntry>, silverpelt::Error> {
        let rows = sqlx::query!(
            "
            SELECT guild_id, user_id, moderator, action, stings, reason, duration, id, created_at, state, void_reason FROM moderation__actions
            WHERE (
                $1::TEXT IS NULL OR 
                guild_id = $1::TEXT
            ) AND (
                $2::TEXT IS NULL OR 
                user_id = $2::TEXT
            ) AND (
                $3::TEXT IS NULL OR
                state = $3::TEXT
            ) AND (
                $4::BOOL IS NULL OR 
                ($4 = true AND duration IS NOT NULL) OR 
                ($4 = false AND duration IS NULL)
            ) AND (
                $5::BOOL IS NULL OR 
                ($5 = true AND created_at + duration > NOW()) OR 
                ($5 = false AND created_at + duration < NOW()) 
            )",
            filters.guild_id.map(|g| g.to_string()),
            filters.user_id.map(|u| u.to_string()),
            filters.state.map(|s| s.to_string()),
            filters.has_duration,
            filters.expired,
        )
        .fetch_all(&data.pool)
        .await?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(sting_sources::FullStingEntry {
                entry: sting_sources::StingEntry {
                    user_id: row.user_id.parse()?,
                    guild_id: row.guild_id.parse()?,
                    stings: row.stings,
                    reason: row.reason,
                    void_reason: row.void_reason,
                    action: sting_sources::Action::from_str(&row.action)?,
                    state: sting_sources::StingState::from_str(&row.state)?,
                    duration: match row.duration {
                        Some(d) => Some(std::time::Duration::from_secs(u64::try_from(
                            pg_interval_to_secs(d),
                        )?)),
                        None => None,
                    },
                    creator: sting_sources::StingCreator::User(row.moderator.parse()?),
                },
                created_at: row.created_at,
                id: row.id.to_string(),
            });
        }

        Ok(entries)
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
        sqlx::query!(
            "UPDATE moderation__actions 
            SET stings = COALESCE($1, stings),
            reason = COALESCE($2, reason),
            duration = COALESCE($3, duration),
            action = COALESCE($4, action),
            state = COALESCE($5, state),
            void_reason = COALESCE($6, void_reason) 
            WHERE id = $7",
            entry.stings,
            entry.reason,
            entry
                .duration
                .map(|d| splashcore_rs::utils::secs_to_pg_interval_u64(d.as_secs())),
            entry.action.map(|a| a.to_string()),
            entry.state.map(|s| s.to_string()),
            entry.void_reason,
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
        sqlx::query!("DELETE FROM limits__user_actions WHERE action_id = $1", id)
            .execute(&data.pool)
            .await?;

        Ok(())
    }
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

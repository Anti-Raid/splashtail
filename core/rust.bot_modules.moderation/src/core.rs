use serenity::all::User;
use serenity::async_trait;
use silverpelt::sting_sources;
use splashcore_rs::utils::pg_interval_to_secs;
use sqlx::postgres::types::PgInterval;
use sqlx::Row;
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
        ctx: &serenity::all::Context,
        filters: sting_sources::StingFetchFilters,
    ) -> Result<Vec<sting_sources::FullStingEntry>, silverpelt::Error> {
        let base_query = "SELECT guild_id, user_id, moderator, action, stings, reason, duration, id, created_at, state, void_reason FROM moderation__actions";

        let mut where_filters = Vec::new();
        let mut total_binds = 0;

        // Guild ID filter
        if filters.guild_id.is_some() {
            where_filters.push(format!("guild_id = ${}", total_binds + 1));
            total_binds += 1;
        }

        // User ID filter
        if filters.user_id.is_some() {
            where_filters.push(format!("user_id = ${}", total_binds + 1));
            total_binds += 1;
        }

        // State filter
        if filters.state.is_some() {
            where_filters.push(format!("state = ${}", total_binds + 1));
        }

        // Has duration filter
        if let Some(has_duration) = filters.has_duration {
            if has_duration {
                where_filters.push("duration IS NOT NULL".to_string());
            } else {
                where_filters.push("duration IS NULL".to_string());
            }
        }

        // Expired filter
        if let Some(expired) = filters.expired {
            if expired {
                where_filters.push("created_at + duration < NOW()".to_string());
            } else {
                where_filters.push("created_at + duration > NOW()".to_string());
            }
        }

        let query = if where_filters.is_empty() {
            base_query.to_string()
        } else {
            format!("{} WHERE {}", base_query, where_filters.join(" AND "))
        };

        let query = sqlx::query(&query);

        // Bind filters
        let query = if let Some(guild_id) = filters.guild_id {
            query.bind(guild_id.to_string())
        } else {
            query
        };

        let query = if let Some(user_id) = filters.user_id {
            query.bind(user_id.to_string())
        } else {
            query
        };

        let query = if let Some(state) = filters.state {
            query.bind(state.to_string())
        } else {
            query
        };

        let rows = query
            .fetch_all(&ctx.data::<silverpelt::data::Data>().pool)
            .await?;

        let mut entries = Vec::new();
        for row in rows {
            let guild_id = row.try_get::<String, _>("guild_id")?;
            let user_id = row.try_get::<String, _>("user_id")?;
            let moderator = row.try_get::<String, _>("moderator")?;
            let action = row.try_get::<String, _>("action")?;
            let stings = row.try_get::<i32, _>("stings")?;
            let reason = row.try_get::<Option<String>, _>("reason")?;
            let duration = row.try_get::<Option<PgInterval>, _>("duration")?;
            let id = row.try_get::<sqlx::types::Uuid, _>("id")?;
            let created_at = row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at")?;
            let state = row.try_get::<String, _>("state")?;
            let void_reason = row.try_get::<Option<String>, _>("void_reason")?;

            entries.push(sting_sources::FullStingEntry {
                entry: sting_sources::StingEntry {
                    user_id: user_id.parse()?,
                    guild_id: guild_id.parse()?,
                    stings,
                    reason,
                    void_reason,
                    action: sting_sources::Action::from_str(&action)?,
                    state: sting_sources::StingState::from_str(&state)?,
                    duration: match duration {
                        Some(d) => Some(std::time::Duration::from_secs(u64::try_from(
                            pg_interval_to_secs(d),
                        )?)),
                        None => None,
                    },
                    creator: sting_sources::StingCreator::User(moderator.parse()?),
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

    async fn update_sting_entry(
        &self,
        ctx: &serenity::all::Context,
        id: String,
        entry: sting_sources::UpdateStingEntry,
    ) -> Result<(), silverpelt::Error> {
        let data = ctx.data::<silverpelt::data::Data>();

        let mut query = "UPDATE moderation__actions SET".to_string();

        let mut total_binds = 0;

        if entry.stings.is_some() {
            query.push_str(&format!(" stings = ${},", total_binds + 1));
            total_binds += 1;
        }

        if entry.reason.is_some() {
            query.push_str(&format!(" reason = ${},", total_binds + 1));
            total_binds += 1;
        }

        if entry.duration.is_some() {
            query.push_str(&format!(" duration = ${},", total_binds + 1));
            total_binds += 1;
        }

        if entry.action.is_some() {
            query.push_str(&format!(" action = ${},", total_binds + 1));
            total_binds += 1;
        }

        if entry.state.is_some() {
            query.push_str(&format!(" state = ${},", total_binds + 1));
            total_binds += 1;
        }

        if entry.void_reason.is_some() {
            query.push_str(&format!(" void_reason = ${},", total_binds + 1));
            total_binds += 1;
        }

        query.pop(); // Remove trailing comma

        query.push_str(format!(" WHERE id = ${}", total_binds + 1).as_str());

        let query = sqlx::query(&query);

        let query = if let Some(stings) = entry.stings {
            query.bind(stings)
        } else {
            query
        };

        let query = if let Some(reason) = entry.reason {
            query.bind(reason)
        } else {
            query
        };

        let query = if let Some(duration) = entry.duration {
            query.bind(duration)
        } else {
            query
        };

        let query = if let Some(action) = entry.action {
            query.bind(action.to_string())
        } else {
            query
        };

        let query = if let Some(state) = entry.state {
            query.bind(state.to_string())
        } else {
            query
        };

        let query = if let Some(void_reason) = entry.void_reason {
            query.bind(void_reason)
        } else {
            query
        };

        let query = query.bind(id.parse::<sqlx::types::Uuid>()?);

        query.execute(&data.pool).await?;

        Ok(())
    }

    async fn delete_sting_entry(
        &self,
        ctx: &serenity::all::Context,
        id: String,
    ) -> Result<(), silverpelt::Error> {
        sqlx::query!("DELETE FROM limits__user_actions WHERE action_id = $1", id)
            .execute(&ctx.data::<silverpelt::data::Data>().pool)
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

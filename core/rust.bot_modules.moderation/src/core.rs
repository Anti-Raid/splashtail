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

    async fn count(
        &self,
        data: &sting_sources::StingSourceData,
        filters: sting_sources::StingCountFilters,
    ) -> Result<usize, silverpelt::Error> {
        let row = sqlx::query!(
            "SELECT COUNT(*) FROM moderation__actions
            WHERE 
                ($1::TEXT IS NULL OR guild_id = $1::TEXT) AND 
                ($2::TEXT IS NULL OR user_id = $2::TEXT) AND (
                $3::BOOL IS NULL OR 
                ($3 = true AND (created_at + duration) < NOW()) OR
                ($3 = false AND (created_at + duration) > NOW())
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

pub(crate) mod punishment_actions {
    use async_trait::async_trait;
    use serenity::all::{EditMember, Timestamp};
    use silverpelt::punishments::{
        CreatePunishmentAction, GuildPunishment, PunishmentAction, PunishmentActionData,
        PunishmentUserAction,
    };

    pub struct CreateTimeoutAction;

    #[async_trait]
    impl CreatePunishmentAction for CreateTimeoutAction {
        fn name(&self) -> &'static str {
            "Timeout User"
        }

        fn syntax(&self) -> &'static str {
            "timeout"
        }

        fn to_punishment_action(
            &self,
            s: &str,
        ) -> Result<Option<PunishmentAction>, silverpelt::Error> {
            if s == "timeout" {
                Ok(Some(PunishmentAction::User(Box::new(TimeoutAction))))
            } else {
                Ok(None)
            }
        }
    }

    pub struct TimeoutAction;

    #[async_trait]
    impl PunishmentUserAction for TimeoutAction {
        fn creator(&self) -> Box<dyn CreatePunishmentAction> {
            Box::new(CreateTimeoutAction)
        }

        fn string_form(&self) -> String {
            "timeout".to_string()
        }

        async fn create(
            &self,
            data: &PunishmentActionData,
            member: &mut serenity::all::Member,
            _bot_member: &mut serenity::all::Member,
            _applied_punishments: &[GuildPunishment],
        ) -> Result<(), silverpelt::Error> {
            let timeout_duration = chrono::Duration::minutes(5);
            let new_time = chrono::Utc::now() + timeout_duration;

            member
                .edit(
                    &data.cache_http.http,
                    EditMember::new()
                        .disable_communication_until(Timestamp::from(new_time))
                        .audit_log_reason("[Punishment] Timeout applied to user"),
                )
                .await?;

            Ok(())
        }

        async fn revert(
            &self,
            data: &PunishmentActionData,
            member: &mut serenity::all::Member,
            _bot_member: &mut serenity::all::Member,
            _applied_punishments: &[GuildPunishment],
        ) -> Result<(), silverpelt::Error> {
            member
                .edit(
                    &data.cache_http.http,
                    EditMember::new()
                        .enable_communication()
                        .audit_log_reason("[Punishment] Timeout removed from user"),
                )
                .await?;

            Ok(())
        }
    }

    pub struct CreateKickAction;

    #[async_trait]
    impl CreatePunishmentAction for CreateKickAction {
        fn name(&self) -> &'static str {
            "Kick User"
        }

        fn syntax(&self) -> &'static str {
            "kick"
        }

        fn to_punishment_action(
            &self,
            s: &str,
        ) -> Result<Option<PunishmentAction>, silverpelt::Error> {
            if s == "kick" {
                Ok(Some(PunishmentAction::User(Box::new(KickAction))))
            } else {
                Ok(None)
            }
        }
    }

    pub struct KickAction;

    #[async_trait]
    impl PunishmentUserAction for KickAction {
        fn creator(&self) -> Box<dyn CreatePunishmentAction> {
            Box::new(CreateKickAction)
        }

        fn string_form(&self) -> String {
            "kick".to_string()
        }

        async fn create(
            &self,
            data: &PunishmentActionData,
            member: &mut serenity::all::Member,
            _bot_member: &mut serenity::all::Member,
            _applied_punishments: &[GuildPunishment],
        ) -> Result<(), silverpelt::Error> {
            member
                .kick(
                    &data.cache_http.http,
                    Some("[Punishment] User kicked from server"),
                )
                .await?;

            Ok(())
        }

        async fn revert(
            &self,
            _data: &PunishmentActionData,
            _member: &mut serenity::all::Member,
            _bot_member: &mut serenity::all::Member,
            _applied_punishments: &[GuildPunishment],
        ) -> Result<(), silverpelt::Error> {
            Ok(()) // No-op
        }
    }

    pub struct CreateBanAction;

    #[async_trait]
    impl CreatePunishmentAction for CreateBanAction {
        fn name(&self) -> &'static str {
            "Ban User"
        }

        fn syntax(&self) -> &'static str {
            "ban"
        }

        fn to_punishment_action(
            &self,
            s: &str,
        ) -> Result<Option<PunishmentAction>, silverpelt::Error> {
            if s == "ban" {
                Ok(Some(PunishmentAction::User(Box::new(BanAction))))
            } else {
                Ok(None)
            }
        }
    }

    pub struct BanAction;

    #[async_trait]
    impl PunishmentUserAction for BanAction {
        fn creator(&self) -> Box<dyn CreatePunishmentAction> {
            Box::new(CreateBanAction)
        }

        fn string_form(&self) -> String {
            "ban".to_string()
        }

        async fn create(
            &self,
            data: &PunishmentActionData,
            member: &mut serenity::all::Member,
            _bot_member: &mut serenity::all::Member,
            _applied_punishments: &[GuildPunishment],
        ) -> Result<(), silverpelt::Error> {
            member
                .ban(
                    &data.cache_http.http,
                    0,
                    Some("[Punishment] User banned from server"),
                )
                .await?;

            Ok(())
        }

        async fn revert(
            &self,
            _data: &PunishmentActionData,
            _member: &mut serenity::all::Member,
            _bot_member: &mut serenity::all::Member,
            _applied_punishments: &[GuildPunishment],
        ) -> Result<(), silverpelt::Error> {
            Ok(()) // No-op
        }
    }

    pub struct CreateRemoveAllRolesAction;

    #[async_trait]
    impl CreatePunishmentAction for CreateRemoveAllRolesAction {
        fn name(&self) -> &'static str {
            "Remove All Roles"
        }

        fn syntax(&self) -> &'static str {
            "remove_all_roles"
        }

        fn to_punishment_action(
            &self,
            s: &str,
        ) -> Result<Option<PunishmentAction>, silverpelt::Error> {
            if s == "remove_all_roles" {
                Ok(Some(PunishmentAction::User(Box::new(RemoveAllRolesAction))))
            } else {
                Ok(None)
            }
        }
    }

    pub struct RemoveAllRolesAction;

    #[async_trait]
    impl PunishmentUserAction for RemoveAllRolesAction {
        fn creator(&self) -> Box<dyn CreatePunishmentAction> {
            Box::new(CreateRemoveAllRolesAction)
        }

        fn string_form(&self) -> String {
            "remove_all_roles".to_string()
        }

        async fn create(
            &self,
            data: &PunishmentActionData,
            member: &mut serenity::all::Member,
            _bot_member: &mut serenity::all::Member,
            _applied_punishments: &[GuildPunishment],
        ) -> Result<(), silverpelt::Error> {
            member
                .edit(
                    &data.cache_http.http,
                    EditMember::new()
                        .roles(Vec::new())
                        .audit_log_reason("[Punishment] All roles removed from user"),
                )
                .await?;

            Ok(())
        }

        /// TODO: Implement this
        async fn revert(
            &self,
            _data: &PunishmentActionData,
            _member: &mut serenity::all::Member,
            _bot_member: &mut serenity::all::Member,
            _applied_punishments: &[GuildPunishment],
        ) -> Result<(), silverpelt::Error> {
            Ok(()) // No-op
        }
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

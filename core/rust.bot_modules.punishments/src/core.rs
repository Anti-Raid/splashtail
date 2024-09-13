use async_trait::async_trait;
use dashmap::DashMap;
use proxy_support::{guild, member_in_guild};
use serde::{Deserialize, Serialize};
use serenity::all::{EditMember, GuildId, Timestamp, UserId};
use silverpelt::module_config::is_module_enabled;
use silverpelt::sting_sources::{self, FullStingEntry, StingFetchFilters};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

pub static PUNISHMENT_ACTIONS: LazyLock<DashMap<String, PunishmentAction>> = LazyLock::new(|| {
    let map: DashMap<String, PunishmentAction> = DashMap::new();

    map
});

pub struct PunishmentActionData {
    pub cache_http: botox::cache::CacheHttpImpl,
    pub pool: sqlx::PgPool,
    pub reqwest: reqwest::Client,
    pub object_store: Arc<splashcore_rs::objectstore::ObjectStore>,
}

/// Trait for creating a punishment action
#[async_trait]
pub trait CreatePunishmentAction
where
    Self: Send + Sync,
{
    /// Returns the syntax for the action
    ///
    /// E.g. `ban` for banning a user
    fn syntax(&self) -> &'static str;

    /// Given the string form of the action, returns the action
    fn to_punishment_action(&self, s: &str) -> Result<Option<PunishmentAction>, silverpelt::Error>;
}

pub enum PunishmentAction {
    User(Box<dyn PunishmentUserAction>),
    Global(Box<dyn PunishmentGlobalAction>),
}

impl PunishmentAction {
    pub fn creator(&self) -> Box<dyn CreatePunishmentAction> {
        match self {
            Self::User(action) => action.creator(),
            Self::Global(action) => action.creator(),
        }
    }

    pub fn string_form(&self) -> String {
        match self {
            Self::User(action) => action.string_form(),
            Self::Global(action) => action.string_form(),
        }
    }
}

#[async_trait]
pub trait PunishmentUserAction
where
    Self: Send + Sync,
{
    /// Returns the creator for the punishment action
    fn creator(&self) -> Box<dyn CreatePunishmentAction>;

    /// Returns the string form of the punishment action
    fn string_form(&self) -> String;

    /// Applies a punishment to the target
    async fn create(
        &self,
        data: &PunishmentActionData,
        member: &mut serenity::all::Member,
        bot_member: &mut serenity::all::Member,
        applied_punishments: &[GuildPunishment],
    ) -> Result<(), silverpelt::Error>;

    /// Attempts to revert a punishment from the target
    async fn revert(
        &self,
        data: &PunishmentActionData,
        member: &mut serenity::all::Member,
        bot_member: &mut serenity::all::Member,
        applied_punishments: &[GuildPunishment],
    ) -> Result<(), silverpelt::Error>;
}

#[async_trait]
pub trait PunishmentGlobalAction
where
    Self: Send + Sync,
{
    /// Returns the creator for the punishment action
    fn creator(&self) -> Box<dyn CreatePunishmentAction>;

    /// Returns the string form of the punishment action
    fn string_form(&self) -> String;

    /// Applies a punishment
    async fn create(
        &self,
        data: &PunishmentActionData,
        partial_guild: &mut serenity::all::PartialGuild,
        bot_member: &mut serenity::all::Member,
        applied_punishments: &[GuildPunishment],
    ) -> Result<(), silverpelt::Error>;

    /// Attempts to revert a punishment from the target
    async fn revert(
        &self,
        data: &PunishmentActionData,
        partial_guild: &mut serenity::all::PartialGuild,
        bot_member: &mut serenity::all::Member,
        applied_punishments: &[GuildPunishment],
    ) -> Result<(), silverpelt::Error>;
}

/// Given a string, returns the punishment action
pub fn from_punishment_action_string(s: &str) -> Result<PunishmentAction, silverpelt::Error> {
    for pair in PUNISHMENT_ACTIONS.iter() {
        let creator = pair.value().creator();
        if let Some(m) = creator.to_punishment_action(s)? {
            return Ok(m);
        }
    }

    Err("Unknown punishment".into())
}

/// Serde serialization for PunishmentAction
impl Serialize for PunishmentAction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.string_form().serialize(serializer)
    }
}

/// Serde deserialization for PunishmentAction
impl<'de> Deserialize<'de> for PunishmentAction {
    fn deserialize<D>(deserializer: D) -> Result<PunishmentAction, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        // Call `from_lockdown_mode_string` to get the lockdown mode
        from_punishment_action_string(&s).map_err(serde::de::Error::custom)
    }
}

pub type StingEntryMap = HashMap<String, Vec<FullStingEntry>>;

/// Returns all sting entries that a server has. This can be useful when triggering punishments to users
/// or just showing them a user friendly list of all the stings
#[allow(dead_code)]
pub async fn get_consolidated_sting_entries(
    ctx: &serenity::all::Context,
    guild_id: GuildId,
    filters: Option<StingFetchFilters>,
) -> Result<StingEntryMap, silverpelt::Error> {
    let source_data = sting_sources::StingSourceData::from_ctx(ctx);

    if !is_module_enabled(
        &source_data.silverpelt_cache,
        &source_data.pool,
        guild_id,
        "punishments",
    )
    .await?
    {
        // Punishments module is not enabled
        return Err("Punishments module is not enabled".into());
    }

    let filters = filters.unwrap_or(StingFetchFilters {
        guild_id: Some(guild_id),
        ..Default::default()
    });

    let mut stings = HashMap::new();

    for (_, module) in source_data.silverpelt_cache.module_cache.iter() {
        for source in module.sting_sources.iter() {
            let entries = source.fetch(&source_data, filters.clone()).await?;

            stings.insert(source.id(), entries);
        }
    }

    Ok(stings)
}

/// This struct stores a guild punishment that can then be used to trigger punishments
/// on a user through the bot
#[derive(Serialize, Deserialize, Clone)]
pub struct GuildPunishment {
    pub id: String,
    pub guild_id: GuildId,
    pub creator: UserId,
    pub stings: i32,
    pub action: Arc<PunishmentAction>,
    pub duration: Option<i32>,
    pub modifiers: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// A guild punishment list is internally a Vec<GuildPunishment> but has special methods
/// to make things easier when coding punishments
///
/// Note that the guild punishment list should not be modified directly
#[derive(Serialize, Deserialize, Clone)]
pub struct GuildPunishmentList {
    punishments: Vec<GuildPunishment>,
}

impl GuildPunishmentList {
    /// Gets the punishment list of a specific guild
    pub async fn guild(
        ctx: &serenity::all::Context,
        guild_id: GuildId,
    ) -> Result<Self, silverpelt::Error> {
        let data = ctx.data::<silverpelt::data::Data>();
        let rec = sqlx::query!(
            "SELECT id, guild_id, creator, stings, action, modifiers, created_at, EXTRACT(seconds FROM duration)::integer AS duration FROM punishments__guild_punishment_list WHERE guild_id = $1",
            guild_id.to_string(),
        )
        .fetch_all(&data.pool)
        .await?;

        let mut punishments = vec![];

        for row in rec {
            punishments.push(GuildPunishment {
                id: row.id.to_string(),
                guild_id: row.guild_id.parse::<GuildId>()?,
                creator: row.creator.parse::<UserId>()?,
                stings: row.stings,
                action: {
                    let action = from_punishment_action_string(&row.action);

                    let Ok(action) = action else {
                        continue; // Skip this punishment if the action is not found
                    };

                    action.into()
                },
                modifiers: row.modifiers,
                duration: row.duration,
                created_at: row.created_at,
            });
        }

        Ok(Self { punishments })
    }

    /// Returns the list of punishments
    ///
    /// This is a method to ensure that the returned list is not modified (is immutable)
    #[allow(dead_code)]
    pub fn punishments(&self) -> &Vec<GuildPunishment> {
        &self.punishments
    }

    /// Filter returns a new GuildPunishmentList with only the punishments that match the set of filters
    ///
    /// Note that this drops the existing punishment list
    pub fn filter(&self, stings: i32) -> Vec<GuildPunishment> {
        let mut punishments = vec![];

        for punishment in self.punishments.iter() {
            if punishment.stings <= stings {
                punishments.push(punishment.clone());
            }
        }

        punishments
    }
}

/// Returns (per_user_sting_counts, total_system_punishments, total sting count)
pub fn get_sting_counts(sting_entries_map: &StingEntryMap) -> (HashMap<UserId, i32>, i32, i32) {
    let mut per_user_sting_counts = HashMap::new();

    let mut total_system_punishments = 0;
    for (_src, entries) in sting_entries_map {
        for entry in entries {
            match entry.entry.creator {
                sting_sources::StingCreator::User(user_id) => {
                    let count = per_user_sting_counts.entry(user_id).or_insert(0);
                    *count += 1;
                }
                sting_sources::StingCreator::System => {
                    total_system_punishments += 1;
                }
            }
        }
    }

    // Total sting count is the sum of all user stings and system stings prior to adding the system stings to all users
    let mut total_sting_count = total_system_punishments;
    for (_, count) in per_user_sting_counts.iter() {
        total_sting_count += count;
    }

    // Now add the total system punishments to all users
    for (_, count) in per_user_sting_counts.iter_mut() {
        *count += total_system_punishments;
    }

    (
        per_user_sting_counts,
        total_system_punishments,
        total_sting_count,
    )
}

pub async fn trigger_punishment(
    ctx: &serenity::all::Context,
    guild_id: GuildId,
    _creator: sting_sources::StingCreator,
) -> Result<(), silverpelt::Error> {
    let sting_entries = get_consolidated_sting_entries(ctx, guild_id, None).await?;

    if sting_entries.is_empty() {
        return Ok(());
    }

    let punishments = GuildPunishmentList::guild(ctx, guild_id).await?;

    if punishments.punishments().is_empty() {
        return Ok(());
    }

    let (per_user_sting_counts, _total_system_punishments, total_sting_count) =
        get_sting_counts(&sting_entries);

    let data = ctx.data::<silverpelt::data::Data>();
    let cache_http = botox::cache::CacheHttpImpl::from_ctx(ctx);

    let bot_userid = ctx.cache.current_user().id;
    let Some(mut bot) = member_in_guild(&cache_http, &data.reqwest, guild_id, bot_userid).await?
    else {
        return Err("Bot not found".into());
    };

    let punishment_data = PunishmentActionData {
        cache_http: cache_http.clone(),
        pool: data.pool.clone(),
        reqwest: data.reqwest.clone(),
        object_store: data.object_store.clone(),
    };

    let mut guild = guild(&cache_http, &data.reqwest, guild_id).await?;

    // First apply global punishments, the total sting count is used for this
    let system_punishments = punishments.filter(total_sting_count);

    if !system_punishments.is_empty() {
        for punishment in system_punishments.iter() {
            match &*punishment.action {
                PunishmentAction::Global(action) => {
                    action
                        .create(&punishment_data, &mut guild, &mut bot, &system_punishments)
                        .await?;
                }
                _ => {}
            }
        }
    }

    for (user_id, sting_count) in per_user_sting_counts {
        let Some(mut user) = member_in_guild(&cache_http, &data.reqwest, guild_id, user_id).await?
        else {
            return Ok(());
        };

        if guild
            .greater_member_hierarchy(&bot, &user)
            .unwrap_or(user.user.id)
            == user.user.id
        {
            return Err(
                "Bot does not have the required permissions to carry out this action".into(),
            );
        }

        let punishments = punishments.filter(sting_count.try_into()?);

        for punishment in punishments.iter() {
            match &*punishment.action {
                PunishmentAction::User(action) => {
                    action
                        .create(&punishment_data, &mut user, &mut bot, &punishments)
                        .await?;
                }
                _ => {}
            }
        }
    }

    Ok(())
}

pub mod timeout {
    use super::*;

    pub struct CreateTimeoutAction;

    #[async_trait]
    impl CreatePunishmentAction for CreateTimeoutAction {
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
}

pub mod kick {
    use super::*;

    pub struct CreateKickAction;

    #[async_trait]
    impl CreatePunishmentAction for CreateKickAction {
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
}

pub mod ban {
    use super::*;

    pub struct CreateBanAction;

    #[async_trait]
    impl CreatePunishmentAction for CreateBanAction {
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
}

pub mod remove_all_roles {
    use super::*;

    pub struct CreateRemoveAllRolesAction;

    #[async_trait]
    impl CreatePunishmentAction for CreateRemoveAllRolesAction {
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

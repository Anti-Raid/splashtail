use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serenity::all::{GuildId, UserId};
use std::sync::Arc;

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
    /// Name of the action
    fn name(&self) -> &'static str;

    /// Returns the syntax for the action
    ///
    /// E.g. `ban` for banning a user
    fn syntax(&self) -> &'static str;

    /// Given the string form of the action, returns the action
    fn to_punishment_action(&self, s: &str) -> Result<Option<Box<dyn PunishmentAction>>, crate::Error>;
}

/// Display impl for CreatePunishmentAction
impl std::fmt::Display for dyn CreatePunishmentAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name(), self.syntax())
    }
}

#[async_trait]
pub trait PunishmentAction
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
        user_id: UserId,
        bot_member: &mut serenity::all::Member,
        reason: String,
    ) -> Result<(), crate::Error>;

    /// Attempts to revert a punishment from the target
    async fn revert(
        &self,
        data: &PunishmentActionData,
        user_id: UserId,
        bot_member: &mut serenity::all::Member,
        reason: String,
    ) -> Result<(), crate::Error>;
}

/// Display impl for PunishmentAction
impl std::fmt::Display for dyn PunishmentAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({}) [per-user]",
            self.creator().name(),
            self.string_form()
        )
    }
}

/// Given a guild ID and a silverpelt::Data, return the punishment actions for the guild
pub async fn get_punishment_actions_for_guild(
    guild_id: serenity::all::GuildId,
    data: &crate::data::Data,
) -> Result<Vec<Arc<dyn CreatePunishmentAction>>, crate::Error> {
    let mut actions_map = Vec::new();
    for refs in data.silverpelt_cache.module_cache.iter() {
        let module = refs.value();
        if !crate::module_config::is_module_enabled(
            &data.silverpelt_cache,
            &data.pool,
            guild_id,
            module.id(),
        )
        .await?
        {
            continue;
        }

        if !module.punishment_actions().is_empty() {
            actions_map.extend(module.punishment_actions().iter().cloned());
        }
    }

    Ok(actions_map)
}

/// Given a string, returns the punishment action
pub fn from_punishment_action_string(
    actions: &[Arc<dyn CreatePunishmentAction>],
    s: &str,
) -> Result<Box<dyn PunishmentAction>, crate::Error> {
    for action in actions.iter() {
        if let Some(m) = action.to_punishment_action(s)? {
            return Ok(m);
        }
    }

    Err("Unknown punishment".into())
}

/// Serde serialization for PunishmentAction
impl Serialize for dyn PunishmentAction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.string_form().serialize(serializer)
    }
}

/// This struct stores a guild punishment that can then be used to trigger punishments
/// on a user through the bot
#[derive(Clone)]
pub struct GuildPunishment {
    pub id: String,
    pub guild_id: GuildId,
    pub creator: UserId,
    pub stings: i32,
    pub action: Arc<dyn PunishmentAction>,
    pub duration: Option<i32>,
    pub modifiers: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl GuildPunishment {
    pub fn to_canonical(&self) -> CanonicalGuildPunishment {
        CanonicalGuildPunishment {
            id: self.id.clone(),
            guild_id: self.guild_id,
            creator: self.creator,
            stings: self.stings,
            action: self.action.string_form(),
            duration: self.duration,
            modifiers: self.modifiers.clone(),
            created_at: self.created_at,
        }
    }

    pub fn from_canonical(
        &self,
        actions: &[Arc<dyn CreatePunishmentAction>],
        canonical: CanonicalGuildPunishment,
    ) -> Result<Self, crate::Error> {
        let action = from_punishment_action_string(actions, &canonical.action)?;

        Ok(Self {
            id: canonical.id,
            guild_id: canonical.guild_id,
            creator: canonical.creator,
            stings: canonical.stings,
            action: action.into(),
            duration: canonical.duration,
            modifiers: canonical.modifiers,
            created_at: canonical.created_at,
        })
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CanonicalGuildPunishment {
    pub id: String,
    pub guild_id: GuildId,
    pub creator: UserId,
    pub stings: i32,
    pub action: String,
    pub duration: Option<i32>,
    pub modifiers: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

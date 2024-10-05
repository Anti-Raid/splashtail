use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serenity::all::{GuildId, UserId};
use std::{str::FromStr, sync::Arc};

/// A punishment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildPunishment {
    /// The ID of the applied punishment
    pub id: sqlx::types::Uuid,
    /// The module name
    pub module: String,
    /// Src of the sting, this can be useful if a module wants to store the source of the sting
    pub src: Option<String>,
    /// The guild id of the applied punishment
    pub guild_id: GuildId,
    /// The punishment string
    pub punishment: String,
    /// Creator of the punishment
    pub creator: PunishmentTarget,
    /// The target of the punishment
    pub target: PunishmentTarget,
    /// The handle log encountered while handling the punishment
    pub handle_log: serde_json::Value,
    /// When the punishment was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Duration of the punishment
    pub duration: Option<std::time::Duration>,
    /// The reason for the punishment
    pub reason: String,
    /// Extra misc data
    pub data: Option<serde_json::Value>,
}

impl GuildPunishment {
    pub async fn get_expired(
        db: impl sqlx::PgExecutor<'_>,
    ) -> Result<Vec<GuildPunishment>, crate::Error> {
        let rec = sqlx::query!(
            "SELECT id, module, src, guild_id, punishment, creator, target, handle_log, created_at, duration, reason, data FROM punishments WHERE duration IS NOT NULL AND (created_at + duration) < NOW()",
        )
        .fetch_all(db)
        .await?;

        let mut stings = Vec::new();

        for row in rec {
            stings.push(GuildPunishment {
                id: row.id,
                module: row.module,
                src: row.src,
                guild_id: row.guild_id.parse()?,
                punishment: row.punishment,
                creator: PunishmentTarget::from_str(&row.creator)?,
                target: PunishmentTarget::from_str(&row.target)?,
                handle_log: row.handle_log,
                created_at: row.created_at,
                duration: row.duration.map(|d| {
                    let secs = splashcore_rs::utils::pg_interval_to_secs(d);
                    std::time::Duration::from_secs(secs.try_into().unwrap())
                }),
                reason: row.reason,
                data: row.data,
            });
        }

        Ok(stings)
    }
}

/// Data required to create a punishment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PunishmentCreate {
    /// The module name
    pub module: String,
    /// Src of the sting, this can be useful if a module wants to store the source of the sting
    pub src: Option<String>,
    /// The guild id of the applied punishment
    pub guild_id: GuildId,
    /// The punishment string
    pub punishment: String,
    /// Creator of the punishment
    pub creator: PunishmentTarget,
    /// The target of the punishment
    pub target: PunishmentTarget,
    /// The handle log encountered while handling the punishment
    pub handle_log: serde_json::Value,
    /// Duration of the punishment
    pub duration: Option<std::time::Duration>,
    /// The reason for the punishment
    pub reason: String,
    /// Extra misc data
    pub data: Option<serde_json::Value>,
}

impl PunishmentCreate {
    pub async fn create(
        self,
        db: impl sqlx::PgExecutor<'_>,
    ) -> Result<GuildPunishment, crate::Error> {
        let ret_data = sqlx::query!(
            r#"
            INSERT INTO punishments (module, src, guild_id, punishment, creator, target, handle_log, duration, reason, data)
            VALUES ($1, $2, $3, $4, $5, $6, $7, make_interval(secs => $8), $9, $10) RETURNING id, created_at
            "#,
            self.module,
            self.src,
            self.guild_id.to_string(),
            self.punishment,
            self.creator.to_string(),
            self.target.to_string(),
            self.handle_log,
            self.duration.map(|d| d.as_secs() as f64),
            self.reason,
            self.data
        )
        .fetch_one(db)
        .await?;

        Ok(GuildPunishment {
            id: ret_data.id,
            module: self.module,
            src: self.src,
            guild_id: self.guild_id,
            punishment: self.punishment,
            creator: self.creator,
            target: self.target,
            handle_log: self.handle_log,
            created_at: ret_data.created_at,
            duration: self.duration,
            reason: self.reason,
            data: self.data,
        })
    }
}

/// A punishment target (either user or system)
#[derive(Debug, Clone, Copy)]
pub enum PunishmentTarget {
    /// The punishment was created by a user
    User(UserId),
    /// The punishment was created by the system
    System,
}

impl std::fmt::Display for PunishmentTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PunishmentTarget::User(user_id) => write!(f, "user:{}", user_id),
            PunishmentTarget::System => write!(f, "system"),
        }
    }
}

impl std::str::FromStr for PunishmentTarget {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "system" {
            Ok(PunishmentTarget::System)
        } else {
            let user_id = s
                .strip_prefix("user:")
                .ok_or_else(|| format!("Invalid sting creator: {}", s))?;
            Ok(PunishmentTarget::User(
                user_id
                    .parse()
                    .map_err(|e| format!("Invalid user ID: {}", e))?,
            ))
        }
    }
}

// Serde impls for PunishmentTarget
impl Serialize for PunishmentTarget {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

impl<'de> Deserialize<'de> for PunishmentTarget {
    fn deserialize<D>(deserializer: D) -> Result<PunishmentTarget, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        PunishmentTarget::from_str(&s).map_err(serde::de::Error::custom)
    }
}

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
    fn to_punishment_action(
        &self,
        s: &str,
    ) -> Result<Option<Box<dyn PunishmentAction>>, crate::Error>;
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

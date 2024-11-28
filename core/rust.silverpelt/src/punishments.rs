use serde::{Deserialize, Serialize};
use serenity::all::{GuildId, UserId};
use std::str::FromStr;

/// A punishment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Punishment {
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
    /// Is Handled
    pub is_handled: bool,
    /// Extra misc data
    pub data: Option<serde_json::Value>,
}

impl Punishment {
    pub async fn get_expired(
        db: impl sqlx::PgExecutor<'_>,
    ) -> Result<Vec<Punishment>, crate::Error> {
        let rec = sqlx::query!(
            "SELECT id, module, src, guild_id, punishment, creator, target, is_handled, handle_log, created_at, duration, reason, data FROM punishments WHERE duration IS NOT NULL AND is_handled = false AND (created_at + duration) < NOW()",
        )
        .fetch_all(db)
        .await?;

        let mut punishments = Vec::new();

        for row in rec {
            punishments.push(Punishment {
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
                is_handled: row.is_handled,
                reason: row.reason,
                data: row.data,
            });
        }

        Ok(punishments)
    }

    /// Dispatch a PunishmentCreate event
    pub async fn dispatch_event(self, ctx: serenity::all::Context) -> Result<(), crate::Error> {
        crate::ar_event::dispatch_event_to_modules(&crate::ar_event::EventHandlerContext {
            guild_id: self.guild_id,
            data: ctx.data::<crate::data::Data>(),
            event: crate::ar_event::AntiraidEvent::PunishmentCreate(self.into()),
            serenity_context: ctx,
        })
        .await
        .map_err(|e| {
            format!("Failed to dispatch event: {}", {
                let mut strs = String::new();

                for err in e {
                    strs.push_str(&format!("{}\n", err));
                }

                strs
            })
        })?;

        Ok(())
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
    pub fn to_punishment(
        self,
        id: sqlx::types::Uuid,
        created_at: chrono::DateTime<chrono::Utc>,
        is_handled: bool,
    ) -> Punishment {
        Punishment {
            id,
            created_at,
            is_handled,
            module: self.module,
            src: self.src,
            guild_id: self.guild_id,
            punishment: self.punishment,
            creator: self.creator,
            target: self.target,
            handle_log: self.handle_log,
            duration: self.duration,
            reason: self.reason,
            data: self.data,
        }
    }

    /// Creates a new Punishment without dispatching it as an event
    pub async fn create_without_dispatch(
        self,
        db: impl sqlx::PgExecutor<'_>,
    ) -> Result<Punishment, crate::Error> {
        let ret_data = sqlx::query!(
            r#"
            INSERT INTO punishments (module, src, guild_id, punishment, creator, target, handle_log, duration, reason, data)
            VALUES ($1, $2, $3, $4, $5, $6, $7, make_interval(secs => $8), $9, $10) RETURNING id, created_at, is_handled
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

        Ok(self.to_punishment(ret_data.id, ret_data.created_at, ret_data.is_handled))
    }

    /// Creates a new Punishment and dispatches it as an event in one go
    pub async fn create_and_dispatch(
        self,
        ctx: serenity::all::Context,
        db: impl sqlx::PgExecutor<'_>,
    ) -> Result<(), crate::Error> {
        let punishment = self.create_without_dispatch(db).await?;

        punishment.dispatch_event(ctx).await?;

        Ok(())
    }

    /// Creates a new Punishment and dispatches it as an event in one go
    pub async fn create_and_dispatch_returning_id(
        self,
        ctx: serenity::all::Context,
        db: impl sqlx::PgExecutor<'_>,
    ) -> Result<sqlx::types::Uuid, crate::Error> {
        let punishment = self.create_without_dispatch(db).await?;
        let sid = punishment.id;

        punishment.dispatch_event(ctx).await?;

        Ok(sid)
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

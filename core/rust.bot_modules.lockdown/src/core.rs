use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;

use crate::priority_handles::PrioritySet;

/// Change operation, commonly used in lockdown modes
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Hash, PartialEq)]
pub enum ChangeOp {
    Add,
    Remove,
}

impl std::fmt::Display for ChangeOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeOp::Add => write!(f, "Add"),
            ChangeOp::Remove => write!(f, "Remove"),
        }
    }
}

/// Returns the critical roles given a [PartialGuild](`serenity::all::PartialGuild`) and a set of member roles
pub fn get_critical_roles(
    pg: &serenity::all::PartialGuild,
    member_roles: &HashSet<serenity::all::RoleId>,
) -> Result<HashSet<serenity::all::RoleId>, silverpelt::Error> {
    if member_roles.is_empty() {
        // Find the everyone role
        let everyone_role = pg
            .roles
            .iter()
            .find(|r| r.id.get() == pg.id.get())
            .ok_or_else(|| silverpelt::Error::from("No @everyone role found"))?;

        Ok(std::iter::once(everyone_role.id).collect())
    } else {
        Ok(member_roles.clone())
    }
}

pub struct LockdownData {
    pub cache_http: botox::cache::CacheHttpImpl,
    pub pool: sqlx::PgPool,
    pub reqwest: reqwest::Client,
    pub object_store: Arc<splashcore_rs::objectstore::ObjectStore>,
}

impl LockdownData {
    pub fn from_settings_data(data: &module_settings::types::SettingsData) -> Self {
        Self {
            cache_http: data.cache_http.clone(),
            pool: data.pool.clone(),
            reqwest: data.reqwest.clone(),
            object_store: data.object_store.clone(),
        }
    }
}

pub trait LockdownTestResult
where
    Self: Send + Sync,
{
    /// Returns whether the lockdown can be applied perfectly with the current server layout
    fn can_apply_perfectly(&self) -> bool;

    /// Returns a string representation of the result
    fn display_result(&self, pg: &serenity::all::PartialGuild) -> String;
}

/// To ensure two lockdowns don't conflict with each other, we need some information about what all lockdowns are handling
/// along with what specificity they have
pub struct LockdownModeHandle {
    pub roles: HashSet<serenity::all::RoleId>,
    pub channels: HashSet<serenity::all::ChannelId>,
}

/// To ensure two lockdowns don't conflict with each other, we need some information about what all lockdowns are handling
/// along with what specificity they have
pub struct LockdownModeHandles {
    pub roles: PrioritySet<serenity::all::RoleId>,
    pub channels: PrioritySet<serenity::all::ChannelId>,
}

impl LockdownModeHandles {
    /// `add_handle` adds a handle to the set given the specificity of the handle
    pub fn add_handle(&mut self, handle: LockdownModeHandle, specificity: usize) {
        for role in handle.roles {
            self.roles.add(role, specificity);
        }

        for channel in handle.channels {
            self.channels.add(channel, specificity);
        }
    }

    pub fn remove_handle(&mut self, handle: &LockdownModeHandle, specificity: usize) {
        for role in handle.roles.iter() {
            self.roles.remove(*role, specificity);
        }

        for channel in handle.channels.iter() {
            self.channels.remove(*channel, specificity);
        }
    }

    // A role is locked if it contains all roles of the current *with a lower specificity*
    pub fn is_role_locked(
        &self,
        role: serenity::all::RoleId,
        specificity: usize,
    ) -> Option<(serenity::all::RoleId, usize)> {
        if let Some(current_spec) = self.roles.highest_priority(&role) {
            if current_spec >= specificity {
                return Some((role, current_spec));
            }
        }

        None
    }

    // A channel is locked if it contains all channels of the current *with a lower specificity*
    pub fn is_channel_locked(
        &self,
        channel: serenity::all::ChannelId,
        specificity: usize,
    ) -> Option<(serenity::all::ChannelId, usize)> {
        if let Some(current_spec) = self.channels.highest_priority(&channel) {
            if current_spec >= specificity {
                return Some((channel, current_spec));
            }
        }

        None
    }

    // A handle is redundant if it contains all roles and channels of the current *with a lower specificity*
    pub fn is_redundant(&self, other: &LockdownModeHandle, other_spec: usize) -> bool {
        for role in other.roles.iter() {
            if let Some(current_spec) = self.roles.highest_priority(role) {
                if current_spec >= other_spec {
                    return false;
                }
            } else {
                return false;
            }
        }

        for channel in other.channels.iter() {
            if let Some(current_spec) = self.channels.highest_priority(channel) {
                if current_spec >= other_spec {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

#[async_trait]
pub trait LockdownMode
where
    Self: Send + Sync,
{
    /// All lockdowns will be sorted by this value, with the highest value being the most specific and hence viewed first
    fn specificity(&self) -> usize;

    async fn test(
        &self,
        lockdown_data: &LockdownData,
        pg: &serenity::all::PartialGuild,
        pgc: &[serenity::all::GuildChannel],
        critical_roles: &HashSet<serenity::all::RoleId>,
    ) -> Result<Box<dyn LockdownTestResult>, silverpelt::Error>;

    /// Sets up the lockdown mode, returning any data to be stored in database
    async fn setup(
        &self,
        lockdown_data: &LockdownData,
        pg: &serenity::all::PartialGuild,
        pgc: &[serenity::all::GuildChannel],
        critical_roles: &HashSet<serenity::all::RoleId>,
    ) -> Result<serde_json::Value, silverpelt::Error>;

    async fn create(
        &self,
        lockdown_data: &LockdownData,
        pg: &mut serenity::all::PartialGuild,
        pgc: &mut [serenity::all::GuildChannel],
        critical_roles: &HashSet<serenity::all::RoleId>,
        data: &serde_json::Value,
        all_handles: &LockdownModeHandles,
    ) -> Result<(), silverpelt::Error>;

    async fn revert(
        &self,
        lockdown_data: &LockdownData,
        pg: &mut serenity::all::PartialGuild,
        pgc: &mut [serenity::all::GuildChannel],
        critical_roles: &HashSet<serenity::all::RoleId>,
        data: &serde_json::Value,
        all_handles: &LockdownModeHandles,
    ) -> Result<(), silverpelt::Error>;

    async fn handles(
        &self,
        lockdown_data: &LockdownData,
        pg: &serenity::all::PartialGuild,
        pgc: &[serenity::all::GuildChannel],
        critical_roles: &HashSet<serenity::all::RoleId>,
        data: &serde_json::Value,
    ) -> Result<LockdownModeHandle, silverpelt::Error>;
}

pub trait CustomLockdownMode: LockdownMode + std::fmt::Display {}

/// Enum containing all variants
pub enum LockdownModes {
    QuickServerLockdown(qsl::QuickServerLockdown),
    TraditionalServerLockdown(tsl::TraditionalServerLockdown),
    SingleChannelLockdown(scl::SingleChannelLockdown),
    Unknown(Box<dyn CustomLockdownMode>),
}

impl LockdownModes {
    pub fn from_string(s: &str) -> Result<Option<LockdownModes>, silverpelt::Error> {
        if s == "qsl" {
            Ok(Some(LockdownModes::QuickServerLockdown(
                qsl::QuickServerLockdown,
            )))
        } else if s == "tsl" {
            Ok(Some(LockdownModes::TraditionalServerLockdown(
                tsl::TraditionalServerLockdown,
            )))
        } else if s.starts_with("scl/") {
            let channel_id = s
                .strip_prefix("scl/")
                .ok_or_else(|| silverpelt::Error::from("Invalid SCL string"))?
                .parse()
                .map_err(|e| format!("Error while parsing channel ID: {}", e))?;
            Ok(Some(LockdownModes::SingleChannelLockdown(
                scl::SingleChannelLockdown(channel_id),
            )))
        } else {
            Ok(None)
        }
    }
}

impl std::fmt::Display for LockdownModes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LockdownModes::QuickServerLockdown(_) => write!(f, "qsl"),
            LockdownModes::TraditionalServerLockdown(_) => write!(f, "tsl"),
            LockdownModes::SingleChannelLockdown(scl) => {
                write!(f, "scl/{}", scl.0)
            }
            LockdownModes::Unknown(m) => write!(f, "{}", m),
        }
    }
}

#[async_trait]
impl LockdownMode for LockdownModes {
    fn specificity(&self) -> usize {
        match self {
            LockdownModes::QuickServerLockdown(qsl) => qsl.specificity(),
            LockdownModes::TraditionalServerLockdown(tsl) => tsl.specificity(),
            LockdownModes::SingleChannelLockdown(scl) => scl.specificity(),
            LockdownModes::Unknown(m) => m.specificity(),
        }
    }

    async fn test(
        &self,
        lockdown_data: &LockdownData,
        pg: &serenity::all::PartialGuild,
        pgc: &[serenity::all::GuildChannel],
        critical_roles: &HashSet<serenity::all::RoleId>,
    ) -> Result<Box<dyn LockdownTestResult>, silverpelt::Error> {
        match self {
            LockdownModes::QuickServerLockdown(qsl) => {
                qsl.test(lockdown_data, pg, pgc, critical_roles).await
            }
            LockdownModes::TraditionalServerLockdown(tsl) => {
                tsl.test(lockdown_data, pg, pgc, critical_roles).await
            }
            LockdownModes::SingleChannelLockdown(scl) => {
                scl.test(lockdown_data, pg, pgc, critical_roles).await
            }
            LockdownModes::Unknown(m) => m.test(lockdown_data, pg, pgc, critical_roles).await,
        }
    }

    async fn setup(
        &self,
        lockdown_data: &LockdownData,
        pg: &serenity::all::PartialGuild,
        pgc: &[serenity::all::GuildChannel],
        critical_roles: &HashSet<serenity::all::RoleId>,
    ) -> Result<serde_json::Value, silverpelt::Error> {
        match self {
            LockdownModes::QuickServerLockdown(qsl) => {
                qsl.setup(lockdown_data, pg, pgc, critical_roles).await
            }
            LockdownModes::TraditionalServerLockdown(tsl) => {
                tsl.setup(lockdown_data, pg, pgc, critical_roles).await
            }
            LockdownModes::SingleChannelLockdown(scl) => {
                scl.setup(lockdown_data, pg, pgc, critical_roles).await
            }
            LockdownModes::Unknown(m) => m.setup(lockdown_data, pg, pgc, critical_roles).await,
        }
    }

    async fn create(
        &self,
        lockdown_data: &LockdownData,
        pg: &mut serenity::all::PartialGuild,
        pgc: &mut [serenity::all::GuildChannel],
        critical_roles: &HashSet<serenity::all::RoleId>,
        data: &serde_json::Value,
        all_handles: &LockdownModeHandles,
    ) -> Result<(), silverpelt::Error> {
        match self {
            LockdownModes::QuickServerLockdown(qsl) => {
                qsl.create(lockdown_data, pg, pgc, critical_roles, data, all_handles)
                    .await
            }
            LockdownModes::TraditionalServerLockdown(tsl) => {
                tsl.create(lockdown_data, pg, pgc, critical_roles, data, all_handles)
                    .await
            }
            LockdownModes::SingleChannelLockdown(scl) => {
                scl.create(lockdown_data, pg, pgc, critical_roles, data, all_handles)
                    .await
            }
            LockdownModes::Unknown(m) => {
                m.create(lockdown_data, pg, pgc, critical_roles, data, all_handles)
                    .await
            }
        }
    }

    async fn revert(
        &self,
        lockdown_data: &LockdownData,
        pg: &mut serenity::all::PartialGuild,
        pgc: &mut [serenity::all::GuildChannel],
        critical_roles: &HashSet<serenity::all::RoleId>,
        data: &serde_json::Value,
        all_handles: &LockdownModeHandles,
    ) -> Result<(), silverpelt::Error> {
        match self {
            LockdownModes::QuickServerLockdown(qsl) => {
                qsl.revert(lockdown_data, pg, pgc, critical_roles, data, all_handles)
                    .await
            }
            LockdownModes::TraditionalServerLockdown(tsl) => {
                tsl.revert(lockdown_data, pg, pgc, critical_roles, data, all_handles)
                    .await
            }
            LockdownModes::SingleChannelLockdown(scl) => {
                scl.revert(lockdown_data, pg, pgc, critical_roles, data, all_handles)
                    .await
            }
            LockdownModes::Unknown(m) => {
                m.revert(lockdown_data, pg, pgc, critical_roles, data, all_handles)
                    .await
            }
        }
    }

    async fn handles(
        &self,
        lockdown_data: &LockdownData,
        pg: &serenity::all::PartialGuild,
        pgc: &[serenity::all::GuildChannel],
        critical_roles: &HashSet<serenity::all::RoleId>,
        data: &serde_json::Value,
    ) -> Result<LockdownModeHandle, silverpelt::Error> {
        match self {
            LockdownModes::QuickServerLockdown(qsl) => {
                qsl.handles(lockdown_data, pg, pgc, critical_roles, data)
                    .await
            }
            LockdownModes::TraditionalServerLockdown(tsl) => {
                tsl.handles(lockdown_data, pg, pgc, critical_roles, data)
                    .await
            }
            LockdownModes::SingleChannelLockdown(scl) => {
                scl.handles(lockdown_data, pg, pgc, critical_roles, data)
                    .await
            }
            LockdownModes::Unknown(m) => {
                m.handles(lockdown_data, pg, pgc, critical_roles, data)
                    .await
            }
        }
    }
}

// Serializer for LockdownMode
impl serde::Serialize for LockdownModes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            LockdownModes::QuickServerLockdown(_) => serializer.serialize_str("qsl"),
            LockdownModes::TraditionalServerLockdown(_) => serializer.serialize_str("tsl"),
            LockdownModes::SingleChannelLockdown(scl) => {
                serializer.serialize_str(&format!("scl/{}", scl.0))
            }
            LockdownModes::Unknown(m) => serializer.serialize_str(&m.to_string()),
        }
    }
}

// Deserializer for LockdownMode
impl<'de> serde::Deserialize<'de> for LockdownModes {
    fn deserialize<D>(deserializer: D) -> Result<LockdownModes, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        match LockdownModes::from_string(&s) {
            Ok(Some(m)) => Ok(m),
            Ok(None) => Err(serde::de::Error::custom("Invalid lockdown mode")),
            Err(e) => Err(serde::de::Error::custom(e)),
        }
    }
}

/// Represents a lockdown
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Lockdown {
    pub id: sqlx::types::Uuid,
    pub reason: String,
    pub r#type: LockdownModes,
    pub data: serde_json::Value,
}

impl Lockdown {
    pub fn to_map(&self) -> indexmap::IndexMap<String, splashcore_rs::value::Value> {
        indexmap::indexmap! {
            "type".to_string() => splashcore_rs::value::Value::String(self.r#type.to_string()),
            "data".to_string() => splashcore_rs::value::Value::from_json(&self.data),
        }
    }
}

/// Represents a list of lockdowns
pub struct LockdownSet {
    pub lockdowns: Vec<Lockdown>,
    pub settings: Arc<super::cache::GuildLockdownSettings>,
    pub guild_id: serenity::all::GuildId,
}

impl LockdownSet {
    pub async fn guild(
        guild_id: serenity::all::GuildId,
        pool: &sqlx::PgPool,
    ) -> Result<Self, silverpelt::Error> {
        let data = sqlx::query!(
            "SELECT id, type, data, reason FROM lockdown__guild_lockdowns WHERE guild_id = $1",
            guild_id.to_string(),
        )
        .fetch_all(pool)
        .await?;

        let mut lockdowns = Vec::new();

        for row in data {
            let id = row.id;
            let r#type = row.r#type;
            let data = row.data;
            let reason = row.reason;

            let lockdown = match LockdownModes::from_string(&r#type) {
                Ok(Some(m)) => Lockdown {
                    id,
                    r#type: m,
                    data,
                    reason,
                },
                Ok(None) => continue,
                Err(e) => {
                    return Err(silverpelt::Error::from(format!(
                        "Error while parsing lockdown type: {}",
                        e
                    )))
                }
            };

            lockdowns.push(lockdown);
        }

        let settings = super::cache::get_guild_lockdown_settings(pool, guild_id).await?;

        Ok(LockdownSet {
            lockdowns,
            settings,
            guild_id,
        })
    }

    /// Sorts the lockdowns by specificity in descending order
    pub fn sort(&mut self) {
        self.lockdowns
            .sort_by(|a, b| b.r#type.specificity().cmp(&a.r#type.specificity()));
    }

    pub async fn get_handles(
        &self,
        lockdown_data: &LockdownData,
        pg: &serenity::all::PartialGuild,
        pgc: &[serenity::all::GuildChannel],
    ) -> Result<LockdownModeHandles, silverpelt::Error> {
        let mut handles = LockdownModeHandles {
            roles: PrioritySet::default(),
            channels: PrioritySet::default(),
        };

        for lockdown in self.lockdowns.iter() {
            let handle = lockdown
                .r#type
                .handles(
                    lockdown_data,
                    pg,
                    pgc,
                    &self.settings.member_roles,
                    &lockdown.data,
                )
                .await?;

            // Extend roles and channels
            handles.add_handle(handle, lockdown.r#type.specificity());
        }

        Ok(handles)
    }

    /// Adds a lockdown to the set returning the id of the created entry
    pub async fn apply(
        &mut self,
        lockdown_type: LockdownModes,
        lockdown_data: &LockdownData,
        reason: &str,
    ) -> Result<sqlx::types::Uuid, silverpelt::Error> {
        self.sort();

        // Fetch guild+channel info to advance to avoid needing to fetch it on every interaction with the trait
        let mut pg = proxy_support::guild(
            &lockdown_data.cache_http,
            &lockdown_data.reqwest,
            self.guild_id,
        )
        .await?;

        let mut pgc = proxy_support::guild_channels(
            &lockdown_data.cache_http,
            &lockdown_data.reqwest,
            self.guild_id,
        )
        .await?;

        let critical_roles = get_critical_roles(&pg, &self.settings.member_roles)?;

        // Test new lockdown if required
        if self.settings.require_correct_layout {
            let test_results = lockdown_type
                .test(lockdown_data, &pg, &pgc, &critical_roles)
                .await?;

            if !test_results.can_apply_perfectly() {
                return Err(test_results.display_result(&pg).into());
            }
        }

        // Setup the lockdown
        let data = lockdown_type
            .setup(lockdown_data, &pg, &pgc, &critical_roles)
            .await?;

        let current_handles = self.get_handles(lockdown_data, &pg, &pgc).await?;

        // Get the handles for the new lockdown
        /*let new_handle = lockdown_type
            .handles(lockdown_data, &pg, &pgc, &critical_roles, &data)
            .await?;

        if current_handles.is_redundant(&new_handle, lockdown_type.specificity()) {
            return Err("Lockdown is redundant (all changes made by this lockdown handle are already locked by another handle)".into());
        }*/

        let id = sqlx::query!(
            "INSERT INTO lockdown__guild_lockdowns (guild_id, type, data, reason) VALUES ($1, $2, $3, $4) RETURNING id",
            self.guild_id.to_string(),
            lockdown_type.to_string(),
            &data,
            reason,
        )
        .fetch_one(&lockdown_data.pool)
        .await?;

        // Apply the lockdown
        lockdown_type
            .create(
                lockdown_data,
                &mut pg,
                &mut pgc,
                &critical_roles,
                &data,
                &current_handles,
            )
            .await?;

        // Update self.lockdowns
        self.lockdowns.push(Lockdown {
            id: id.id,
            r#type: lockdown_type,
            data,
            reason: reason.to_string(),
        });

        Ok(id.id)
    }

    /// Removes a lockdown from the set by index
    pub async fn remove(
        &mut self,
        index: usize,
        lockdown_data: &LockdownData,
    ) -> Result<(), silverpelt::Error> {
        self.sort();

        let lockdown = self.lockdowns.get(index).ok_or_else(|| {
            silverpelt::Error::from("Lockdown index out of bounds (does not exist)")
        })?;

        // Fetch guild+channel info to advance to avoid needing to fetch it on every interaction with the trait
        let mut pg = proxy_support::guild(
            &lockdown_data.cache_http,
            &lockdown_data.reqwest,
            self.guild_id,
        )
        .await?;

        let mut pgc = proxy_support::guild_channels(
            &lockdown_data.cache_http,
            &lockdown_data.reqwest,
            self.guild_id,
        )
        .await?;

        let critical_roles = get_critical_roles(&pg, &self.settings.member_roles)?;

        let mut current_handles = self.get_handles(lockdown_data, &pg, &pgc).await?;

        // Remove handle from the set
        let handle = lockdown
            .r#type
            .handles(lockdown_data, &pg, &pgc, &critical_roles, &lockdown.data)
            .await?;

        current_handles.remove_handle(&handle, lockdown.r#type.specificity());

        // Revert the lockdown
        lockdown
            .r#type
            .revert(
                lockdown_data,
                &mut pg,
                &mut pgc,
                &critical_roles,
                &lockdown.data,
                &current_handles,
            )
            .await?;

        // Remove the lockdown from the database
        sqlx::query!(
            "DELETE FROM lockdown__guild_lockdowns WHERE guild_id = $1 AND type = $2",
            self.guild_id.to_string(),
            lockdown.r#type.to_string(),
        )
        .execute(&lockdown_data.pool)
        .await?;

        // Update self.lockdowns
        self.lockdowns.remove(index);

        Ok(())
    }

    /// Remove all lockdowns in order of specificity
    pub async fn remove_all(
        &mut self,
        lockdown_data: &LockdownData,
    ) -> Result<(), silverpelt::Error> {
        self.sort();

        // Fetch guild+channel info to advance to avoid needing to fetch it on every interaction with the trait
        let mut pg = proxy_support::guild(
            &lockdown_data.cache_http,
            &lockdown_data.reqwest,
            self.guild_id,
        )
        .await?;

        let mut pgc = proxy_support::guild_channels(
            &lockdown_data.cache_http,
            &lockdown_data.reqwest,
            self.guild_id,
        )
        .await?;

        let critical_roles = get_critical_roles(&pg, &self.settings.member_roles)?;

        let mut current_handles = self.get_handles(lockdown_data, &pg, &pgc).await?;

        for lockdown in self.lockdowns.iter() {
            // Revert the lockdown
            lockdown
                .r#type
                .revert(
                    lockdown_data,
                    &mut pg,
                    &mut pgc,
                    &critical_roles,
                    &lockdown.data,
                    &current_handles,
                )
                .await?;

            // Remove the lockdown from the database
            sqlx::query!(
                "DELETE FROM lockdown__guild_lockdowns WHERE guild_id = $1 AND type = $2",
                self.guild_id.to_string(),
                lockdown.r#type.to_string(),
            )
            .execute(&lockdown_data.pool)
            .await?;

            // We need to re-fetch the handles after each lockdown is removed
            current_handles = self.get_handles(lockdown_data, &pg, &pgc).await?;
        }

        // Update self.lockdowns
        self.lockdowns.clear();

        Ok(())
    }
}

/// Quick server lockdown
pub mod qsl {
    use super::*;
    use serde::{Deserialize, Serialize};

    /// The base permissions for quick lockdown
    ///
    /// If any of these permissions are provided, quick lockdown cannot proceed
    static BASE_PERMS: [serenity::all::Permissions; 2] = [
        serenity::all::Permissions::VIEW_CHANNEL,
        serenity::all::Permissions::SEND_MESSAGES,
    ];

    static LOCKDOWN_PERMS: std::sync::LazyLock<serenity::all::Permissions> =
        std::sync::LazyLock::new(|| serenity::all::Permissions::VIEW_CHANNEL);

    /// The result of a `test_quick_lockdown` call
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct QuickLockdownTestResult {
        /// Which roles need to be changed/fixed combined with the target perms
        pub changes_needed: std::collections::HashMap<
            serenity::all::RoleId,
            (ChangeOp, serenity::all::Permissions),
        >,
        /// The critical roles (either member roles or the `@everyone` role)
        pub critical_roles: HashSet<serenity::all::RoleId>,
    }

    impl LockdownTestResult for QuickLockdownTestResult {
        /// Returns whether the guild is in a state where quick lockdown can be applied perfectly
        fn can_apply_perfectly(&self) -> bool {
            self.changes_needed.is_empty()
        }

        fn display_result(&self, pg: &serenity::all::PartialGuild) -> String {
            let mut needed_changes = String::new();

            needed_changes.push_str("The following roles need to be changed:\n");
            for (role_id, perms) in self.changes_needed.iter() {
                let role_name = pg
                    .roles
                    .get(role_id)
                    .map(|r| r.name.to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                needed_changes.push_str(&format!("Role: {} ({})\n", role_name, role_id));
                needed_changes.push_str(&format!("Permissions: {} {}\n", perms.0, perms.1));
                needed_changes.push('\n');
            }

            needed_changes
        }
    }

    pub struct QuickServerLockdown;

    impl QuickServerLockdown {
        pub fn from_data(
            data: &serde_json::Value,
        ) -> Result<
            std::collections::HashMap<serenity::all::RoleId, serenity::all::Permissions>,
            silverpelt::Error,
        > {
            let v: std::collections::HashMap<serenity::all::RoleId, serenity::all::Permissions> =
                serde_json::from_value(data.clone())
                    .map_err(|e| format!("Error while deserializing permissions: {}", e))?;

            Ok(v)
        }
    }

    #[async_trait]
    impl LockdownMode for QuickServerLockdown {
        // Lowest specificity
        fn specificity(&self) -> usize {
            0
        }

        async fn test(
            &self,
            _lockdown_data: &LockdownData,
            pg: &serenity::all::PartialGuild,
            _pgc: &[serenity::all::GuildChannel],
            critical_roles: &HashSet<serenity::all::RoleId>,
        ) -> Result<Box<dyn LockdownTestResult>, silverpelt::Error> {
            let mut changes_needed = std::collections::HashMap::new();

            // From here on out, we only need to care about critical and non critical roles
            for role in pg.roles.iter() {
                if critical_roles.contains(&role.id) {
                    let mut needed_perms = serenity::all::Permissions::empty();

                    let mut missing = false;
                    for perm in BASE_PERMS {
                        if !role.permissions.contains(perm) {
                            needed_perms |= perm;
                            missing = true;
                        }
                    }

                    if missing {
                        changes_needed.insert(role.id, (ChangeOp::Add, needed_perms));
                    }
                } else {
                    let mut perms_to_remove = serenity::all::Permissions::empty();

                    let mut needs_perms_removed = false;
                    for perm in BASE_PERMS {
                        if role.permissions.contains(perm) {
                            perms_to_remove |= perm;
                            needs_perms_removed = true;
                        }
                    }

                    if needs_perms_removed {
                        changes_needed.insert(role.id, (ChangeOp::Remove, perms_to_remove));
                    }
                }
            }

            Ok(Box::new(QuickLockdownTestResult {
                changes_needed,
                critical_roles: critical_roles.clone(),
            }))
        }

        async fn setup(
            &self,
            _lockdown_data: &LockdownData,
            pg: &serenity::all::PartialGuild,
            _pgc: &[serenity::all::GuildChannel],
            _critical_roles: &HashSet<serenity::all::RoleId>,
        ) -> Result<serde_json::Value, silverpelt::Error> {
            let mut map = serde_json::Map::new();

            for role in pg.roles.iter() {
                map.insert(
                    role.id.to_string(),
                    serde_json::Value::String(role.permissions.bits().to_string()),
                );
            }

            Ok(serde_json::Value::Object(map))
        }

        async fn create(
            &self,
            lockdown_data: &LockdownData,
            pg: &mut serenity::all::PartialGuild,
            _pgc: &mut [serenity::all::GuildChannel],
            critical_roles: &HashSet<serenity::all::RoleId>,
            _data: &serde_json::Value,
            _all_handles: &LockdownModeHandles,
        ) -> Result<(), silverpelt::Error> {
            let mut new_roles = Vec::new();
            for role in pg.roles.iter() {
                // If critical, lock it down
                if critical_roles.contains(&role.id) {
                    new_roles.push(
                        pg.id
                            .edit_role(
                                &lockdown_data.cache_http.http,
                                role.id,
                                serenity::all::EditRole::new().permissions(*LOCKDOWN_PERMS),
                            )
                            .await?,
                    );
                }
            }

            for role in new_roles {
                pg.roles.insert(role);
            }

            Ok(())
        }

        async fn revert(
            &self,
            lockdown_data: &LockdownData,
            pg: &mut serenity::all::PartialGuild,
            _pgc: &mut [serenity::all::GuildChannel],
            critical_roles: &HashSet<serenity::all::RoleId>,
            data: &serde_json::Value,
            _all_handles: &LockdownModeHandles,
        ) -> Result<(), silverpelt::Error> {
            let old_permissions = Self::from_data(data)?;

            let mut new_roles = Vec::new();
            for role in pg.roles.iter() {
                if critical_roles.contains(&role.id) {
                    let perms = old_permissions.get(&role.id).copied().unwrap_or(
                        BASE_PERMS
                            .iter()
                            .copied()
                            .fold(serenity::all::Permissions::empty(), |acc, perm| acc | perm),
                    );

                    new_roles.push(
                        pg.id
                            .edit_role(
                                &lockdown_data.cache_http.http,
                                role.id,
                                serenity::all::EditRole::new().permissions(perms),
                            )
                            .await?,
                    );
                }
            }

            for role in new_roles {
                pg.roles.insert(role);
            }

            Ok(())
        }

        async fn handles(
            &self,
            _lockdown_data: &LockdownData,
            _pg: &serenity::all::PartialGuild,
            _pgc: &[serenity::all::GuildChannel],
            critical_roles: &HashSet<serenity::all::RoleId>,
            _data: &serde_json::Value,
        ) -> Result<LockdownModeHandle, silverpelt::Error> {
            // QSL locks the critical roles
            Ok(LockdownModeHandle {
                roles: critical_roles.clone(),
                channels: HashSet::new(),
            })
        }
    }
}

/// Traditional server lockdown (lock all channels)
pub mod tsl {
    use super::*;
    use serde::{Deserialize, Serialize};

    static DENY_PERMS: std::sync::LazyLock<serenity::all::Permissions> =
        std::sync::LazyLock::new(|| {
            serenity::all::Permissions::SEND_MESSAGES
                | serenity::all::Permissions::SEND_MESSAGES_IN_THREADS
                | serenity::all::Permissions::SEND_TTS_MESSAGES
                | serenity::all::Permissions::CONNECT
        });

    // The big advantage of TSL is the lack of constraints/tests regarding server layout
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct TraditionalLockdownTestResult;

    impl LockdownTestResult for TraditionalLockdownTestResult {
        fn can_apply_perfectly(&self) -> bool {
            log::info!("Called can_apply_perfectly");
            true
        }

        fn display_result(&self, _pg: &serenity::all::PartialGuild) -> String {
            "".to_string()
        }
    }
    pub struct TraditionalServerLockdown;

    impl TraditionalServerLockdown {
        pub fn from_data(
            data: &serde_json::Value,
        ) -> Result<
            std::collections::HashMap<
                serenity::all::ChannelId,
                Vec<serenity::all::PermissionOverwrite>,
            >,
            silverpelt::Error,
        > {
            log::info!("Called from_data");
            let v: std::collections::HashMap<
                serenity::all::ChannelId,
                Vec<serenity::all::PermissionOverwrite>,
            > = serde_json::from_value(data.clone())
                .map_err(|e| format!("Error while deserializing permissions: {}", e))?;

            Ok(v)
        }
    }

    #[async_trait]
    impl LockdownMode for TraditionalServerLockdown {
        // TSL > QSL as it updates all channels in a server
        fn specificity(&self) -> usize {
            1
        }

        // TSL doesn't need to test anything so just return the result
        async fn test(
            &self,
            _lockdown_data: &LockdownData,
            _pg: &serenity::all::PartialGuild,
            _pgc: &[serenity::all::GuildChannel],
            _critical_roles: &HashSet<serenity::all::RoleId>,
        ) -> Result<Box<dyn LockdownTestResult>, silverpelt::Error> {
            log::info!("Called test");
            Ok(Box::new(TraditionalLockdownTestResult))
        }

        async fn setup(
            &self,
            _lockdown_data: &LockdownData,
            _pg: &serenity::all::PartialGuild,
            pgc: &[serenity::all::GuildChannel],
            _critical_roles: &HashSet<serenity::all::RoleId>,
        ) -> Result<serde_json::Value, silverpelt::Error> {
            log::info!("Called setup");
            let mut map = serde_json::Map::new();

            for channel in pgc.iter() {
                map.insert(
                    channel.id.to_string(),
                    serde_json::to_value(channel.permission_overwrites.clone())?,
                );
            }

            Ok(serde_json::Value::Object(map))
        }

        async fn create(
            &self,
            lockdown_data: &LockdownData,
            _pg: &mut serenity::all::PartialGuild,
            pgc: &mut [serenity::all::GuildChannel],
            critical_roles: &HashSet<serenity::all::RoleId>,
            _data: &serde_json::Value,
            all_handles: &LockdownModeHandles,
        ) -> Result<(), silverpelt::Error> {
            log::info!("Called create");
            for channel in pgc.iter_mut() {
                if all_handles
                    .is_channel_locked(channel.id, self.specificity())
                    .is_some()
                {
                    continue; // Someone else is handling this channel
                }

                let mut overwrites = channel.permission_overwrites.to_vec();

                let mut nyset_overwrite = critical_roles.clone();
                for overwrite in overwrites.iter_mut() {
                    match overwrite.kind {
                        serenity::all::PermissionOverwriteType::Role(role_id) => {
                            if critical_roles.contains(&role_id) {
                                overwrite.deny = *DENY_PERMS;
                                nyset_overwrite.remove(&role_id);
                            }
                        }
                        _ => continue,
                    }
                }

                if !nyset_overwrite.is_empty() {
                    for critical_role in nyset_overwrite.iter() {
                        let overwrite = serenity::all::PermissionOverwrite {
                            allow: serenity::all::Permissions::empty(),
                            deny: *DENY_PERMS,
                            kind: serenity::all::PermissionOverwriteType::Role(*critical_role),
                        };

                        overwrites.push(overwrite);
                    }
                }

                match channel
                    .edit(
                        &lockdown_data.cache_http.http,
                        serenity::all::EditChannel::new().permissions(overwrites),
                    )
                    .await
                {
                    Ok(_) => {}
                    Err(e) => match e {
                        serenity::Error::Http(e) => match e {
                            serenity::all::HttpError::UnsuccessfulRequest(er) => {
                                if er.status_code == reqwest::StatusCode::NOT_FOUND {
                                    log::info!("Channel not found: {}", channel.id);
                                    continue; // Rare, but sometimes happens (?)
                                } else {
                                    return Err(format!(
                                        "Failed to create channel lockdown (http, non-404) {}: {:?}",
                                        channel.id, er
                                    )
                                    .into());
                                }
                            }
                            _ => {
                                return Err(format!(
                                    "Failed to create channel lockdown (http) {}: {:?}",
                                    channel.id, e
                                )
                                .into());
                            }
                        },
                        _ => {
                            return Err(format!(
                                "Failed to create channel lockdown {}: {:?}",
                                channel.id, e
                            )
                            .into());
                        }
                    },
                };
            }

            Ok(())
        }

        async fn revert(
            &self,
            lockdown_data: &LockdownData,
            _pg: &mut serenity::all::PartialGuild,
            pgc: &mut [serenity::all::GuildChannel],
            _critical_roles: &HashSet<serenity::all::RoleId>,
            data: &serde_json::Value,
            all_handles: &LockdownModeHandles,
        ) -> Result<(), silverpelt::Error> {
            log::info!("Called can_apply_perfectly");

            let old_permissions = Self::from_data(data)?;

            for channel in pgc.iter_mut() {
                if all_handles
                    .is_channel_locked(channel.id, self.specificity())
                    .is_some()
                {
                    continue; // Someone else is handling this channel
                }

                // TODO: Handle this slightly better (maybe only apply the changes to critical roles somehow)
                let Some(overwrites) = old_permissions.get(&channel.id).cloned() else {
                    continue;
                };

                match channel
                    .edit(
                        &lockdown_data.cache_http.http,
                        serenity::all::EditChannel::new().permissions(overwrites),
                    )
                    .await
                {
                    Ok(_) => {}
                    Err(e) => match e {
                        serenity::Error::Http(e) => match e {
                            serenity::all::HttpError::UnsuccessfulRequest(er) => {
                                if er.status_code == reqwest::StatusCode::NOT_FOUND {
                                    continue; // Rare, but sometimes happens (?)
                                } else {
                                    return Err(format!(
                                        "Failed to delete channel lockdown (http, non-404) {}: {:?}",
                                        channel.id, er
                                    )
                                    .into());
                                }
                            }
                            _ => {
                                return Err(format!(
                                    "Failed to delete channel lockdown (http) {}: {:?}",
                                    channel.id, e
                                )
                                .into());
                            }
                        },
                        _ => {
                            return Err(format!(
                                "Failed to delete channel lockdown {}: {:?}",
                                channel.id, e
                            )
                            .into());
                        }
                    },
                };
            }

            Ok(())
        }

        async fn handles(
            &self,
            _lockdown_data: &LockdownData,
            _pg: &serenity::all::PartialGuild,
            pgc: &[serenity::all::GuildChannel],
            _critical_roles: &HashSet<serenity::all::RoleId>,
            _data: &serde_json::Value,
        ) -> Result<LockdownModeHandle, silverpelt::Error> {
            // TSL locks all channels, but *NOT* roles
            Ok(LockdownModeHandle {
                roles: HashSet::new(),
                channels: pgc.iter().map(|c| c.id).collect(),
            })
        }
    }
}

/// Single channel lock
pub mod scl {
    use super::*;
    use serde::{Deserialize, Serialize};

    static DENY_PERMS: std::sync::LazyLock<serenity::all::Permissions> =
        std::sync::LazyLock::new(|| {
            serenity::all::Permissions::SEND_MESSAGES
                | serenity::all::Permissions::SEND_MESSAGES_IN_THREADS
                | serenity::all::Permissions::SEND_TTS_MESSAGES
                | serenity::all::Permissions::CONNECT
        });

    // The big advantage of TSL is the lack of constraints/tests regarding server layout
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct SingleChannelLockdownTestResult;

    impl LockdownTestResult for SingleChannelLockdownTestResult {
        fn can_apply_perfectly(&self) -> bool {
            true
        }

        fn display_result(&self, _pg: &serenity::all::PartialGuild) -> String {
            "".to_string()
        }
    }
    pub struct SingleChannelLockdown(pub serenity::all::ChannelId);

    impl SingleChannelLockdown {
        pub fn from_data(
            data: &serde_json::Value,
        ) -> Result<Vec<serenity::all::PermissionOverwrite>, silverpelt::Error> {
            let v: Vec<serenity::all::PermissionOverwrite> =
                serde_json::from_value(data.clone())
                    .map_err(|e| format!("Error while deserializing permissions: {}", e))?;

            Ok(v)
        }
    }

    #[async_trait]
    impl LockdownMode for SingleChannelLockdown {
        // SCL > TSL as it updates a single channel
        fn specificity(&self) -> usize {
            2
        }

        // SCL doesn't need to test anything so just return the result
        async fn test(
            &self,
            _lockdown_data: &LockdownData,
            _pg: &serenity::all::PartialGuild,
            _pgc: &[serenity::all::GuildChannel],
            _critical_roles: &HashSet<serenity::all::RoleId>,
        ) -> Result<Box<dyn LockdownTestResult>, silverpelt::Error> {
            Ok(Box::new(SingleChannelLockdownTestResult))
        }

        async fn setup(
            &self,
            _lockdown_data: &LockdownData,
            _pg: &serenity::all::PartialGuild,
            pgc: &[serenity::all::GuildChannel],
            _critical_roles: &HashSet<serenity::all::RoleId>,
        ) -> Result<serde_json::Value, silverpelt::Error> {
            let channel = pgc
                .iter()
                .find(|c| c.id == self.0)
                .ok_or_else(|| silverpelt::Error::from("Channel not found"))?;

            Ok(serde_json::to_value(channel.permission_overwrites.clone())?)
        }

        async fn create(
            &self,
            lockdown_data: &LockdownData,
            _pg: &mut serenity::all::PartialGuild,
            _pgc: &mut [serenity::all::GuildChannel],
            critical_roles: &HashSet<serenity::all::RoleId>,
            data: &serde_json::Value,
            all_handles: &LockdownModeHandles,
        ) -> Result<(), silverpelt::Error> {
            if all_handles
                .is_channel_locked(self.0, self.specificity())
                .is_some()
            {
                return Ok(()); // Someone else is handling this channel
            }

            let mut overwrites = Self::from_data(data)?;

            let mut nyset_overwrite = critical_roles.clone();
            for overwrite in overwrites.iter_mut() {
                match overwrite.kind {
                    serenity::all::PermissionOverwriteType::Role(role_id) => {
                        if critical_roles.contains(&role_id) {
                            overwrite.deny = *DENY_PERMS;
                            nyset_overwrite.remove(&role_id);
                        }
                    }
                    _ => continue,
                }
            }

            if !nyset_overwrite.is_empty() {
                for critical_role in nyset_overwrite.iter() {
                    let overwrite = serenity::all::PermissionOverwrite {
                        allow: serenity::all::Permissions::empty(),
                        deny: *DENY_PERMS,
                        kind: serenity::all::PermissionOverwriteType::Role(*critical_role),
                    };

                    overwrites.push(overwrite);
                }
            }

            self.0
                .edit(
                    &lockdown_data.cache_http.http,
                    serenity::all::EditChannel::new().permissions(overwrites),
                )
                .await?;

            Ok(())
        }

        async fn revert(
            &self,
            lockdown_data: &LockdownData,
            _pg: &mut serenity::all::PartialGuild,
            _pgc: &mut [serenity::all::GuildChannel],
            _critical_roles: &HashSet<serenity::all::RoleId>,
            data: &serde_json::Value,
            all_handles: &LockdownModeHandles,
        ) -> Result<(), silverpelt::Error> {
            if all_handles
                .is_channel_locked(self.0, self.specificity())
                .is_some()
            {
                return Ok(()); // Someone else is handling this channel
            }

            let overwrites = Self::from_data(data)?;

            self.0
                .edit(
                    &lockdown_data.cache_http.http,
                    serenity::all::EditChannel::new().permissions(overwrites),
                )
                .await?;

            Ok(())
        }

        async fn handles(
            &self,
            _lockdown_data: &LockdownData,
            _pg: &serenity::all::PartialGuild,
            _pgc: &[serenity::all::GuildChannel],
            _critical_roles: &HashSet<serenity::all::RoleId>,
            _data: &serde_json::Value,
        ) -> Result<LockdownModeHandle, silverpelt::Error> {
            // SCL locks a single channel
            Ok(LockdownModeHandle {
                roles: HashSet::new(),
                channels: std::iter::once(self.0).collect(),
            })
        }
    }
}

use async_trait::async_trait;
use bitflags::bitflags;
use module_settings::types::{OperationType, SettingsError};
use serde::{Deserialize, Serialize};
use serenity::all::{GuildId, UserId};
use std::str::FromStr;
use std::sync::Arc;

bitflags! {
    /// Flags for the sting source
    pub struct StingSourceFlags: u32 {
        /// Whether or not the sting source supports *manually* voiding entries
        const SUPPORTS_MANUAL_VOIDING = 1 << 0;
        /// Supports durations. Not all producers support these
        const SUPPORTS_DURATIONS = 1 << 1;
        /// Supports (revertable) actions (remove_all_roles etc). Not all producers support these
        ///
        /// Required for any sort of punishment reversal system (such as the standard temp_punishment module)
        const SUPPORTS_ACTIONS = 1 << 2;
        /// Supports creating sting entries
        const SUPPORTS_CREATE = 1 << 3;
        /// Supports updating sting entries
        const SUPPORTS_UPDATE = 1 << 4;
        /// Supports deleting sting entries
        const SUPPORTS_DELETE = 1 << 5;
        /// Requires a guild id to be provided under filters to work. Some producers may require this
        const REQUIRES_GUILD_ID_IN_FILTER = 1 << 6;
    }
}

impl StingSourceFlags {
    // Utility functions

    pub fn supports_manually_voiding(&self) -> bool {
        self.contains(StingSourceFlags::SUPPORTS_MANUAL_VOIDING)
    }

    pub fn supports_actions(&self) -> bool {
        self.contains(StingSourceFlags::SUPPORTS_ACTIONS)
    }

    pub fn can_create(&self) -> bool {
        self.contains(StingSourceFlags::SUPPORTS_CREATE)
    }

    pub fn can_update(&self) -> bool {
        self.contains(StingSourceFlags::SUPPORTS_UPDATE)
    }

    pub fn can_delete(&self) -> bool {
        self.contains(StingSourceFlags::SUPPORTS_DELETE)
    }

    pub fn is_readonly(&self) -> bool {
        !self.can_create() && !self.can_update() && !self.can_delete()
    }

    pub fn supports_duration(&self) -> bool {
        self.contains(StingSourceFlags::SUPPORTS_DURATIONS)
    }
}

#[derive(Hash, Default, Debug, Clone, Copy, PartialEq)]
pub enum StingState {
    #[default]
    Active,
    Voided,
    Handled,
}

impl std::fmt::Display for StingState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StingState::Active => write!(f, "active"),
            StingState::Voided => write!(f, "voided"),
            StingState::Handled => write!(f, "handled"),
        }
    }
}

impl std::str::FromStr for StingState {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(StingState::Active),
            "voided" => Ok(StingState::Voided),
            "handled" => Ok(StingState::Handled),
            _ => Err(format!("Invalid sting state: {}", s).into()),
        }
    }
}

// Serde impls for StingState
impl Serialize for StingState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

impl<'de> Deserialize<'de> for StingState {
    fn deserialize<D>(deserializer: D) -> Result<StingState, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        StingState::from_str(&s).map_err(serde::de::Error::custom)
    }
}

/// What created the sting
#[derive(Debug, Clone)]
pub enum StingCreator {
    /// The sting was created by a user
    User(UserId),
    /// The sting was created by the system
    System,
}

// Serde impls for StingCreator
impl Serialize for StingCreator {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            StingCreator::User(user_id) => serializer.serialize_str(&format!("user:{}", user_id)),
            StingCreator::System => serializer.serialize_str("system"),
        }
    }
}

impl<'de> Deserialize<'de> for StingCreator {
    fn deserialize<D>(deserializer: D) -> Result<StingCreator, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s == "system" {
            Ok(StingCreator::System)
        } else {
            let user_id = s
                .strip_prefix("user:")
                .ok_or_else(|| serde::de::Error::custom("Invalid sting creator"))?;
            Ok(StingCreator::User(
                user_id.parse().map_err(serde::de::Error::custom)?,
            ))
        }
    }
}

/// An action taken due to this sting
#[derive(Debug, Clone)]
pub enum Action {
    None,
    Ban,
    Timeout,
    RemoveAllRoles,
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::None => write!(f, "none"),
            Action::Ban => write!(f, "ban"),
            Action::Timeout => write!(f, "timeout"),
            Action::RemoveAllRoles => write!(f, "remove_all_roles"),
        }
    }
}

impl std::str::FromStr for Action {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(Action::None),
            "ban" => Ok(Action::Ban),
            "timeout" => Ok(Action::Timeout),
            "remove_all_roles" => Ok(Action::RemoveAllRoles),
            _ => Err(format!("Invalid action: {}", s).into()),
        }
    }
}

// Serde impls for Action
impl Serialize for Action {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            Action::None => serializer.serialize_str("none"),
            Action::Ban => serializer.serialize_str("ban"),
            Action::Timeout => serializer.serialize_str("timeout"),
            Action::RemoveAllRoles => serializer.serialize_str("remove_all_roles"),
        }
    }
}

impl<'de> Deserialize<'de> for Action {
    fn deserialize<D>(deserializer: D) -> Result<Action, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "none" => Ok(Action::None),
            "ban" => Ok(Action::Ban),
            "remove_all_roles" => Ok(Action::RemoveAllRoles),
            _ => Err(serde::de::Error::custom("Invalid action")),
        }
    }
}

/// This struct contains data about a sting
///
/// As multiple modules may use and store stings in their own way,
/// StingEntry is a common abstraction for the punishment module
/// to store sting data and reason for presentation to users etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct FullStingEntry {
    /// The unique ID of the sting entry
    pub id: String,
    /// The sting entry
    pub entry: StingEntry,
    /// When the sting entry was created
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl FullStingEntry {
    /// Returns whether or not the sting entry is expired or not
    pub fn is_expired(&self) -> bool {
        if matches!(self.entry.state, StingState::Voided | StingState::Handled) {
            return true; // Voided/handled entries are always expired
        }

        if let Some(duration) = self.entry.duration {
            self.created_at + duration < chrono::Utc::now()
        } else {
            false
        }
    }
}

/// This struct contains data about a sting
///
/// As multiple modules may use and store stings in their own way,
/// StingEntry is a common abstraction for the punishment module
/// to store string data and reason for presentation to users etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct StingEntry {
    /// The user ID of the user who was stung
    pub user_id: UserId,
    /// The guild ID of the guild where the sting occurred
    pub guild_id: GuildId,
    /// What created the sting
    pub creator: StingCreator,
    /// The number of stings the user has in this entry
    pub stings: i32,
    /// The reason for the stings
    pub reason: Option<String>,
    /// Duration of the punishment
    pub duration: Option<std::time::Duration>,
    /// What action was taken
    pub action: Action,
    /// The current state of the sting
    pub state: StingState,
    /// The reason the sting was voided
    pub void_reason: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct UpdateStingEntry {
    pub stings: Option<i32>,
    pub reason: Option<String>,
    pub duration: Option<std::time::Duration>,
    pub action: Option<Action>,
    pub state: Option<StingState>,
    pub void_reason: Option<String>,
}

#[derive(Hash, Debug, Default, Clone)]
pub struct StingCountFilters {
    pub user_id: Option<UserId>,
    pub guild_id: Option<GuildId>,
    pub state: Option<StingState>,
    pub expired: Option<bool>,
    pub has_duration: Option<bool>, // Is 'ephemeral' AKA has a duration
}

impl StingCountFilters {
    pub fn from_map(
        src: &indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<Self, crate::Error> {
        let mut filters = StingCountFilters::default();

        for (k, v) in src.iter() {
            match k.as_str() {
                "user_id" => {
                    let uid_str = v.as_string().ok_or("Invalid user_id")?;
                    filters.user_id = Some(uid_str.parse()?);
                }
                "guild_id" => {
                    let gid_str = v.as_string().ok_or("Invalid guild_id")?;
                    filters.guild_id = Some(gid_str.parse()?);
                }
                "state" => {
                    filters.state =
                        Some(StingState::from_str(v.as_string().ok_or("Invalid state")?)?);
                }
                "expired" => {
                    filters.expired = Some(v.as_bool().ok_or("Invalid expired")?);
                }
                "has_duration" => {
                    filters.has_duration = Some(v.as_bool().ok_or("Invalid has_duration")?);
                }
                _ => continue,
            }
        }

        Ok(filters)
    }
}

#[derive(Hash, Debug, Default, Clone)]
pub struct StingFetchFilters {
    pub user_id: Option<UserId>,
    pub guild_id: Option<GuildId>,
    pub state: Option<StingState>,
    pub expired: Option<bool>,
    pub has_duration: Option<bool>, // Is 'ephemeral' AKA has a duration
}

impl StingFetchFilters {
    pub fn from_map(
        src: &indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<Self, crate::Error> {
        let mut filters = StingFetchFilters::default();

        for (k, v) in src.iter() {
            match k.as_str() {
                "user_id" => {
                    let uid_str = v.as_string().ok_or("Invalid user_id")?;
                    filters.user_id = Some(uid_str.parse()?);
                }
                "guild_id" => {
                    let gid_str = v.as_string().ok_or("Invalid guild_id")?;
                    filters.guild_id = Some(gid_str.parse()?);
                }
                "state" => {
                    filters.state =
                        Some(StingState::from_str(v.as_string().ok_or("Invalid state")?)?);
                }
                "expired" => {
                    filters.expired = Some(v.as_bool().ok_or("Invalid expired")?);
                }
                "has_duration" => {
                    filters.has_duration = Some(v.as_bool().ok_or("Invalid has_duration")?);
                }
                _ => continue,
            }
        }

        Ok(filters)
    }
}

impl StingFetchFilters {
    /// In some cases, the filters cannot be applied in any highly optimized way
    ///
    /// ``client_side_apply_filters`` applies the filters to a Vec<StringEntry> on the source side
    pub fn client_side_apply_filters(&self, entries: Vec<FullStingEntry>) -> Vec<FullStingEntry> {
        entries
            .into_iter()
            .filter(|entry| {
                // User ID filter
                if let Some(user_id) = self.user_id {
                    if entry.entry.user_id != user_id {
                        return false;
                    }
                }

                // Guild ID filter
                if let Some(guild_id) = self.guild_id {
                    if entry.entry.guild_id != guild_id {
                        return false;
                    }
                }

                // State filter
                if let Some(state) = self.state {
                    if entry.entry.state != state {
                        return false;
                    }
                }

                // Expired filter
                if let Some(expired) = self.expired {
                    if entry.is_expired() != expired {
                        return false;
                    }
                }

                // Has duration filter
                if let Some(has_duration) = self.has_duration {
                    if entry.entry.duration.is_some() != has_duration {
                        return false;
                    }
                }

                true
            })
            .collect()
    }
}

pub struct StingSourceData {
    pub pool: sqlx::PgPool,
    pub reqwest: reqwest::Client,
    pub cache_http: botox::cache::CacheHttpImpl,
    pub silverpelt_cache: std::sync::Arc<crate::cache::SilverpeltCache>,
}

impl StingSourceData {
    pub fn from_ctx(ctx: &serenity::all::Context) -> Self {
        let data = ctx.data::<crate::data::Data>();
        Self {
            pool: data.pool.clone(),
            reqwest: data.reqwest.clone(),
            cache_http: botox::cache::CacheHttpImpl::from_ctx(ctx),
            silverpelt_cache: data.silverpelt_cache.clone(),
        }
    }
}

/// As multiple modules may use and store stings in their own way,
/// StingSource is a common abstraction for to store sources
///
/// Ex: moderation can now store stings in moderation__actions, this
/// can then be shared with all punishment modules by defining a
/// [`StingSource`](crate::punishments::sting_source::StingSource)
#[async_trait]
pub trait StingSource
where
    Self: Send + Sync,
{
    /// The unique ID of the sting source
    fn id(&self) -> String;

    /// A description of the sting source
    fn description(&self) -> String;

    /// The flags of the sting source
    fn flags(&self) -> StingSourceFlags;

    /// Returns the count
    async fn count(
        &self,
        data: &StingSourceData,
        filters: StingCountFilters,
    ) -> Result<usize, crate::Error>;

    /// Fetches sting entries from the source
    async fn fetch(
        &self,
        data: &StingSourceData,
        filters: StingFetchFilters,
    ) -> Result<Vec<FullStingEntry>, crate::Error>;

    /// Creates a new sting entry
    async fn create_sting_entry(
        &self,
        data: &StingSourceData,
        entry: StingEntry,
    ) -> Result<FullStingEntry, crate::Error>;

    /// Updates a sting entry
    async fn update_sting_entry(
        &self,
        data: &StingSourceData,
        id: String,
        entry: UpdateStingEntry,
    ) -> Result<(), crate::Error>;

    /// Deletes a sting entry
    async fn delete_sting_entry(
        &self,
        data: &StingSourceData,
        id: String,
    ) -> Result<(), crate::Error>;
}

// Data source for stings
pub struct StingsDataStore {
    pub silverpelt_cache: std::sync::Arc<crate::cache::SilverpeltCache>,
}

#[async_trait]
impl module_settings::types::CreateDataStore for StingsDataStore {
    async fn create(
        &self,
        setting: &module_settings::types::ConfigOption,
        guild_id: serenity::all::GuildId,
        author: serenity::all::UserId,
        data: &module_settings::types::SettingsData,
        common_filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<Box<dyn module_settings::types::DataStore>, module_settings::types::SettingsError>
    {
        Ok(Box::new(StingsDataStoreImpl {
            setting_table: setting.table,
            setting_primary_key: setting.primary_key,
            author,
            guild_id,
            columns: setting.columns.clone(),
            valid_columns: setting.columns.iter().map(|c| c.id.to_string()).collect(),
            pool: data.pool.clone(),
            reqwest: data.reqwest.clone(),
            cache_http: data.cache_http.clone(),
            silverpelt_cache: self.silverpelt_cache.clone(),
            common_filters,
        }))
    }
}

pub struct StingsDataStoreImpl {
    // Args needed for queries
    pub pool: sqlx::PgPool,
    pub reqwest: reqwest::Client,
    pub cache_http: botox::cache::CacheHttpImpl,
    pub setting_table: &'static str,
    pub setting_primary_key: &'static str,
    pub author: serenity::all::UserId,
    pub guild_id: serenity::all::GuildId,
    pub columns: Arc<Vec<module_settings::types::Column>>,
    pub valid_columns: std::collections::HashSet<String>, // Derived from columns
    pub common_filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    pub silverpelt_cache: std::sync::Arc<crate::cache::SilverpeltCache>,
}

#[async_trait]
impl module_settings::types::DataStore for StingsDataStoreImpl {
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    async fn start_transaction(&mut self) -> Result<(), SettingsError> {
        Ok(()) // No-op for our use case
    }

    async fn commit(&mut self) -> Result<(), SettingsError> {
        Ok(()) // No-op for our use case
    }

    async fn columns(&mut self) -> Result<Vec<String>, SettingsError> {
        Ok(self.columns.iter().map(|c| c.id.to_string()).collect())
    }

    async fn fetch_all(
        &mut self,
        fields: &[String],
        filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<Vec<module_settings::state::State>, SettingsError> {
        let mut states = Vec::new();

        for module in self.silverpelt_cache.module_cache.values() {
            for source in module.sting_sources.iter() {
                let entries = source
                    .fetch(
                        &StingSourceData {
                            pool: self.pool.clone(),
                            reqwest: self.reqwest.clone(),
                            cache_http: self.cache_http.clone(),
                            silverpelt_cache: self.silverpelt_cache.clone(),
                        },
                        StingFetchFilters::from_map(&filters).map_err(|e| {
                            SettingsError::Generic {
                                message: format!("Failed to parse filters: {}", e),
                                src: "fetch_all".to_string(),
                                typ: "internal".to_string(),
                            }
                        })?,
                    )
                    .await
                    .map_err(|e| SettingsError::Generic {
                        message: format!("Failed to fetch stings: {}", e),
                        src: "fetch_all".to_string(),
                        typ: "internal".to_string(),
                    })?;

                for entry in entries {
                    let mut state = module_settings::state::State::default();

                    let serde_json::Value::Object(obj) = serde_json::to_value(entry.entry)
                        .map_err(|e| SettingsError::Generic {
                            message: format!("Failed to serialize sting entry: {}", e),
                            src: "fetch_all".to_string(),
                            typ: "internal".to_string(),
                        })?
                    else {
                        return Err(SettingsError::Generic {
                            message: "Failed to serialize sting entry".to_string(),
                            src: "fetch_all".to_string(),
                            typ: "internal".to_string(),
                        });
                    };

                    for (k, v) in obj {
                        if !fields.is_empty() && !fields.contains(&k) {
                            continue;
                        }

                        state
                            .state
                            .insert(k, splashcore_rs::value::Value::from_json(&v));
                    }

                    states.push(state);
                }
            }
        }

        Ok(states)
    }

    async fn matching_entry_count(
        &mut self,
        filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<usize, SettingsError> {
        let mut count = 0;

        for module in self.silverpelt_cache.module_cache.values() {
            for source in module.sting_sources.iter() {
                count += source
                    .count(
                        &StingSourceData {
                            pool: self.pool.clone(),
                            reqwest: self.reqwest.clone(),
                            cache_http: self.cache_http.clone(),
                            silverpelt_cache: self.silverpelt_cache.clone(),
                        },
                        StingCountFilters::from_map(&filters).map_err(|e| {
                            SettingsError::Generic {
                                message: format!("Failed to parse filters: {}", e),
                                src: "matching_entry_count".to_string(),
                                typ: "internal".to_string(),
                            }
                        })?,
                    )
                    .await
                    .map_err(|e| SettingsError::Generic {
                        message: format!("Failed to count stings: {}", e),
                        src: "matching_entry_count".to_string(),
                        typ: "internal".to_string(),
                    })?;
            }
        }

        Ok(count)
    }

    async fn create_entry(
        &mut self,
        _entry: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<module_settings::state::State, SettingsError> {
        Err(SettingsError::OperationNotSupported {
            operation: OperationType::Create,
        })
    }

    async fn update_matching_entries(
        &mut self,
        _filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
        _entry: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<(), SettingsError> {
        Err(SettingsError::OperationNotSupported {
            operation: OperationType::Update,
        })
    }

    async fn delete_matching_entries(
        &mut self,
        _filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<(), SettingsError> {
        Err(SettingsError::OperationNotSupported {
            operation: OperationType::Delete,
        })
    }
}

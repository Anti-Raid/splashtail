/// In order to consolidate temporary punishments (tempbans) etc, the `punishments`
/// module will provide a temporary punishments source to handle all bans in one single place
use dashmap::DashMap;
use futures_util::future::BoxFuture;
use once_cell::sync::Lazy;
use serenity::all::{GuildId, UserId};
use std::sync::Arc;

/// An action that can be reverted by the temp punishment module
#[derive(Debug, Clone)]
pub enum Action {
    Ban,
    RemoveAllRoles,
}

/// Stores a map of all sting sources
///
/// Note that modules wanting to add sting sources
/// should call [`add_sting_source`](crate::punishments::sting_source::add_sting_source)
/// to add their sting source to this map
pub static SOURCES: Lazy<DashMap<String, Arc<Source>>> = Lazy::new(DashMap::new);

/// Allows a module to add a new source for a temporary punishment
pub fn add_source(source: Source) {
    SOURCES.insert(source.id.clone(), Arc::new(source));
}

/// This struct contains data about a temporary punishment
///
/// As multiple modules may use and store temporary punishments in their own way,
/// Entry is a common abstraction for the temp_punishment module
///
/// Note that all punishment entries must be expired
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Entry {
    /// The ID of the entry
    pub id: String,
    /// The user ID of the affected user
    pub user_id: UserId,
    /// The moderator who created the action
    pub moderator: UserId,
    /// The guild ID of the affected guild
    pub guild_id: GuildId,
    /// Action to revert
    pub action: Action,
    /// Duration of the temp punishment, for audit logging
    ///
    /// Note that the punishment must still be expired when created as an entry
    pub duration: std::time::Duration,
    /// Number of stings for the punishment, for audit logging
    pub stings: i32,
    /// The reason for the punishment
    pub reason: Option<String>,
    /// When the temporary punishment was created
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub type Fetch = Box<
    dyn Send
        + Sync
        + for<'a> Fn(
            &'a serenity::all::Context,
        ) -> BoxFuture<'a, Result<Vec<Entry>, base_data::Error>>,
>;

pub type LogError = Box<
    dyn Send
        + Sync
        + for<'a> Fn(
            &'a serenity::all::Context,
            &'a Entry,
            Option<String>,
        ) -> BoxFuture<'a, Result<(), base_data::Error>>,
>;

/// As multiple modules may use and store temporary punishments in their own way,
/// Source is a common abstraction for the temp_punishment module
/// to store sources
///
/// Ex: moderation can now store temp_punishments in moderation__actions, this
/// can then be shared with the temp_punishments module by defining a
/// [`Source`](crate::temp_punishments::source::Source)
#[allow(dead_code)]
pub struct Source {
    pub id: String,
    pub description: String,
    pub fetch: Fetch,
    pub log_error: LogError,
}

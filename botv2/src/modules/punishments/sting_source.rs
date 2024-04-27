use serenity::all::{GuildId, UserId};
use futures_util::future::BoxFuture;
use dashmap::DashMap;
use once_cell::sync::Lazy;

/// Stores a map of all sting sources
///
/// Note that modules wanting to add sting sources
/// should call [`add_sting_source`](crate::modules::punishments::sting_source::add_sting_source)
/// to add their sting source to this map 
pub static STING_SOURCES: Lazy<DashMap<String, StingSource>> =
    Lazy::new(DashMap::new);

pub fn add_sting_source(source: StingSource) {
    STING_SOURCES.insert(source.id.clone(), source);
}

/// This struct contains data about a sting
/// 
/// As multiple modules may use and store stings in their own way,
/// StingEntry is a common abstraction for the punishment module
/// to store string data and reason for presentation to users etc.
#[derive(Debug, Clone)]
pub struct StingEntry {
    /// The user ID of the user who was stung
    pub user_id: UserId,
    /// The guild ID of the guild where the sting occurred
    pub guild_id: GuildId,
    /// The number of stings the user has in this entry
    pub stings: u32,
    /// The reason for the stings
    pub reason: String,
    /// When the sting entry was created
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub type StingFetch = Box<
    dyn Send
        + Sync
        + for<'a> Fn(
            &'a serenity::all::Context,
            &GuildId,
            &UserId,
        ) -> BoxFuture<'a, Result<Vec<StingEntry>, crate::Error>>,
>;

/// As multiple modules may use and store stings in their own way,
/// StringSource is a common abstraction for the punishment module
/// to store sources
/// 
/// Ex: moderation can now store stings in moderation__actions, this
/// can then be shared with the punishment module by defining a 
/// [`StingSource`](crate::modules::punishments::sting_source::StingSource)
pub struct StingSource {
    pub id: String,
    pub description: String,
    pub fetch: StingFetch,
}
use async_trait::async_trait;
use bitflags::bitflags;
use serenity::all::{GuildId, UserId};

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

/// What created the sting
#[derive(Debug, Clone)]
pub enum StingCreator {
    /// The sting was created by a user
    User(UserId),
    /// The sting was created by the system
    System,
}

/// An action taken due to this sting
#[derive(Debug, Clone)]
pub enum Action {
    None,
    Ban,
    RemoveAllRoles,
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::None => write!(f, "none"),
            Action::Ban => write!(f, "ban"),
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
            "remove_all_roles" => Ok(Action::RemoveAllRoles),
            _ => Err(format!("Invalid action: {}", s).into()),
        }
    }
}

/// This struct contains data about a sting
///
/// As multiple modules may use and store stings in their own way,
/// StingEntry is a common abstraction for the punishment module
/// to store string data and reason for presentation to users etc.
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
pub struct StingFetchFilters {
    pub user_id: Option<UserId>,
    pub guild_id: Option<GuildId>,
    pub state: Option<StingState>,
    pub expired: Option<bool>,
    pub has_duration: Option<bool>, // Is 'ephemeral' AKA has a duration
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

/// As multiple modules may use and store stings in their own way,
/// StringSource is a common abstraction for to store sources
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

    /// Fetches sting entries from the source
    async fn fetch(
        &self,
        ctx: &serenity::all::Context,
        filters: StingFetchFilters,
    ) -> Result<Vec<FullStingEntry>, crate::Error>;

    /// Creates a new sting entry
    async fn create_sting_entry(
        &self,
        ctx: &serenity::all::Context,
        entry: StingEntry,
    ) -> Result<FullStingEntry, crate::Error>;

    /// Updates a sting entry
    async fn update_sting_entry(
        &self,
        ctx: &serenity::all::Context,
        id: String,
        entry: UpdateStingEntry,
    ) -> Result<(), crate::Error>;

    /// Deletes a sting entry
    async fn delete_sting_entry(
        &self,
        ctx: &serenity::all::Context,
        id: String,
    ) -> Result<(), crate::Error>;
}

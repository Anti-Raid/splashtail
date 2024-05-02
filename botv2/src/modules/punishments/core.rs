use super::sting_source::StingEntry;
use crate::silverpelt::module_config::is_module_enabled;
use serenity::all::{GuildId, UserId};
use strum_macros::{Display, EnumString, VariantNames};
use serde::{Deserialize, Serialize};

pub struct ConsolidatedStingEntry {
    pub source_id: String,
    pub entry: StingEntry,
}

/// This struct is a wrapper around a list of consolidated sting entries
pub struct ConsolidatedStingEntries {
    /// The list of consolidated sting entries
    pub entries: Vec<ConsolidatedStingEntry>,

    // The total sting count, is determined automatically on calls to sting_count()
    sting_count: Option<usize>,
}

impl ConsolidatedStingEntries {
    /// Returns the total number of stings in the list
    ///
    /// Note that this function caches the result
    /// so calling it multiple times will not result in
    /// a new sting count calculation
    pub fn sting_count(&mut self) -> usize {
        if let Some(count) = self.sting_count {
            return count;
        }

        let mut total_count: usize = 0;
        for entry in &self.entries {
            let count = entry.entry.stings as usize;
            total_count += count;
        }

        self.sting_count = Some(total_count);
        total_count
    }
}

/// Returns all sting entries that a user has. This can be useful when triggering punishments to users
/// or just showing them a user friendly list of all the stings they have.
pub async fn get_all_user_sting_entries(
    ctx: &serenity::all::Context,
    guild_id: GuildId,
    user_id: UserId,
) -> Result<ConsolidatedStingEntries, crate::Error> {
    let data = ctx.data::<crate::Data>();
    if !is_module_enabled(&data.pool, guild_id, "punishments").await? {
        // Punishments module is not enabled
        return Err("Punishments module is not enabled".into());
    }

    let mut stings = vec![];

    for source in super::sting_source::STING_SOURCES.iter() {
        let source = source.value();
        let entries = (source.fetch)(ctx, &guild_id, &user_id).await?;

        for entry in entries {
            stings.push(ConsolidatedStingEntry {
                source_id: source.id.clone(),
                entry,
            });
        }
    }

    Ok(ConsolidatedStingEntries {
        entries: stings,
        sting_count: None,
    })
}

/// Poise helper to allow displaying the different punishment actions in a menu
#[derive(poise::ChoiceParameter)]
pub enum ActionsChoices {
    #[name = "Timeout"]
    Timeout,
    #[name = "Kick"]
    Kick,
    #[name = "Ban"]
    Ban,
}

impl ActionsChoices {
    pub fn resolve(self) -> Actions {
        match self {
            Self::Timeout => Actions::Timeout,
            Self::Kick => Actions::Kick,
            Self::Ban => Actions::Ban,
        }
    }
}

#[derive(EnumString, Display, PartialEq, VariantNames, Clone, Debug, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
pub enum Actions {
    Timeout,
    Kick,
    Ban,
}
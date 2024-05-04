use super::sting_source::StingEntry;
use crate::silverpelt::module_config::is_module_enabled;
use serenity::all::{GuildId, UserId, RoleId, EditMember, Timestamp};
use strum_macros::{Display, EnumString, VariantNames};
use serde::{Deserialize, Serialize};
use crate::silverpelt::utils::serenity_utils::greater_member_hierarchy;
use crate::silverpelt::proxysupport::{guild, member_in_guild};
use std::collections::HashSet;

/// This struct is a wrapper around a sting entry that has been consolidated
#[allow(dead_code)]
pub struct ConsolidatedStingEntry {
    pub source_id: String,
    pub entry: StingEntry,
}

/// This struct is a wrapper around a list of consolidated sting entries
pub struct ConsolidatedStingEntries {
    /// The list of consolidated sting entries
    pub entries: Vec<ConsolidatedStingEntry>,

    // The total sting count, is determined automatically on calls to sting_count()
    sting_count: Option<i32>,
}

impl ConsolidatedStingEntries {
    /// Returns the total number of stings in the list
    ///
    /// Note that this function caches the result
    /// so calling it multiple times will not result in
    /// a new sting count calculation
    pub fn sting_count(&mut self) -> i32 {
        if let Some(count) = self.sting_count {
            return count;
        }

        let mut total_count: i32 = 0;
        for entry in &self.entries {
            let count = entry.entry.stings;
            total_count += count;
        }

        self.sting_count = Some(total_count);
        total_count
    }

    /// Returns all sting entries that a user has. This can be useful when triggering punishments to users
    /// or just showing them a user friendly list of all the stings they have.
    #[allow(dead_code)]
    pub async fn get_entries_for_guild_user(
        ctx: &serenity::all::Context,
        guild_id: GuildId,
        user_id: UserId,
    ) -> Result<Self, crate::Error> {
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

        Ok(Self {
            entries: stings,
            sting_count: None,
        })
    }
}

/// This struct stores a guild punishment that can then be used to trigger punishments
/// on a user through the bot 
#[derive(Serialize, Deserialize, Clone)]
pub struct GuildPunishment {
    pub id: String,
    pub guild_id: GuildId,
    pub creator: UserId,
    pub stings: i32,
    pub action: Action,
    pub modifiers: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// A guild punishment list is internally a Vec<GuildPunishment> but has special methods
/// to make things easier when coding punishments
/// 
/// Note that the guild punishment list should not be modified directly
pub struct GuildPunishmentList {
    punishments: Vec<GuildPunishment>,
    
    /// dominant_action is a cached value that is calculated when calling get_dominant_action
    dominant_action: Option<Action>,
}

impl GuildPunishmentList {
    /// Gets the punishment list of a specific guild
    pub async fn guild(
        ctx: &serenity::all::Context,
        guild_id: GuildId,
    ) -> Result<Self, crate::Error> {
        let data = ctx.data::<crate::Data>();
        let rec = sqlx::query!(
            "SELECT id, guild_id, creator, stings, action, modifiers, created_at FROM punishments__guild_punishment_list WHERE guild_id = $1",
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
                action: row.action.parse()?,
                modifiers: row.modifiers,
                created_at: row.created_at,
            });
        }

        Ok(Self {
            punishments,
            dominant_action: None,
        })
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
    pub fn filter(self, stings: i32) -> Self {
        let mut punishments = vec![];

        for punishment in self.punishments {
            if punishment.stings <= stings {
                punishments.push(punishment);
            }
        }

        Self {
            punishments,
            dominant_action: None,
        }
    }

    /// `get_dominant_action` returns the dominant action in the list
    pub fn get_dominant_action(&mut self) -> Option<Action> {
        if let Some(dominant_action) = self.dominant_action {
            return Some(dominant_action);
        }

        if self.punishments.is_empty() {
            return None;
        }

        let mut dominant_action = self.punishments[0].action;

        for punishment in &self.punishments {
            if punishment.action.is_dominant_to(&dominant_action) {
                dominant_action = punishment.action;
            }
        }

        self.dominant_action = Some(dominant_action);

        Some(dominant_action)
    }

    /// `get_punishments_to_apply` returns a list of punishments that should be applied to a user
    /// based on their dominance
    pub fn get_punishments_to_apply(mut self) -> Vec<GuildPunishment> {
        let mut punishments = Vec::new();
        let Some(dominant_action) = self.get_dominant_action() else {
            return punishments;
        };

        for punishment in self.punishments {
            if punishment.action.is_dominant_to(&dominant_action) {
                punishments.push(punishment);
            }
        }

        punishments
    }
}

/// Poise helper to allow displaying the different punishment actions in a menu
#[derive(poise::ChoiceParameter)]
pub enum ActionChoices {
    #[name = "Timeout"]
    Timeout,
    #[name = "Kick"]
    Kick,
    #[name = "Ban"]
    Ban,
    #[name = "Remove All Roles"]
    RemoveAllRoles,
}

impl ActionChoices {
    pub fn resolve(self) -> Action {
        match self {
            Self::Timeout => Action::Timeout,
            Self::Kick => Action::Kick,
            Self::Ban => Action::Ban,
            Self::RemoveAllRoles => Action::RemoveAllRoles,
        }
    }
}

#[derive(EnumString, Display, PartialEq, VariantNames, Copy, Clone, Debug, Serialize, Deserialize, Hash, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum Action {
    Timeout,
    Kick,
    Ban,
    RemoveAllRoles,
}

impl Action {
    /// Returns the 'standing' for a action
    /// 
    /// An action with higher stading is considered as dominant
    /// 
    /// Non-dominant actions should be ignored in favor of dominant actions
    /// 
    /// This stops cases where a user is kicked and then banned for example
    pub fn standing(&self) -> i32 {
        match self {
            Self::Ban => 1,
            Self::Kick => 2,

            // Remove all roles and timeout are considered the same priority
            Self::RemoveAllRoles => 3,
            Self::Timeout => 3,
        }
    }

    pub fn is_dominant_to(&self, other: &Action) -> bool {
        self.standing() < other.standing()
    }

    /// Attempts to carry out the given action on a given user (ID)
    /// TODO: Allow customizing the timeout duration in the future
    /// TODO: Allow showing kick reason (which punishment was hit)
    pub async fn execute(
        &self,
        ctx: &serenity::all::Context,
        punishment: &GuildPunishment,
        guild_id: GuildId,
        user_id: UserId,
    ) -> Result<Option<String>, crate::Error> {
        let data = ctx.data::<crate::Data>();
        let cache_http = botox::cache::CacheHttpImpl::from_ctx(ctx);

        let guild = guild(
            &cache_http, 
            &data.reqwest,
            guild_id
        ).await?;

        let bot_userid = ctx.cache.current_user().id;
        let Some(bot) = member_in_guild(
            &cache_http, 
            &data.reqwest,
            guild_id, 
            bot_userid
        ).await? else {
            return Err("Bot not found".into());
        };

        let Some(mut user) = member_in_guild(
            &cache_http, 
            &data.reqwest,
            guild_id, 
            user_id
        ).await? else {
            return Err("User not found".into());
        };

        for modifier in &punishment.modifiers {
            if modifier.is_empty() {
                continue
            }

            let negator = modifier.chars().nth(0).unwrap_or('-') == '-';
            let splitted = modifier.splitn(2, ':').collect::<Vec<&str>>();

            let (modifier_type, modifier_id) = match splitted[..] {
                [a, b] => (a, b),
                [a] => (a, ""),
                _ => continue,
            };

            let matches_modifier = match modifier_type {
                "r" => {
                    let role_id = modifier_id.parse::<RoleId>().unwrap_or(RoleId::new(0));
                    user.roles.contains(&role_id)
                },
                "u" => {
                    let user_id = modifier_id.parse::<UserId>().unwrap_or(UserId::new(0));
                    user.user.id == user_id
                },
                _ => false,
            };

            if negator && matches_modifier {
                return Ok(Some("User matches a negated modifier".to_string()))
            } else if !negator && !matches_modifier {
                return Ok(Some("User does not match a specified modifier".to_string()))
            }
        }

        if greater_member_hierarchy(&guild, &bot, &user).unwrap_or(user.user.id) == user.user.id {
            return Err("Bot does not have the required permissions to carry out this action".into());
        }

        match self {
            Action::Timeout => {
                // TODO: Allow customizing the timeout duration in the future
                let new_time = chrono::Utc::now() + chrono::Duration::minutes(5);

                user.edit(
                    &ctx.http,
                    EditMember::new()
                    .disable_communication_until(Timestamp::from(new_time))
                )
                .await?;
            },
            Action::Kick => {
                user.kick(&ctx.http, Some(
                    "Punishment applied to user"
                )).await?;
            },
            Action::Ban => {
                user.ban(&ctx.http, 0, Some("Punishment applied to user")).await?;
            },
            Action::RemoveAllRoles => {
                user.edit(
                    &ctx.http, 
                    EditMember::new()
                    .roles(Vec::new())
                ).await?;
            },
        }

        // TODO: Implement the actual action

        Ok(None)
    }
}

pub async fn trigger_punishment(
    ctx: &serenity::all::Context,
    guild_id: GuildId,
    user_id: UserId,
    ignore_actions: HashSet<Action>,
) -> Result<(), crate::Error> {
    let mut sting_entries = ConsolidatedStingEntries::get_entries_for_guild_user(ctx, guild_id, user_id).await?;
    let sting_count = sting_entries.sting_count();

    let punishments = GuildPunishmentList::guild(ctx, guild_id).await?.filter(sting_count);
    let apply_punishments = punishments.get_punishments_to_apply();

    for punishment in apply_punishments {
        if ignore_actions.contains(&punishment.action) {
            continue;
        }

        punishment.action.execute(ctx, &punishment, guild_id, user_id).await?;
    }

    Ok(())
}
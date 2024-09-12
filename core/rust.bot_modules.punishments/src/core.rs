use proxy_support::{guild, member_in_guild};
use serde::{Deserialize, Serialize};
use serenity::all::{EditMember, GuildId, RoleId, Timestamp, UserId};
use silverpelt::module_config::is_module_enabled;
use silverpelt::sting_sources::{self, FullStingEntry, StingFetchFilters};
use std::collections::{HashMap, HashSet};
use strum_macros::{Display, EnumString, VariantNames};

pub type StingEntryMap = HashMap<String, Vec<FullStingEntry>>;

/// Returns all sting entries that a server has. This can be useful when triggering punishments to users
/// or just showing them a user friendly list of all the stings
#[allow(dead_code)]
pub async fn get_consolidated_sting_entries(
    ctx: &serenity::all::Context,
    guild_id: GuildId,
    filters: Option<StingFetchFilters>,
) -> Result<StingEntryMap, silverpelt::Error> {
    let source_data = sting_sources::StingSourceData::from_ctx(ctx);

    if !is_module_enabled(
        &source_data.silverpelt_cache,
        &source_data.pool,
        guild_id,
        "punishments",
    )
    .await?
    {
        // Punishments module is not enabled
        return Err("Punishments module is not enabled".into());
    }

    let filters = filters.unwrap_or(StingFetchFilters {
        guild_id: Some(guild_id),
        ..Default::default()
    });

    let mut stings = HashMap::new();

    for (_, module) in source_data.silverpelt_cache.module_cache.iter() {
        for source in module.sting_sources.iter() {
            let entries = source.fetch(&source_data, filters.clone()).await?;

            stings.insert(source.id(), entries);
        }
    }

    Ok(stings)
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
    pub duration: Option<i32>,
    pub modifiers: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// A guild punishment list is internally a Vec<GuildPunishment> but has special methods
/// to make things easier when coding punishments
///
/// Note that the guild punishment list should not be modified directly
#[derive(Serialize, Deserialize, Clone)]
pub struct GuildPunishmentList {
    punishments: Vec<GuildPunishment>,
}

impl GuildPunishmentList {
    /// Gets the punishment list of a specific guild
    pub async fn guild(
        ctx: &serenity::all::Context,
        guild_id: GuildId,
    ) -> Result<Self, silverpelt::Error> {
        let data = ctx.data::<silverpelt::data::Data>();
        let rec = sqlx::query!(
            "SELECT id, guild_id, creator, stings, action, modifiers, created_at, EXTRACT(seconds FROM duration)::integer AS duration FROM punishments__guild_punishment_list WHERE guild_id = $1",
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
                duration: row.duration,
                created_at: row.created_at,
            });
        }

        Ok(Self { punishments })
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

        Self { punishments }
    }

    /// `get_dominant` returns the dominat punishments in the list
    ///
    /// Dominant punishments are the punishments with the highest standing and the highest duration
    pub fn get_dominating(&self) -> Vec<&GuildPunishment> {
        if self.punishments.is_empty() {
            return Vec::new();
        }

        let mut curr_dominant_punishment = &self.punishments[0];
        let mut dominant_punishments = vec![]; // Start with empty list, the iteration with handle the rest (including the first punishment)

        for punishment in &self.punishments {
            if punishment
                .action
                .is_dominant_to(&curr_dominant_punishment.action)
                && punishment.duration.unwrap_or_default()
                    > curr_dominant_punishment.duration.unwrap_or_default()
            {
                curr_dominant_punishment = punishment;
                dominant_punishments.clear();
                dominant_punishments.push(punishment);
            }
        }

        dominant_punishments
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

#[derive(
    EnumString,
    Display,
    PartialEq,
    VariantNames,
    Copy,
    Clone,
    Debug,
    Serialize,
    Deserialize,
    Hash,
    Eq,
)]
#[strum(serialize_all = "snake_case")]
pub enum Action {
    Timeout,
    Kick,
    Ban,
    RemoveAllRoles,
    Unknown,
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
            Self::Unknown => 0,
            Self::Ban => 1,
            Self::Kick => 2,

            // Remove all roles and timeout are considered the same priority
            Self::RemoveAllRoles => 3,
            Self::Timeout => 3,
        }
    }

    pub fn is_dominant_to(&self, other: &Action) -> bool {
        self.standing() <= other.standing()
    }

    /// Attempts to carry out the given action on a user
    ///
    /// TODO: Improve audit log reasons
    pub async fn execute(
        &self,
        ctx: &serenity::all::Context,
        punishment: &GuildPunishment,
        guild_id: GuildId,
        user_id: UserId,
        _trigger: sting_sources::StingCreator,
    ) -> Result<Option<String>, silverpelt::Error> {
        let data = ctx.data::<silverpelt::data::Data>();
        let cache_http = botox::cache::CacheHttpImpl::from_ctx(ctx);

        let guild = guild(&cache_http, &data.reqwest, guild_id).await?;

        let bot_userid = ctx.cache.current_user().id;
        let Some(bot) = member_in_guild(&cache_http, &data.reqwest, guild_id, bot_userid).await?
        else {
            return Err("Bot not found".into());
        };

        if user_id == bot_userid {
            return Err("Bot cannot be punished".into());
        }

        let Some(mut user) = member_in_guild(&cache_http, &data.reqwest, guild_id, user_id).await?
        else {
            return Ok(None);
        };

        // Handle modifiers
        //
        // Modifiers are used to limit/expand the scope of a punishment
        for modifier in &punishment.modifiers {
            if modifier.is_empty() {
                continue;
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
                }
                "u" => {
                    let user_id = modifier_id.parse::<UserId>().unwrap_or(UserId::new(0));
                    user.user.id == user_id
                }
                _ => false,
            };

            if negator && matches_modifier {
                return Ok(Some("User matches a negated modifier".to_string()));
            } else if !negator && !matches_modifier {
                return Ok(Some("User does not match a specified modifier".to_string()));
            }
        }

        if guild
            .greater_member_hierarchy(&bot, &user)
            .unwrap_or(user.user.id)
            == user.user.id
        {
            return Err(
                "Bot does not have the required permissions to carry out this action".into(),
            );
        }

        match self {
            Action::Unknown => {
                // Do nothing
                return Ok(None);
            }
            Action::Timeout => {
                let timeout_duration = if let Some(duration) = punishment.duration {
                    chrono::Duration::seconds(duration as i64)
                } else {
                    chrono::Duration::minutes(5)
                };

                let new_time = chrono::Utc::now() + timeout_duration;

                user.edit(
                    &ctx.http,
                    EditMember::new()
                        .disable_communication_until(Timestamp::from(new_time))
                        .audit_log_reason(
                            format!("Punishment applied to user: {}", punishment.id).as_str(),
                        ),
                )
                .await?;
            }
            Action::Kick => {
                user.kick(
                    &ctx.http,
                    Some(format!("Punishment applied to user: {}", punishment.id).as_str()),
                )
                .await?;
            }
            Action::Ban => {
                user.ban(
                    &ctx.http,
                    0,
                    Some(format!("Punishment applied to user: {}", punishment.id).as_str()),
                )
                .await?;
            }
            Action::RemoveAllRoles => {
                user.edit(
                    &ctx.http,
                    EditMember::new().roles(Vec::new()).audit_log_reason(
                        format!("Punishment applied to user: {}", punishment.id).as_str(),
                    ),
                )
                .await?;
            }
        }

        Ok(None)
    }
}

pub fn get_per_user_sting_counts(sting_entries_map: &StingEntryMap) -> HashMap<UserId, i32> {
    // TODO: Allow scoping punishments based on sting source

    let mut per_user_sting_counts = HashMap::new();

    let mut total_system_punishments = 0;
    for (_src, entries) in sting_entries_map {
        for entry in entries {
            match entry.entry.creator {
                sting_sources::StingCreator::User(user_id) => {
                    let count = per_user_sting_counts.entry(user_id).or_insert(0);
                    *count += 1;
                }
                sting_sources::StingCreator::System => {
                    total_system_punishments += 1;
                }
            }
        }
    }

    // Add the total system punishments to all users
    for (_, count) in per_user_sting_counts.iter_mut() {
        *count += total_system_punishments;
    }

    per_user_sting_counts
}

pub async fn trigger_punishment(
    ctx: &serenity::all::Context,
    guild_id: GuildId,
    creator: sting_sources::StingCreator,
    ignore_actions: HashSet<Action>,
) -> Result<(), silverpelt::Error> {
    let sting_entries = get_consolidated_sting_entries(ctx, guild_id, None).await?;

    let punishments = GuildPunishmentList::guild(ctx, guild_id).await?;

    let per_user_sting_counts = get_per_user_sting_counts(&sting_entries);

    async {}.await; // Insert await point

    for (user_id, sting_count) in per_user_sting_counts {
        let punishments = punishments.clone().filter(sting_count.try_into()?);

        let apply_punishments = punishments.get_dominating();

        for punishment in apply_punishments {
            if ignore_actions.contains(&punishment.action) {
                continue;
            }

            punishment
                .action
                .execute(ctx, punishment, guild_id, user_id, creator)
                .await?;
        }
    }

    Ok(())
}

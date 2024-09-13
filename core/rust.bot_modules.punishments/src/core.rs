use proxy_support::{guild, member_in_guild};
use serenity::all::{GuildId, RoleId, UserId};
use silverpelt::module_config::is_module_enabled;
use silverpelt::punishments::*;
use silverpelt::sting_sources::{self, FullStingEntry, StingFetchFilters};
use std::collections::HashMap;

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

/// A guild punishment list is internally a Vec<GuildPunishment> but has special methods
/// to make things easier when coding punishments
///
/// Note that the guild punishment list should not be modified directly
#[derive(Clone)]
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

        let actions_map =
            silverpelt::punishments::get_punishment_actions_for_guild(guild_id, &data).await?;

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
                action: {
                    let action = from_punishment_action_string(&actions_map, &row.action);

                    let Ok(action) = action else {
                        continue; // Skip this punishment if the action is not found
                    };

                    action.into()
                },
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
    pub fn filter(&self, stings: i32) -> Vec<GuildPunishment> {
        let mut punishments = vec![];

        for punishment in self.punishments.iter() {
            if punishment.stings <= stings {
                punishments.push(punishment.clone());
            }
        }

        punishments
    }
}

/// Returns (per_user_sting_counts, total_system_punishments, total sting count)
pub fn get_sting_counts(sting_entries_map: &StingEntryMap) -> (HashMap<UserId, i32>, i32, i32) {
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

    // Total sting count is the sum of all user stings and system stings prior to adding the system stings to all users
    let mut total_sting_count = total_system_punishments;
    for (_, count) in per_user_sting_counts.iter() {
        total_sting_count += count;
    }

    // Now add the total system punishments to all users
    for (_, count) in per_user_sting_counts.iter_mut() {
        *count += total_system_punishments;
    }

    (
        per_user_sting_counts,
        total_system_punishments,
        total_sting_count,
    )
}

pub async fn trigger_punishment(
    ctx: &serenity::all::Context,
    guild_id: GuildId,
    _creator: sting_sources::StingCreator,
) -> Result<(), silverpelt::Error> {
    let sting_entries = get_consolidated_sting_entries(ctx, guild_id, None).await?;

    if sting_entries.is_empty() {
        return Ok(());
    }

    let punishments = GuildPunishmentList::guild(ctx, guild_id).await?;

    if punishments.punishments().is_empty() {
        return Ok(());
    }

    let (per_user_sting_counts, _total_system_punishments, total_sting_count) =
        get_sting_counts(&sting_entries);

    let data = ctx.data::<silverpelt::data::Data>();
    let cache_http = botox::cache::CacheHttpImpl::from_ctx(ctx);

    let bot_userid = ctx.cache.current_user().id;
    let Some(mut bot) = member_in_guild(&cache_http, &data.reqwest, guild_id, bot_userid).await?
    else {
        return Err("Bot not found".into());
    };

    let punishment_data = PunishmentActionData {
        cache_http: cache_http.clone(),
        pool: data.pool.clone(),
        reqwest: data.reqwest.clone(),
        object_store: data.object_store.clone(),
    };

    let mut guild = guild(&cache_http, &data.reqwest, guild_id).await?;

    // First apply global punishments, the total sting count is used for this
    let system_punishments = punishments.filter(total_sting_count);

    if !system_punishments.is_empty() {
        for punishment in system_punishments.iter() {
            match &*punishment.action {
                PunishmentAction::Global(action) => {
                    action
                        .create(&punishment_data, &mut guild, &mut bot, &system_punishments)
                        .await?;
                }
                _ => {}
            }
        }
    }

    for (user_id, sting_count) in per_user_sting_counts {
        let Some(mut user) = member_in_guild(&cache_http, &data.reqwest, guild_id, user_id).await?
        else {
            return Ok(());
        };

        if guild
            .greater_member_hierarchy(&bot, &user)
            .unwrap_or(user.user.id)
            == user.user.id
        {
            return Err(
                "Bot does not have the required permissions to carry out this action".into(),
            );
        }

        let punishments = punishments.filter(sting_count.try_into()?);

        for punishment in punishments.iter() {
            let mut modifier_match = None;
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
                    modifier_match = Some("User matches a negated modifier".to_string());
                    break;
                } else if !negator && !matches_modifier {
                    modifier_match = Some("User does not match a specified modifier".to_string());
                    break;
                }
            }

            if let Some(reason) = modifier_match {
                log::trace!(
                    "Skipping punishment {} for user {} due to modifier match reason: {}",
                    punishment.id,
                    user_id,
                    reason
                );
                continue;
            }

            match &*punishment.action {
                PunishmentAction::User(action) => {
                    action
                        .create(&punishment_data, &mut user, &mut bot, &punishments)
                        .await?;
                }
                _ => {}
            }
        }
    }

    Ok(())
}

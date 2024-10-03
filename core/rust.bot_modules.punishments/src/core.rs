use proxy_support::{guild, member_in_guild};
use serenity::all::{GuildId, UserId};
use silverpelt::punishments::*;
use std::sync::Arc;

/// This struct stores a guild punishment autotrigger that can then be used to trigger punishments
/// on a user through the bot based on sting count
#[derive(Clone)]
pub struct GuildPunishmentAutoTrigger {
    pub id: String,
    pub guild_id: GuildId,
    pub creator: UserId,
    pub stings: i32,
    pub action: Arc<dyn PunishmentAction>,
    pub duration: Option<i32>,
    pub modifiers: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// A guild punishment list is internally a Vec<GuildPunishment> but has special methods
/// to make things easier when coding punishments
///
/// Note that the guild punishment list should not be modified directly
#[derive(Clone)]
pub struct GuildPunishmentAutoTriggerList {
    punishments: Vec<GuildPunishmentAutoTrigger>,
}

impl GuildPunishmentAutoTriggerList {
    /// Gets the punishment list of a specific guild
    pub async fn guild(
        ctx: &serenity::all::Context,
        guild_id: GuildId,
    ) -> Result<Self, silverpelt::Error> {
        let data = ctx.data::<silverpelt::data::Data>();

        let actions_map =
            silverpelt::punishments::get_punishment_actions_for_guild(guild_id, &data).await?;

        let rec = sqlx::query!(
                "SELECT id, guild_id, creator, stings, action, modifiers, created_at, EXTRACT(seconds FROM duration)::integer AS duration FROM punishments__autotriggers WHERE guild_id = $1",
                guild_id.to_string(),
            )
            .fetch_all(&data.pool)
            .await?;

        let mut punishments = vec![];

        for row in rec {
            punishments.push(GuildPunishmentAutoTrigger {
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
    pub fn punishments(&self) -> &Vec<GuildPunishmentAutoTrigger> {
        &self.punishments
    }

    /// Filter returns a new GuildPunishmentList with only the punishments that match the set of filters
    ///
    /// Note that this drops the existing punishment list
    pub fn filter(&self, stings: i32) -> Vec<GuildPunishmentAutoTrigger> {
        let mut punishments = vec![];

        for punishment in self.punishments.iter() {
            if punishment.stings <= stings {
                punishments.push(punishment.clone());
            }
        }

        punishments
    }
}

// TODO: Readd support for modifiers later
pub async fn autotrigger(
    ctx: &serenity::all::Context,
    guild_id: GuildId,
) -> Result<(), silverpelt::Error> {
    let data = ctx.data::<silverpelt::data::Data>();

    let (per_user_sting_counts, _system_stings) =
        silverpelt::stings::StingAggregate::total_stings_per_user(
            silverpelt::stings::get_aggregate_stings_for_guild(&data.pool, guild_id).await?,
        );

    let punishments = GuildPunishmentAutoTriggerList::guild(ctx, guild_id).await?;

    if punishments.punishments().is_empty() {
        return Ok(());
    }

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

    let guild = guild(&cache_http, &data.reqwest, guild_id).await?;

    for (user_id, sting_count) in per_user_sting_counts {
        let Some(user) = member_in_guild(&cache_http, &data.reqwest, guild_id, user_id).await?
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
            punishment
                .action
                .create(
                    &punishment_data,
                    user.user.id,
                    &mut bot,
                    format!(
                        "[Auto-Triggered] {} at {} stings",
                        punishment.action.string_form(),
                        sting_count
                    ),
                )
                .await?;

            // Add punishment
            silverpelt::punishments::PunishmentCreate {
                module: "punishments".to_string(),
                src: None,
                punishment: punishment.action.string_form(),
                creator: silverpelt::punishments::PunishmentTarget::System,
                target: silverpelt::punishments::PunishmentTarget::User(user.user.id),
                handle_log: serde_json::json!({}),
                guild_id,
                duration: None, // TODO: Auto-triggered punishments do not support duration yet
                reason: format!(
                    "[Auto-Triggered] {} at {} stings",
                    punishment.action.string_form(),
                    sting_count
                ),
                data: None,
            }
            .create(&data.pool)
            .await?;
        }
    }

    Ok(())
}

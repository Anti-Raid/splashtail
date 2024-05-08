use super::core;
use crate::modules::limits::core::Limit;
use crate::Error;
use botox::crypto::gen_random;
use poise::serenity_prelude::{GuildId, UserId};
use std::collections::{HashMap, HashSet};

pub struct HandleModAction {
    /// Guild ID
    pub guild_id: GuildId,
    /// User ID
    pub user_id: UserId,
    /// Limit to handle for the User ID in question
    pub limit: core::UserLimitTypes,
    /// Target of the action
    pub target: Option<String>,
    /// Extra data for the action
    pub action_data: serde_json::Value,
}

pub async fn handle_mod_action(
    ctx: &serenity::all::Context,
    ha: &HandleModAction,
) -> Result<(), Error> {
    let data = ctx.data::<crate::Data>();

    let guild_id = ha.guild_id;
    let limit = ha.limit;
    let user_id = ha.user_id;
    let target = ha.target.clone();
    let action_data = &ha.action_data;

    // Check limits cache
    let guild_limits: HashMap<String, Limit> = Limit::guild(&data.pool, guild_id)
        .await?
        .into_iter()
        .filter(|a| a.limit_type == limit)
        .map(|a| (a.limit_id.clone(), a))
        .collect();

    if guild_limits.is_empty() {
        // No limits for this guild
        return Ok(());
    }

    let mut tx = data.pool.begin().await?;

    let action_id = gen_random(48);

    sqlx::query!(
        "INSERT INTO limits__user_actions (action_id, guild_id, user_id, target, limit_type, action_data, created_at, stings) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        action_id,
        guild_id.to_string(),
        user_id.to_string(),
        target,
        limit.to_string(),
        action_data,
        sqlx::types::chrono::Utc::now(),
        0
    )
    .execute(&mut *tx)
    .await?;

    let mut hit_limits = Vec::new();

    let mut stings = 0;
    let mut largest_expiry = 0;
    for (_limit_id, guild_limit) in guild_limits.into_iter() {
        let stings_from_limit = guild_limit.stings;
        let limit_time_from_limit = guild_limit.limit_time;

        // Ensure the expiry is based on all limits, not just infringing
        if limit_time_from_limit > largest_expiry {
            largest_expiry = limit_time_from_limit;
        }

        // Check the limit type and user_id and guild to see if it is in the cache
        let infringing_actions = sqlx::query!(
            "select action_id from limits__user_actions where guild_id = $1 and user_id = $2 and limit_type = $3 and created_at + make_interval(secs => $4) > now()",
            guild_id.to_string(),
            user_id.to_string(),
            limit.to_string(),
            guild_limit.limit_time as f64,
        )
        .fetch_all(&mut *tx)
        .await?;

        if infringing_actions.len() >= guild_limit.limit_per as usize {
            hit_limits.push((
                infringing_actions
                    .into_iter()
                    .map(|v| v.action_id)
                    .collect::<Vec<String>>(),
                guild_limit,
            ));

            stings += stings_from_limit;
        }
    }

    if stings > 0 || largest_expiry > 0 {
        sqlx::query!(
            "UPDATE limits__user_actions SET stings = $1, stings_expiry = $2 WHERE action_id = $3",
            stings,
            sqlx::types::chrono::Utc::now() + chrono::Duration::seconds(largest_expiry),
            action_id
        )
        .execute(&mut *tx)
        .await?;

        // Delete older user actions
        sqlx::query!(
            "DELETE FROM limits__user_actions WHERE user_id = $1 AND guild_id = $2 AND created_at < now() - make_interval(secs => $3)",
            user_id.to_string(),
            guild_id.to_string(),
            largest_expiry as f64,
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    if stings > 0
        && crate::silverpelt::module_config::is_module_enabled(&data.pool, guild_id, "punishments")
            .await?
    {
        log::info!("Triggering punishment for user_id: {}", user_id);
        match crate::modules::punishments::core::trigger_punishment(
            ctx,
            guild_id,
            user_id,
            HashSet::new(),
        )
        .await
        {
            Ok(()) => {
                let mut action_ids = Vec::new();
                let mut limit_ids = Vec::new();

                for (id, limit) in hit_limits.iter() {
                    action_ids.extend(id.clone());
                    limit_ids.push(limit.limit_id.clone());
                }

                sqlx::query!(
                    "
                    INSERT INTO limits__past_hit_limits
                    (id, guild_id, user_id, limit_ids, cause, notes)
                    VALUES ($1, $2, $3, $4, $5, $6)",
                    gen_random(16),
                    guild_id.to_string(),
                    user_id.to_string(),
                    &limit_ids,
                    &action_ids,
                    &vec![]
                )
                .execute(&data.pool)
                .await?;
            }
            Err(e) => {
                log::error!("Failed to trigger punishment: {:?}", e);

                let mut action_ids = Vec::new();
                let mut limit_ids = Vec::new();

                for (id, limit) in hit_limits.iter() {
                    action_ids.extend(id.clone());
                    limit_ids.push(limit.limit_id.clone());
                }

                sqlx::query!(
                    "
                    INSERT INTO limits__past_hit_limits
                    (id, guild_id, user_id, limit_ids, cause, notes)
                    VALUES ($1, $2, $3, $4, $5, $6)",
                    gen_random(16),
                    guild_id.to_string(),
                    user_id.to_string(),
                    &limit_ids,
                    &action_ids,
                    &vec![e.to_string()]
                )
                .execute(&data.pool)
                .await?;
            }
        }
    }

    Ok(())
}

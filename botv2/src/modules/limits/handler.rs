use super::core;
use crate::modules::limits::core::Limit;
use crate::Error;
use botox::crypto::gen_random;
use poise::serenity_prelude::{GuildId, UserId};
use std::collections::{HashSet, HashMap};

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

    sqlx::query!(
        "INSERT INTO limits__user_actions (action_id, guild_id, user_id, target, limit_type, action_data, created_at, limits_hit) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        gen_random(48),
        guild_id.to_string(),
        user_id.to_string(),
        target,
        limit.to_string(),
        action_data,
        sqlx::types::chrono::Utc::now(),
        &[],
    )
    .execute(&data.pool)
    .await?;

    let mut hit_limits = Vec::new();

    for (_limit_id, guild_limit) in guild_limits.into_iter() {
        // Check the limit type and user_id and guild to see if it is in the cache
        let infringing_actions = sqlx::query!(
            "select action_id from limits__user_actions where guild_id = $1 and user_id=  $2 and limit_type = $3 and created_at + make_interval(secs => $4) > now()",
            guild_id.to_string(),
            user_id.to_string(),
            limit.to_string(),
            guild_limit.limit_time as f64,
        )
        .fetch_all(&data.pool)
        .await?;

        if infringing_actions.len() >= guild_limit.limit_per as usize {
            hit_limits.push((
                infringing_actions
                    .into_iter()
                    .map(|v| v.action_id)
                    .collect::<Vec<String>>(),
                guild_limit,
            ));
        }
    }

    if !hit_limits.is_empty() && crate::silverpelt::module_config::is_module_enabled(&data.pool, guild_id, "auditlogs").await? {
        crate::modules::punishments::core::trigger_punishment(ctx, guild_id, user_id, HashSet::new()).await?;
    }

    Ok(())
}

use log::{error, info, warn};
use poise::serenity_prelude::{GuildId, UserId};
use sqlx::PgPool;
use std::collections::HashMap;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

use super::core;
use crate::modules::limits::core::{Limit, UserAction};
use crate::{impls::cache::CacheHttpImpl, Error};

// Returns true if the same user+target combo has appeared in the time interval user_target_repeat_rate
// TODO: This function needs to be rewritten as limits are handled in-memory
/**pub async fn ignore_handling(pool: &PgPool, ha: &HandleModAction) -> Result<bool, Error> {
    let repeat_rate = {
        let user_target_settings = core::GuildUserTargetSettings::from_guild(pool, ha.guild_id).await?;

        if user_target_settings.is_empty() {
            ha.limit.default_user_target_repeat_rate()
        } else {
            user_target_settings
                .iter()
                .find(|a| a.limit_type == ha.limit)
                .map(|a| a.user_target_repeat_rate)
                .unwrap_or_else(|| ha.limit.default_user_target_repeat_rate())
        }
    };

    // Get the last time the same user-target combo was handled
    let last_time_rec = sqlx::query!(
        "
            SELECT created_at FROM limits__user_actions
            WHERE guild_id = $1
            AND user_id = $2
            AND target = $3
            ORDER BY created_at DESC
            LIMIT 1
        ",
        ha.guild_id.to_string(),
        ha.user_id.to_string(),
        ha.target.to_string()
    )
    .fetch_one(pool)
    .await?;

    // Check if the last time was within the repeat rate
    if (sqlx::types::chrono::Utc::now() - last_time_rec.created_at).num_seconds() < repeat_rate {
        return Ok(true); // Ignore
    }

    Ok(false) // Don't ignore
}**/

pub struct HandleModAction {
    /// Guild ID
    pub guild_id: GuildId,
    /// User ID
    pub user_id: UserId,
    /// Limit to handle for the User ID in question
    pub limit: core::UserLimitTypes,
    /// Target of the action
    pub target: String,
    /// Extra data for the action
    pub action_data: serde_json::Value,
}

pub async fn handle_mod_action(
    pool: &PgPool,
    cache: &Surreal<Client>,
    cache_http: &CacheHttpImpl,
    ha: &HandleModAction,
) -> Result<(), Error> {
    let guild_id = ha.guild_id;
    let limit = ha.limit;
    let user_id = ha.user_id;
    let target = ha.target.clone();
    let action_data = &ha.action_data;
    // Check limits cache
    let guild_limits: HashMap<String, Limit> = Limit::fetch(cache, pool, guild_id)
        .await?
        .into_iter()
        .filter(|a| a.limit_type == limit)
        .map(|a| (a.limit_id.clone(), a))
        .collect();

    if guild_limits.is_empty() {
        // No limits for this guild
        return Ok(());
    }
    let _ = cache
        .create::<Vec<UserAction>>("user_actions")
        .content(UserAction {
            action_id: crate::impls::crypto::gen_random(48),
            guild_id,
            user_id,
            target: target.clone(),
            limit_type: limit,
            action_data: action_data.clone(),
            created_at: sqlx::types::chrono::Utc::now(),
            limits_hit: Vec::new(),
        })
        .await?;
    for (limit_id, guild_limit) in guild_limits.into_iter() {
        // Check the limit type and user_id and guild to see if it is in the cache
        let mut query = cache.query("select * from user_actions where guild_id=type::string($guild_id) and user_id=type::string($user_id) and limit_type=type::string($limit_type)")
            .bind(("guild_id", guild_id))
            .bind(("user_id", user_id))
            .bind(("limit_type", limit))
            .await?;
        let response: Vec<UserAction> = query.take(0)?;
        if response.len() as i32 >= guild_limit.limit_per {
            // Cut off the latest limit_per(number) of actions from response
            let mut actions = response.clone();
            actions.sort_by(|a, b| a.created_at.cmp(&b.created_at));
            actions.reverse();
            actions.truncate(guild_limit.limit_per as usize + 1);
            // Now check if actions are done within the time limit
            let last_time = actions[0].created_at;
            let mut action_ids = Vec::new();
            for action in actions {
                if (last_time - action.created_at).num_seconds() < guild_limit.limit_time {
                    action_ids.push(action.action_id.clone());
                }
            }
            if action_ids.len() as i32 > guild_limit.limit_per {
                // remove from cache here
                {
                    let mut tx: sqlx::Transaction<'_, sqlx::Postgres> = pool.begin().await?;

                    info!("Hit limit: limit={:?}", limit);
                    // Remove Cache
                    let _ = cache
                            .query("delete user_actions where guild_id=type::string($guild_id) and user_id=type::string($user_id) and limit_type=type::string($limit_type) return none")
                            .bind(("guild_id", guild_id))
                            .bind(("user_id", user_id))
                            .bind(("limit_type", limit))
                        .await?;
                    // Add UserActions to db Here.
                    sqlx::query!(
                        "
            INSERT INTO limits__user_actions
            (action_id, guild_id, user_id, target, limit_type, action_data, limits_hit)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
        ",
                        crate::impls::crypto::gen_random(48),
                        guild_id.to_string(),
                        user_id.to_string(),
                        target,
                        limit.to_string(),
                        action_data,
                        &action_ids
                    )
                    .execute(&mut *tx)
                    .await?;
                    // Immediately handle the limit
                    let cur_uid = cache_http.cache.current_user().id;
                    let can_mod = {
                        let guild = cache_http.cache.guild(guild_id).ok_or("Guild not found")?;

                        guild.greater_member_hierarchy(cache_http.cache.clone(), cur_uid, user_id)
                    }
                    .unwrap_or(cur_uid);
                    if can_mod == cur_uid {
                        info!("Moderating user");
                        match guild_limit.limit_action {
                            core::UserLimitActions::RemoveAllRoles => {
                                // Get all user roles
                                if let Ok(member) = guild_id.member(cache_http, user_id).await {
                                    let roles = member.roles.clone();
                                    for role in roles.iter() {
                                        if let Err(e) =
                                            member.remove_role(&cache_http.http, *role).await
                                        {
                                            error!("Failed to remove role: {}", e);
                                        }
                                    }
                                }
                            }
                            core::UserLimitActions::KickUser => {
                                if let Err(e) = guild_id.kick(&cache_http.http, user_id).await {
                                    error!("Failed to kick user: {}", e);
                                }
                            }
                            core::UserLimitActions::BanUser => {
                                if let Err(e) = guild_id.ban(&cache_http.http, user_id, 0).await {
                                    error!("Failed to kick user: {}", e);
                                }
                            }
                        }
                    } else {
                        warn!(
                            "Cannot moderate user, not enough permissions: {}, {}",
                            can_mod, cur_uid
                        );

                        sqlx::query!(
                            "
                            INSERT INTO limits__past_hit_limits
                            (id, guild_id, user_id, limit_id, cause, notes)
                            VALUES ($1, $2, $3, $4, $5, $6)",
                            crate::impls::crypto::gen_random(16),
                            guild_id.to_string(),
                            user_id.to_string(),
                            limit_id,
                            &action_ids,
                            &vec!["Not enough permissions to moderate user".to_string()]
                        )
                        .execute(&mut *tx)
                        .await?;

                        return Ok(());
                    }

                    sqlx::query!(
                        "
                        INSERT INTO limits__past_hit_limits
                        (id, guild_id, user_id, limit_id, cause)
                        VALUES ($1, $2, $3, $4, $5)",
                        crate::impls::crypto::gen_random(16),
                        guild_id.to_string(),
                        user_id.to_string(),
                        limit_id,
                        &action_ids
                    )
                    .execute(&mut *tx)
                    .await?;
                    return Ok(());
                }
            }
        } else {
            // No Limits hit.
            continue;
        }
    }
    Ok(())
}

use log::{error, info, warn};
use poise::serenity_prelude::{GuildId, UserId};
use botox::crypto::gen_random;
use sqlx::PgPool;
use std::collections::HashMap;
use botox::cache::CacheHttpImpl;
use super::core;
use crate::modules::limits::core::Limit;
use crate::Error;

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
    pool: &PgPool,
    cache_http: &CacheHttpImpl,
    ha: &HandleModAction,
) -> Result<(), Error> {
    let guild_id = ha.guild_id;
    let limit = ha.limit;
    let user_id = ha.user_id;
    let target = ha.target.clone();
    let action_data = &ha.action_data;

    // Check limits cache
    let guild_limits: HashMap<String, Limit> = Limit::guild(pool, guild_id)
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
        "INSERT INTO user_actions (action_id, guild_id, user_id, target, limit_type, action_data, created_at, limits_hit) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        gen_random(48),
        guild_id.to_string(),
        user_id.to_string(),
        target,
        limit.to_string(),
        action_data,
        sqlx::types::chrono::Utc::now(),
        &[],
    )
    .execute(pool)
    .await?;

    let mut hit_limits = Vec::new();

    for (_limit_id, guild_limit) in guild_limits.into_iter() {
        // Check the limit type and user_id and guild to see if it is in the cache
        let infringing_actions = sqlx::query!(
            "select action_id from user_actions where guild_id = $1 and user_id=  $2 and limit_type = $3 and created_at + make_interval(secs => $4) > now()",
            guild_id.to_string(),
            user_id.to_string(),
            limit.to_string(),
            guild_limit.limit_time as f64,
        )
        .fetch_all(pool)
        .await?;
        
        if infringing_actions.len() >= guild_limit.limit_per as usize {
            hit_limits.push((infringing_actions.into_iter().map(|v| v.action_id).collect::<Vec<String>>(), guild_limit));
        }
    }

    if !hit_limits.is_empty() {
        // Immediately handle the limit
        let cur_uid = cache_http.cache.current_user().id;
        let can_mod = {
            let guild = cache_http.cache.guild(guild_id).ok_or("Guild not found")?;

            guild.greater_member_hierarchy(cur_uid, user_id)
        }
        .unwrap_or(cur_uid);

        if can_mod == cur_uid {
            info!("Moderating user");

            for (action_ids, limit) in hit_limits {
                match limit.limit_action {
                    core::UserLimitActions::RemoveAllRoles => {
                        // Get all user roles
                        if let Ok(member) = guild_id.member(cache_http, user_id).await {
                            let roles = member.roles.clone();
                            
                            let mut errors = Vec::new();
                            for role in roles.iter() {
                                if let Err(e) =
                                    member.remove_role(&cache_http.http, *role, Some("Removing roles due to preconfigured limits")).await
                                {
                                    errors.push(format!("Failed to remove role: {}", e));
                                }
                            }

                            sqlx::query!(
                                "
                                INSERT INTO limits__past_hit_limits
                                (id, guild_id, user_id, limit_id, cause, notes)
                                VALUES ($1, $2, $3, $4, $5, $6)",
                                gen_random(16),
                                guild_id.to_string(),
                                user_id.to_string(),
                                limit.limit_id,
                                &action_ids,
                                &errors
                            )
                            .execute(pool)
                            .await?;                
                        }
                    }
                    core::UserLimitActions::KickUser => {
                        if let Err(e) = guild_id.kick(&cache_http.http, user_id, Some("Kicking user due to preconfigured limits")).await {
                            error!("Failed to kick user: {}", e);

                            sqlx::query!(
                                "
                                INSERT INTO limits__past_hit_limits
                                (id, guild_id, user_id, limit_id, cause, notes)
                                VALUES ($1, $2, $3, $4, $5, $6)",
                                gen_random(16),
                                guild_id.to_string(),
                                user_id.to_string(),
                                limit.limit_id,
                                &action_ids,
                                &vec![format!("Failed to kick user: {}", e)]
                            )
                            .execute(pool)
                            .await?;                
                        } else {
                            sqlx::query!(
                                "
                                INSERT INTO limits__past_hit_limits
                                (id, guild_id, user_id, limit_id, cause)
                                VALUES ($1, $2, $3, $4, $5)",
                                gen_random(16),
                                guild_id.to_string(),
                                user_id.to_string(),
                                limit.limit_id,
                                &action_ids,
                            )
                            .execute(pool)
                            .await?;
                        }

                        return Ok(());
                    }
                    core::UserLimitActions::BanUser => {
                        if let Err(e) = guild_id.ban(&cache_http.http, user_id, 0, Some("Banning user due to preconfigured limits")).await {
                            error!("Failed to kick user: {}", e);

                            sqlx::query!(
                                "
                                INSERT INTO limits__past_hit_limits
                                (id, guild_id, user_id, limit_id, cause, notes)
                                VALUES ($1, $2, $3, $4, $5, $6)",
                                gen_random(16),
                                guild_id.to_string(),
                                user_id.to_string(),
                                limit.limit_id,
                                &action_ids,
                                &vec![format!("Failed to ban user: {}", e)]
                            )
                            .execute(pool)
                            .await?;
                        } else {
                            sqlx::query!(
                                "
                                INSERT INTO limits__past_hit_limits
                                (id, guild_id, user_id, limit_id, cause)
                                VALUES ($1, $2, $3, $4, $5)",
                                gen_random(16),
                                guild_id.to_string(),
                                user_id.to_string(),
                                limit.limit_id,
                                &action_ids,
                            )
                            .execute(pool)
                            .await?;
                        }

                        return Ok(());
                    }
                }
            }
        } else {
            warn!(
                "Cannot moderate user, not enough permissions: {}, {}",
                can_mod, cur_uid
            );

            return Ok(());
        }

        return Ok(());
    }

    Ok(())
}

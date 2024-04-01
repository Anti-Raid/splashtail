use log::{error, info, warn};
use poise::serenity_prelude::{GuildId, UserId};
use splashcore_rs::crypto::gen_random;
use sqlx::PgPool;
use std::collections::HashMap;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use bothelpers::cache::CacheHttpImpl;
use super::core;
use crate::modules::limits::core::{Limit, UserAction};
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
            action_id: gen_random(48),
            guild_id,
            user_id,
            target,
            limit_type: limit,
            action_data: action_data.clone(),
            created_at: sqlx::types::chrono::Utc::now(),
            limits_hit: Vec::new(),
        })
        .await?;

    let mut hit_limits = Vec::new();

    for (_limit_id, guild_limit) in guild_limits.into_iter() {
        // Check the limit type and user_id and guild to see if it is in the cache
        let mut query = cache.query("select action_id from user_actions where guild_id=type::string($guild_id) and user_id=type::string($user_id) and limit_type=type::string($limit_type) and time::now() - created_at < duration::from::secs($limit_time)")
            .bind(("guild_id", guild_id))
            .bind(("user_id", user_id))
            .bind(("limit_type", limit))
            .bind(("limit_time", guild_limit.limit_time))
            .await?;

        let action_ids: Vec<String> = query.take(0)?;

        if action_ids.len() >= guild_limit.limit_per as usize {
            hit_limits.push((action_ids, guild_limit));
        }
    }

    if !hit_limits.is_empty() {
        // Immediately handle the limit
        let cur_uid = cache_http.cache.current_user().id;
        let can_mod = {
            let guild = cache_http.cache.guild(guild_id).ok_or("Guild not found")?;

            guild.greater_member_hierarchy(&cache_http.cache, cur_uid, user_id)
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
                                    member.remove_role(&cache_http.http, *role).await
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
                        if let Err(e) = guild_id.kick(&cache_http.http, user_id).await {
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
                        if let Err(e) = guild_id.ban(&cache_http.http, user_id, 0).await {
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

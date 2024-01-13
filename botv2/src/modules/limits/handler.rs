use log::{error, info, warn, debug};
use poise::serenity_prelude::{GuildId, UserId};
use sqlx::PgPool;

use crate::{impls::cache::CacheHttpImpl, Error};
use super::core;

static PRECHECK_CACHE: once_cell::sync::Lazy<moka::future::Cache<String, bool>> = once_cell::sync::Lazy::new(|| {
    moka::future::Cache::builder()
    .time_to_live(std::time::Duration::from_secs(60))
    .build()
});

pub async fn precheck_guild(pool: &PgPool, guild_id: GuildId, limit: &super::core::UserLimitTypes) -> Result<(), Error> {
    let limit_str = limit.to_string();
    let precache_key = guild_id.to_string() + &limit_str;
    // Look for guild
    if let Some(found) = PRECHECK_CACHE.get(&precache_key).await {
        if !found {
            return Err("Guild has no limits [cached]".into());
        }

        return Ok(());
    }

    let guild = sqlx::query!(
        "SELECT COUNT(*) FROM limits__guild_limits WHERE guild_id = $1 AND limit_type = $2
    ",
        guild_id.to_string(),
        limit_str
    )
    .fetch_one(pool)
    .await?;

    if guild.count.unwrap_or_default() == 0 {
        // Guild not found
        PRECHECK_CACHE.insert(precache_key, false).await;
        return Err("Guild has no limits".into());
    }

    PRECHECK_CACHE.insert(precache_key, true).await;

    Ok(())
}

/// Returns true if the same user+target combo has appeared in the time interval user_target_repeat_rate
pub async fn ignore_handling(pool: &PgPool, ha: &HandleModAction) -> Result<bool, Error> {
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
}

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
    cache_http: &CacheHttpImpl,
    ha: &HandleModAction,
) -> Result<(), Error> {
    let guild_id = ha.guild_id;
    let limit = &ha.limit;
    let user_id = ha.user_id;
    let target = ha.target.clone();
    let action_data = &ha.action_data;

    if let Err(e) = precheck_guild(pool, guild_id, &limit).await {
        debug!("Limits precheck failed: {}", e);
        return Ok(()) // Avoid spamming errors
    }

    if ignore_handling(pool, ha).await? {
        debug!("Ignoring handling [limit={}, user_id={}, target={}]", limit, user_id, target);
        return Ok(());
    }
    
    // SAFETY: Tx should be dropped if error occurs, so make a scope to seperate tx queries
    {
        let mut tx = pool.begin().await?;

        // Insert into limits__user_actions
        sqlx::query!(
            "
            INSERT INTO limits__user_actions 
            (action_id, guild_id, user_id, target, limit_type, action_data)
            VALUES ($1, $2, $3, $4, $5, $6)
        ",
            crate::impls::crypto::gen_random(48),
            guild_id.to_string(),
            user_id.to_string(),
            target,
            limit.to_string(),
            action_data
        )
        .execute(&mut tx)
        .await?;

        // Check if they hit any limits yet
        let hit = core::CurrentUserLimitsHit::hit(guild_id, pool).await?;

        for hit_limit in hit {
            // We have a hit limit for this user
            info!("Hit limit: {:?}", hit_limit);

            // Immediately handle the limit
            let cur_uid = cache_http.cache.current_user().id;
            let can_mod = {
                let guild = cache_http.cache.guild(guild_id).ok_or("Guild not found")?;

                guild.greater_member_hierarchy(cache_http.cache.clone(), cur_uid, user_id)
            }
            .unwrap_or(cur_uid);

            if can_mod == cur_uid {
                info!("Moderating user");
                match hit_limit.limit.limit_action {
                    core::UserLimitActions::RemoveAllRoles => {
                        // Get all user roles
                        if let Ok(member) = guild_id.member(cache_http, user_id).await {
                            let roles = member.roles.clone();
                            for role in roles.iter() {
                                if let Err(e) = member.remove_role(&cache_http.http, *role).await {
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
                    hit_limit.limit.limit_id,
                    &hit_limit
                        .cause
                        .iter()
                        .map(|a| a.action_id.clone())
                        .collect::<Vec<_>>(),
                    &vec!["Not enough permissions to moderate user".to_string()]
                )
                .execute(&mut tx)
                .await?;

                return Ok(());
            }

            for action in hit_limit.cause.iter() {
                sqlx::query!(
                    "
                UPDATE limits__user_actions
                SET limits_hit = array_append(limits_hit, $1)
                WHERE action_id = $2",
                    hit_limit.limit.limit_id,
                    action.action_id
                )
                .execute(&mut tx)
                .await?;
            }

            sqlx::query!(
                "
            INSERT INTO limits__past_hit_limits
            (id, guild_id, user_id, limit_id, cause)
            VALUES ($1, $2, $3, $4, $5)",
                crate::impls::crypto::gen_random(16),
                guild_id.to_string(),
                user_id.to_string(),
                hit_limit.limit.limit_id,
                &hit_limit
                    .cause
                    .iter()
                    .map(|a| a.action_id.clone())
                    .collect::<Vec<_>>()
            )
            .execute(&mut tx)
            .await?;
        }

        tx.commit().await?;
    }

    Ok(())
}

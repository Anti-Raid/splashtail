use log::{error, info, warn};
use poise::serenity_prelude::{GuildId, UserId};
use sqlx::PgPool;

use crate::{impls::cache::CacheHttpImpl, Error};
use super::core;

pub async fn handle_mod_action(
    guild_id: GuildId,
    user_id: UserId,
    pool: &PgPool,
    cache_http: &CacheHttpImpl,
    action: core::UserLimitTypes,
    action_target: String,
) -> Result<(), Error> {
    // SAFETY: Tx should be dropped if error occurs, so make a scope to seperate tx queries
    {
        let mut tx = pool.begin().await?;

        // Look for guild
        let guild = sqlx::query!(
            "SELECT COUNT(*) FROM limits WHERE guild_id = $1
        ",
            guild_id.to_string()
        )
        .fetch_one(&mut tx)
        .await?;

        if guild.count.unwrap_or_default() == 0 {
            // Guild not found
            error!("Guild has no limits: {}", guild_id);
            return Ok(());
        }

        // Insert into user_actions
        sqlx::query!(
            "
            INSERT INTO user_actions (action_id, guild_id, user_id, limit_type, action_target)
            VALUES ($1, $2, $3, $4, $5)
        ",
            crate::impls::crypto::gen_random(48),
            guild_id.to_string(),
            user_id.to_string(),
            action.to_string(),
            action_target
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
                                if let Err(e) = member.remove_role(&cache_http.http, role).await {
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
                INSERT INTO past_hit_limits
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
                UPDATE user_actions
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
            INSERT INTO past_hit_limits
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

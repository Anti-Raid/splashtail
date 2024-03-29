use sqlx::PgPool;

const MAX_CONCURRENT_UNBANS: usize = 3;

#[derive(Debug)]
enum UnbanError {
    SerenityError(serenity::Error),
    SqlxError(sqlx::Error),
}

pub async fn temp_punishment(
    pool: &PgPool,
    ctx: &serenity::client::Context,
) -> Result<(), crate::Error> {
    let temp_punishments = sqlx::query!(
        "SELECT id, guild_id, user_id, moderator, action, stings, reason, extract(epoch from duration) AS duration FROM moderation__actions WHERE handled = false AND duration IS NOT NULL"
    )
    .fetch_all(pool)
    .await?;

    let mut set = tokio::task::JoinSet::new();

    for punishment in temp_punishments {
        // Supported punishments for temp are: 'ban'
        match punishment.action.as_str() {
            "ban" => {},
            _ => continue
        }

        let Ok(guild_id) = punishment.guild_id.parse::<serenity::all::GuildId>() else {
            continue
        };

        // Ensure shard id
        let shard_id = serenity::utils::shard_id(guild_id, crate::ipc::argparse::MEWLD_ARGS.shard_count);

        if !crate::ipc::argparse::MEWLD_ARGS.shards.contains(&shard_id) {
            continue
        }

        let Ok(user_id) = punishment.user_id.parse::<serenity::all::UserId>() else {
            continue
        };

        // If over MAX_CONCURRENT_UNBANS bans ongoing, wait for one to finish
        if set.len() >= MAX_CONCURRENT_UNBANS {
            if let Some(res) = set.join_next().await {
                match res {
                    Err(e) => log::error!("Error while running unban [join]: {}", e),
                    Ok(Ok(_)) => {},
                    Ok(Err(UnbanError::SerenityError(e))) => log::error!("Error while running unban [discord]: {}", e),
                    Ok(Err(UnbanError::SqlxError(e))) => log::error!("Error while running unban [sqlx]: {}", e),
                }
            }
        }

        let ctx = ctx.clone();
        let pool = pool.clone();
        set.spawn(async move {
            let reason = if let Some(reason) = punishment.reason {
                format!("Revert expired ban with reason={}, stings={}", reason, punishment.stings)
            } else {
                format!("Revert expired ban with stings={}", punishment.stings)
            };

            match ctx.http.remove_ban(
                guild_id, 
                user_id,
                Some(reason.as_str())
            )
            .await
            .map_err(UnbanError::SerenityError) {
                Ok(_) => {
                    sqlx::query!(
                        "UPDATE moderation__actions SET handled = true WHERE id = $1",
                        punishment.id
                    )
                    .execute(&pool)
                    .await
                    .map_err(UnbanError::SqlxError)
                },
                Err(e) => Err(e)
            }
        });
    }

    // Wait for all tasks to finish
    while let Some(res) = set.join_next().await {
        match res {
            Err(e) => log::error!("Error while running unban [join]: {}", e),
            Ok(Ok(_)) => {},
            Ok(Err(UnbanError::SerenityError(e))) => log::error!("Error while running unban [discord]: {}", e),
            Ok(Err(UnbanError::SqlxError(e))) => log::error!("Error while running unban [sqlx]: {}", e),
        }
    }

    Ok(())
}

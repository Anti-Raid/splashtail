use sqlx::PgPool;

const MAX_CONCURRENT_UNBANS: usize = 3;

#[derive(Debug)]
enum UnbanError {
    Serenity(serenity::Error),
    Sqlx(sqlx::Error),
    Generic(String),
}

pub async fn temp_punishment(
    pool: &PgPool,
    ctx: &serenity::client::Context,
) -> Result<(), crate::Error> {
    let temp_punishments = sqlx::query!(
        "SELECT id, guild_id, user_id, moderator, action, stings, reason FROM moderation__actions WHERE handled = false AND duration IS NOT NULL AND duration + created_at < NOW()",
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
                    Ok(Err(UnbanError::Serenity(e))) => log::error!("Error while running unban [discord]: {}", e),
                    Ok(Err(UnbanError::Sqlx(e))) => log::error!("Error while running unban [sqlx]: {}", e),
                    Ok(Err(UnbanError::Generic(e))) => log::error!("Error while running unban [generic]: {}", e),
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

            let result = match punishment.action.as_str() {
                "ban" => {
                    ctx.http.remove_ban(
                        guild_id, 
                        user_id,
                        Some(reason.as_str())
                    )
                    .await
                    .map_err(UnbanError::Serenity)
                },
                _ => Err(UnbanError::Generic(format!("Unsupported action: {}", punishment.action)))
            };

            match result {
                Ok(_) => {
                    sqlx::query!(
                        "UPDATE moderation__actions SET handled = true WHERE guild_id = $1 AND user_id = $2 AND action = $3",
                        guild_id.to_string(),
                        user_id.to_string(),
                        punishment.action
                    )
                    .execute(&pool)
                    .await
                    .map_err(UnbanError::Sqlx)
                },
                Err(e) => {
                    if let UnbanError::Serenity(ref e) = e {
                        // Check if we have a http error
                        match e {
                            serenity::Error::Http(serenity::all::HttpError::UnsuccessfulRequest(e)) => {
                                if [serenity::http::StatusCode::BAD_REQUEST, serenity::http::StatusCode::FORBIDDEN, serenity::http::StatusCode::NOT_FOUND].contains(&e.status_code) {
                                    // Ban already removed
                                    sqlx::query!(
                                        "UPDATE moderation__actions SET handled = true, handle_errors = $1 WHERE guild_id = $2 AND user_id = $3 AND action = $4",
                                        format!("{}: {}", e.status_code, e.error.message),
                                        guild_id.to_string(),
                                        user_id.to_string(),
                                        punishment.action
                                    )
                                    .execute(&pool)
                                    .await
                                    .map_err(UnbanError::Sqlx)?;
                                }
                            },
                            serenity::Error::Model(e) => {
                                // Bot doesn't have permissions to unban
                                sqlx::query!(
                                    "UPDATE moderation__actions SET handled = true, handle_errors = $1 WHERE guild_id = $2 AND user_id = $3 AND action = $4",
                                    format!("{:#?}", e),
                                    guild_id.to_string(),
                                    user_id.to_string(),
                                    punishment.action
                                )
                                .execute(&pool)
                                .await
                                .map_err(UnbanError::Sqlx)?;
                            },
                            _ => {}
                        }
                    }
                    
                    Err(e)
                }
            }
        });
    }

    // Wait for all tasks to finish
    while let Some(res) = set.join_next().await {
        match res {
            Err(e) => log::error!("Error while running unban [join]: {}", e),
            Ok(Ok(_)) => {},
            Ok(Err(UnbanError::Serenity(e))) => log::error!("Error while running unban [discord]: {}", e),
            Ok(Err(UnbanError::Sqlx(e))) => log::error!("Error while running unban [sqlx]: {}", e),
            Ok(Err(UnbanError::Generic(e))) => log::error!("Error while running unban [generic]: {}", e),
        }
    }

    Ok(())
}

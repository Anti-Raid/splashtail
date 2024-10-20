const MAX_CONCURRENT: usize = 7;

#[derive(Debug)]
enum EventError {
    Serenity(serenity::Error),
    Generic(String),
    BotNotInGuild(sqlx::types::Uuid),
}

impl std::fmt::Display for EventError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventError::Serenity(e) => write!(f, "Serenity error: {}", e),
            EventError::Generic(e) => write!(f, "Generic error: {}", e),
            EventError::BotNotInGuild(id) => write!(f, "Bot not in guild: {}", id),
        }
    }
}

pub async fn temporary_punishment_task(
    ctx: &serenity::all::client::Context,
) -> Result<(), silverpelt::Error> {
    let data = ctx.data::<silverpelt::data::Data>();
    let pool = &data.pool;

    let punishments = silverpelt::punishments::GuildPunishment::get_expired(pool).await?;

    let mut set = tokio::task::JoinSet::new();

    let shard_count = data.props.shard_count().await?.try_into()?;
    let shards = data.props.shards().await?;

    for punishment in punishments {
        // Ensure shard id
        let shard_id = serenity::utils::shard_id(punishment.guild_id, shard_count);

        if !shards.contains(&shard_id) {
            continue;
        }

        // Ensure temporary punishments module is enabled
        if !silverpelt::module_config::is_module_enabled(
            &data.silverpelt_cache,
            pool,
            punishment.guild_id,
            "temporary_punishments",
        )
        .await?
        {
            continue;
        }

        // If over get_max_concurrent() tasks ongoing, wait for one to finish
        if set.len() >= MAX_CONCURRENT {
            if let Some(res) = set.join_next().await {
                match res {
                    Err(e) => log::error!("Error while reverting punishment [join]: {}", e),
                    Ok(Ok(_)) => {}
                    Ok(Err(EventError::Serenity(e))) => {
                        log::error!("Error while reverting punishment [discord]: {}", e)
                    }
                    Ok(Err(EventError::BotNotInGuild(id))) => {
                        // Void the sting
                        match sqlx::query!(
                            "UPDATE punishments SET duration = NULL, handle_log = $1 WHERE id = $2",
                            serde_json::json!({
                                "error": "Bot not in guild",
                            }),
                            id
                        )
                        .execute(pool)
                        .await
                        .map_err(|e| EventError::Generic(e.to_string()))
                        {
                            Err(e) => {
                                log::error!(
                                    "Error while reverting punishment [bot not in guild]: {}",
                                    e
                                );
                            }
                            _ => {}
                        };
                    }
                    Ok(Err(EventError::Generic(e))) => {
                        log::error!("Error while reverting punishment [generic]: {}", e)
                    }
                }
            }
        }

        let cache_http = botox::cache::CacheHttpImpl::from_ctx(ctx);
        let reqwest = data.reqwest.clone();
        let data = data.clone();

        let target_user_id = match punishment.target {
            silverpelt::punishments::PunishmentTarget::User(user_id) => user_id,
            _ => continue,
        };

        set.spawn(async move {
            let bot_id = cache_http.cache.current_user().id;

            let mut current_user = match sandwich_driver::member_in_guild(
                &cache_http,
                &reqwest,
                punishment.guild_id,
                bot_id,
            )
            .await
            .map_err(|x| EventError::Generic(x.to_string()))?
            {
                Some(user) => user,
                None => {
                    return Err(EventError::BotNotInGuild(punishment.id).into());
                }
            };

            let permissions = current_user
                .permissions(&cache_http.cache)
                .map_err(EventError::Serenity)?;

            if !permissions.ban_members() {
                // Bot doesn't have permissions to unban
                return Err(EventError::Generic(
                    "Bot doesn't have permissions to unban".into(),
                ));
            }

            let reason = format!(
                "Revert expired ban with reason={}, duration={:#?}",
                punishment.reason, punishment.duration
            );

            let punishment_actions = silverpelt::punishments::get_punishment_actions_for_guild(
                punishment.guild_id,
                &data,
            )
            .await
            .map_err(|e| EventError::Generic(e.to_string()))?;

            let cpa_revert = silverpelt::punishments::from_punishment_action_string(
                &punishment_actions,
                &punishment.punishment,
            )
            .map_err(|e| EventError::Generic(e.to_string()))?;

            cpa_revert
                .revert(
                    &silverpelt::punishments::PunishmentActionData {
                        cache_http: cache_http.clone(),
                        pool: data.pool.clone(),
                        reqwest: data.reqwest.clone(),
                        object_store: data.object_store.clone(),
                    },
                    target_user_id,
                    &mut current_user,
                    reason,
                )
                .await
                .map_err(|e| EventError::Generic(e.to_string()))?;

            Ok(())
        });
    }

    // Wait for all tasks to finish
    while let Some(res) = set.join_next().await {
        match res {
            Err(e) => log::error!("Error while running unban [join]: {}", e),
            Ok(Ok(_)) => {}
            Ok(Err(EventError::Serenity(e))) => {
                log::error!("Error while running unban [discord]: {}", e)
            }
            Ok(Err(EventError::BotNotInGuild(id))) => {
                match sqlx::query!(
                    "UPDATE punishments SET duration = NULL, handle_log = $1 WHERE id = $2",
                    serde_json::json!({
                        "error": "Bot not in guild",
                    }),
                    id
                )
                .execute(pool)
                .await
                .map_err(|e| EventError::Generic(e.to_string()))
                {
                    Err(e) => {
                        log::error!("Error while running unban [bot not in guild]: {}", e);
                    }
                    _ => {}
                };
            }
            Ok(Err(EventError::Generic(e))) => {
                /*sqlx::query!(
                    "UPDATE stings SET state = 'voided', handle_log = $1 WHERE id = $2",
                    serde_json::json!({
                        "error": e,
                    }),
                    sting.id
                )
                .execute(pool)
                .await
                .map_err(|e| EventError::Generic(e.to_string()))?;*/

                log::error!("Error while running unban [generic]: {}", e)
            }
        }
    }

    Ok(())
}

use std::sync::Arc;

const MAX_CONCURRENT: usize = 7;

#[derive(Debug)]
enum UnbanError {
    Serenity(serenity::Error),
    Generic(String),
}

async fn get_all_temp_punishments(
    ctx: &serenity::client::Context,
) -> Result<Vec<(Arc<super::source::Source>, Vec<super::source::Entry>)>, crate::Error> {
    let mut temp_punishments = Vec::new();

    for src in super::source::SOURCES.iter() {
        let source = src.value().clone();

        let entries = (source.fetch)(ctx).await?;

        temp_punishments.push((source, entries));
    }

    Ok(temp_punishments)
}

pub async fn temporary_punishment_task(
    ctx: &serenity::client::Context,
) -> Result<(), crate::Error> {
    let data = ctx.data::<crate::Data>();
    let pool = &data.pool;

    let temp_punishments = get_all_temp_punishments(ctx).await?;

    let mut set = tokio::task::JoinSet::new();

    let shard_count = data.props.shard_count().try_into()?;
    let shards = data.props.shards();

    for (source, punishments) in temp_punishments {
        for punishment in punishments {
            // Ensure shard id
            let shard_id = serenity::utils::shard_id(punishment.guild_id, shard_count);

            if !shards.contains(&shard_id) {
                continue;
            }

            // Ensure temporary punishments module is enabled
            if !crate::silverpelt::module_config::is_module_enabled(
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
                        Err(e) => log::error!("Error while running unban [join]: {}", e),
                        Ok(Ok(_)) => {}
                        Ok(Err(UnbanError::Serenity(e))) => {
                            log::error!("Error while running unban [discord]: {}", e)
                        }
                        Ok(Err(UnbanError::Generic(e))) => {
                            log::error!("Error while running unban [generic]: {}", e)
                        }
                    }
                }
            }

            let ctx = ctx.clone();
            let source = source.clone();
            set.spawn(async move {
                let bot_id = ctx.cache.current_user().id;
                let current_user = match punishment.guild_id.member(&ctx, bot_id).await {
                    Ok(user) => user,
                    Err(serenity::Error::Http(serenity::all::HttpError::UnsuccessfulRequest(
                        e,
                    ))) => {
                        if e.status_code == serenity::http::StatusCode::NOT_FOUND {
                            // Bot is not in the guild, mark as handled then return
                            (source.log_error)(
                                &ctx,
                                &punishment,
                                Some("Bot is not in the guild".into()),
                            )
                            .await
                            .map_err(|e| {
                                UnbanError::Generic(format!("Log error failure: {:#?}", e))
                            })?;
                            return Ok(());
                        }

                        return Err(UnbanError::Serenity(serenity::Error::Http(
                            serenity::all::HttpError::UnsuccessfulRequest(e),
                        )));
                    }
                    Err(e) => {
                        log::error!("Error while getting bot member: {}", e);
                        return Err(UnbanError::Serenity(e));
                    }
                };

                let permissions = current_user
                    .permissions(&ctx.cache)
                    .map_err(UnbanError::Serenity)?;

                if !permissions.ban_members() {
                    (source.log_error)(
                        &ctx,
                        &punishment,
                        Some("Bot doesn't have permissions to unban".into()),
                    )
                    .await
                    .map_err(|e| UnbanError::Generic(format!("Log error failure: {:#?}", e)))?;

                    return Ok(());
                }

                let reason = if let Some(ref reason) = punishment.reason {
                    format!(
                        "Revert expired ban with reason={}, stings={}, duration={:#?}",
                        reason, punishment.stings, punishment.duration
                    )
                } else {
                    format!(
                        "Revert expired ban with stings={}, duration={:#?}",
                        punishment.stings, punishment.duration
                    )
                };

                let result = match punishment.action {
                    super::source::Action::Ban => ctx
                        .http
                        .remove_ban(
                            punishment.guild_id,
                            punishment.user_id,
                            Some(reason.as_str()),
                        )
                        .await
                        .map_err(UnbanError::Serenity),
                    super::source::Action::RemoveAllRoles => punishment
                        .guild_id
                        .edit_member(
                            &ctx.http,
                            punishment.user_id,
                            serenity::all::EditMember::new()
                                .audit_log_reason(reason.as_str())
                                .roles(Vec::new()),
                        )
                        .await
                        .map_err(UnbanError::Serenity)
                        .map(|_| ()),
                };

                match result {
                    Ok(_) => {
                        (source.log_error)(&ctx, &punishment, None)
                            .await
                            .map_err(|e| {
                                UnbanError::Generic(format!("Log error failure: {:#?}", e))
                            })?;
                        Ok(())
                    }
                    Err(ue) => {
                        if let UnbanError::Serenity(ref e) = ue {
                            // Check if we have a http error
                            match e {
                                serenity::Error::Http(
                                    serenity::all::HttpError::UnsuccessfulRequest(e),
                                ) => {
                                    (source.log_error)(
                                        &ctx,
                                        &punishment,
                                        Some(format!("{}: {}", e.status_code, e.error.message)),
                                    )
                                    .await
                                    .map_err(|e| {
                                        UnbanError::Generic(format!("Log error failure: {:#?}", e))
                                    })?;
                                    Err(ue)
                                }
                                serenity::Error::Model(e) => {
                                    (source.log_error)(
                                        &ctx,
                                        &punishment,
                                        Some(format!("{:#?}", e)),
                                    )
                                    .await
                                    .map_err(|e| {
                                        UnbanError::Generic(format!("Log error failure: {:#?}", e))
                                    })?;
                                    Err(ue)
                                }
                                _ => Ok(()),
                            }
                        } else {
                            (source.log_error)(&ctx, &punishment, Some(format!("{:#?}", ue)))
                                .await
                                .map_err(|e| {
                                    UnbanError::Generic(format!("Log error failure: {:#?}", e))
                                })?;
                            Err(ue)
                        }
                    }
                }
            });
        }
    }

    // Wait for all tasks to finish
    while let Some(res) = set.join_next().await {
        match res {
            Err(e) => log::error!("Error while running unban [join]: {}", e),
            Ok(Ok(_)) => {}
            Ok(Err(UnbanError::Serenity(e))) => {
                log::error!("Error while running unban [discord]: {}", e)
            }
            Ok(Err(UnbanError::Generic(e))) => {
                log::error!("Error while running unban [generic]: {}", e)
            }
        }
    }

    Ok(())
}

use silverpelt::sting_sources;
use std::sync::Arc;

const MAX_CONCURRENT: usize = 7;

#[derive(Debug)]
enum EventError {
    Serenity(serenity::Error),
    Generic(String),
}

async fn get_all_temp_punishments(
    data: &sting_sources::StingSourceData,
) -> Result<
    Vec<(
        Arc<dyn sting_sources::StingSource>,
        Vec<sting_sources::FullStingEntry>,
    )>,
    silverpelt::Error,
> {
    let mut temp_punishments = Vec::new();

    for (_, module) in data.silverpelt_cache.module_cache.iter() {
        for src in module.sting_sources.iter() {
            // If the module doesn't support durations/expirations, skip
            let flags = src.flags();
            if !flags.supports_duration() || !flags.supports_actions() {
                continue;
            }

            let source = src.clone();

            let entries = source
                .fetch(
                    data,
                    sting_sources::StingFetchFilters {
                        user_id: None,
                        guild_id: None,
                        has_duration: Some(true),
                        state: Some(sting_sources::StingState::Active),
                        ..Default::default()
                    },
                )
                .await?;

            temp_punishments.push((source, entries));
        }
    }

    Ok(temp_punishments)
}

pub async fn temporary_punishment_task(
    ctx: &serenity::all::client::Context,
) -> Result<(), silverpelt::Error> {
    let data = ctx.data::<silverpelt::data::Data>();
    let pool = &data.pool;

    // NOTE: While silverpelt does provide a from_ctx method, it leads to a second Any downcast
    let source_data = Arc::new(sting_sources::StingSourceData {
        pool: pool.clone(),
        reqwest: data.reqwest.clone(),
        cache_http: botox::cache::CacheHttpImpl::from_ctx(ctx),
        silverpelt_cache: data.silverpelt_cache.clone(),
    });

    let temp_punishments = get_all_temp_punishments(&source_data).await?;

    let mut set = tokio::task::JoinSet::new();

    let shard_count = data.props.shard_count().await?.try_into()?;
    let shards = data.props.shards().await?;

    for (source, punishments) in temp_punishments {
        for punishment in punishments {
            // Ensure shard id
            let shard_id = serenity::utils::shard_id(punishment.entry.guild_id, shard_count);

            if !shards.contains(&shard_id) {
                continue;
            }

            // Ensure temporary punishments module is enabled
            if !silverpelt::module_config::is_module_enabled(
                &source_data.silverpelt_cache,
                pool,
                punishment.entry.guild_id,
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
                        Ok(Err(EventError::Serenity(e))) => {
                            log::error!("Error while running unban [discord]: {}", e)
                        }
                        Ok(Err(EventError::Generic(e))) => {
                            log::error!("Error while running unban [generic]: {}", e)
                        }
                    }
                }
            }

            let source = source.clone();
            let source_data = source_data.clone();
            set.spawn(async move {
                let bot_id = source_data.cache_http.cache.current_user().id;

                let current_user = match proxy_support::member_in_guild(
                    &source_data.cache_http,
                    &source_data.reqwest,
                    punishment.entry.guild_id,
                    bot_id,
                )
                .await
                .map_err(|x| EventError::Generic(x.to_string()))?
                {
                    Some(user) => user,
                    None => {
                        // Bot is not in the guild, update the sting entry
                        source
                            .update_sting_entry(
                                &source_data,
                                punishment.id,
                                sting_sources::UpdateStingEntry {
                                    state: Some(sting_sources::StingState::Voided),
                                    void_reason: Some("Bot is not in the guild".into()),
                                    ..Default::default()
                                },
                            )
                            .await
                            .map_err(|e| {
                                EventError::Generic(format!("Log error failure: {:#?}", e))
                            })?;
                        return Ok(());
                    }
                };

                let permissions = current_user
                    .permissions(&source_data.cache_http.cache)
                    .map_err(EventError::Serenity)?;

                if !permissions.ban_members() {
                    source
                        .update_sting_entry(
                            &source_data,
                            punishment.id,
                            sting_sources::UpdateStingEntry {
                                state: Some(sting_sources::StingState::Voided),
                                void_reason: Some("Bot doesn't have permissions to unban".into()),
                                ..Default::default()
                            },
                        )
                        .await
                        .map_err(|e| EventError::Generic(format!("Log error failure: {:#?}", e)))?;
                    return Ok(());
                }

                let reason = if let Some(ref reason) = punishment.entry.reason {
                    format!(
                        "Revert expired ban with reason={}, stings={}, duration={:#?}",
                        reason, punishment.entry.stings, punishment.entry.duration
                    )
                } else {
                    format!(
                        "Revert expired ban with stings={}, duration={:#?}",
                        punishment.entry.stings, punishment.entry.duration
                    )
                };

                let result = match punishment.entry.action {
                    sting_sources::Action::None => {
                        source
                            .update_sting_entry(
                                &source_data,
                                punishment.id,
                                sting_sources::UpdateStingEntry {
                                    state: Some(sting_sources::StingState::Handled),
                                    void_reason: Some(
                                        "Action is None, not doing anything...".into(),
                                    ),
                                    ..Default::default()
                                },
                            )
                            .await
                            .map_err(|e| {
                                EventError::Generic(format!("Log error failure: {:#?}", e))
                            })?;
                        return Ok(());
                    }
                    sting_sources::Action::Ban => source_data
                        .cache_http
                        .http
                        .remove_ban(
                            punishment.entry.guild_id,
                            punishment.entry.user_id,
                            Some(reason.as_str()),
                        )
                        .await
                        .map_err(EventError::Serenity),
                    sting_sources::Action::Timeout => punishment
                        .entry
                        .guild_id
                        .edit_member(
                            &source_data.cache_http.http,
                            punishment.entry.user_id,
                            serenity::all::EditMember::new()
                                .audit_log_reason(reason.as_str())
                                .enable_communication(),
                        )
                        .await
                        .map_err(EventError::Serenity)
                        .map(|_| ()),
                    sting_sources::Action::RemoveAllRoles => punishment
                        .entry
                        .guild_id
                        .edit_member(
                            &source_data.cache_http.http,
                            punishment.entry.user_id,
                            serenity::all::EditMember::new()
                                .audit_log_reason(reason.as_str())
                                .roles(Vec::new()),
                        )
                        .await
                        .map_err(EventError::Serenity)
                        .map(|_| ()),
                };

                match result {
                    Ok(_) => {
                        source
                            .update_sting_entry(
                                &source_data,
                                punishment.id,
                                sting_sources::UpdateStingEntry {
                                    state: Some(sting_sources::StingState::Handled),
                                    void_reason: Some(
                                        "Successfully reverted temporary punishment".into(),
                                    ),
                                    ..Default::default()
                                },
                            )
                            .await
                            .map_err(|e| {
                                EventError::Generic(format!("Log error failure: {:#?}", e))
                            })?;
                        Ok(())
                    }
                    Err(ue) => {
                        if let EventError::Serenity(ref e) = ue {
                            // Check if we have a http error
                            match e {
                                serenity::Error::Http(
                                    serenity::all::HttpError::UnsuccessfulRequest(e),
                                ) => {
                                    if e.status_code.is_server_error() || e.status_code == 429 {
                                        // Retry later
                                        return Err(ue);
                                    }

                                    source
                                        .update_sting_entry(
                                            &source_data,
                                            punishment.id,
                                            sting_sources::UpdateStingEntry {
                                                state: Some(sting_sources::StingState::Voided),
                                                void_reason: Some(format!(
                                                    "{}: {}",
                                                    e.status_code, e.error.message
                                                )),
                                                ..Default::default()
                                            },
                                        )
                                        .await
                                        .map_err(|e| {
                                            EventError::Generic(format!(
                                                "Log error failure: {:#?}",
                                                e
                                            ))
                                        })?;

                                    Err(ue)
                                }
                                serenity::Error::Model(e) => {
                                    source
                                        .update_sting_entry(
                                            &source_data,
                                            punishment.id,
                                            sting_sources::UpdateStingEntry {
                                                state: Some(sting_sources::StingState::Voided),
                                                void_reason: Some(format!(
                                                    "Log error failure: {:#?}",
                                                    e
                                                )),
                                                ..Default::default()
                                            },
                                        )
                                        .await
                                        .map_err(|e| {
                                            EventError::Generic(format!(
                                                "Log error failure: {:#?}",
                                                e
                                            ))
                                        })?;

                                    Err(ue)
                                }
                                _ => Ok(()),
                            }
                        } else {
                            source
                                .update_sting_entry(
                                    &source_data,
                                    punishment.id,
                                    sting_sources::UpdateStingEntry {
                                        state: Some(sting_sources::StingState::Voided),
                                        void_reason: Some(format!("{:#?}", ue)),
                                        ..Default::default()
                                    },
                                )
                                .await
                                .map_err(|e| {
                                    EventError::Generic(format!("Log error failure: {:#?}", e))
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
            Ok(Err(EventError::Serenity(e))) => {
                log::error!("Error while running unban [discord]: {}", e)
            }
            Ok(Err(EventError::Generic(e))) => {
                log::error!("Error while running unban [generic]: {}", e)
            }
        }
    }

    Ok(())
}

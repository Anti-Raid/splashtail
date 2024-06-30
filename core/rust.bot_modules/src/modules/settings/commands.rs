use crate::silverpelt::{silverpelt_cache::SILVERPELT_CACHE, GuildCommandConfiguration};
use crate::{Context, Error};
use botox::cache::CacheHttpImpl;
use futures_util::StreamExt;
use std::time::Duration;

/// Settings related to commands
#[poise::command(
    prefix_command,
    slash_command,
    user_cooldown = 1,
    guild_cooldown = 1,
    subcommands(
        "commands_check",
        "commands_enable",
        "commands_disable",
        "commands_modperms"
    )
)]
pub async fn commands(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Checks if a command is usable
#[poise::command(
    prefix_command,
    slash_command,
    user_cooldown = 1,
    guild_cooldown = 1,
    rename = "check"
)]
pub async fn commands_check(
    ctx: Context<'_>,
    #[description = "The command to check"] command: String,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command must be run in a guild".into());
    };

    // Find command in cache
    let Some(base_command) = command.split_whitespace().next() else {
        return Err("No command provided".into());
    };

    let data = ctx.data();

    // Check if the user has permission to use the command
    let cache_http = &CacheHttpImpl::from_ctx(ctx.serenity_context());
    let perm_res = crate::silverpelt::cmd::check_command(
        base_command,
        &command,
        guild_id,
        ctx.author().id,
        &data.pool,
        cache_http,
        &Some(ctx),
        crate::silverpelt::cmd::CheckCommandOptions {
            ignore_command_disabled: true,
            ..Default::default()
        },
    )
    .await;

    if !perm_res.is_ok() {
        return Err(format!(
            "You do NOT have permission to use this command?\n{}",
            perm_res.to_markdown()
        )
        .into());
    }

    ctx.say("You have permission to use this command").await?;

    Ok(())
}

/// Enables a module. Note that globally disabled modules cannot be used even if enabled
#[poise::command(
    prefix_command,
    slash_command,
    user_cooldown = 1,
    guild_cooldown = 1,
    rename = "enable"
)]
pub async fn commands_enable(
    ctx: Context<'_>,
    #[description = "The command to enable"] command: String,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command must be run in a guild".into());
    };

    // Find command in cache
    let Some(base_command) = command.split_whitespace().next() else {
        return Err("No command provided".into());
    };

    let Some(module) = crate::SILVERPELT_CACHE
        .command_id_module_map
        .get(base_command)
    else {
        return Err("Command not found".into());
    };

    let Some(module) = crate::SILVERPELT_CACHE.module_cache.get(module) else {
        return Err("Module not found".into());
    };

    if !module.commands_toggleable {
        return Err(format!(
            "Commands within the module `{}` cannot be toggled (enabled/disable) at this time!",
            module.id
        )
        .into());
    }

    let data = ctx.data();

    // Check if the user has permission to use the command
    let cache_http = &CacheHttpImpl::from_ctx(ctx.serenity_context());
    let perm_res = crate::silverpelt::cmd::check_command(
        base_command,
        &command,
        guild_id,
        ctx.author().id,
        &data.pool,
        cache_http,
        &Some(ctx),
        crate::silverpelt::cmd::CheckCommandOptions {
            ignore_command_disabled: true,
            ..Default::default()
        },
    )
    .await;

    if !perm_res.is_ok() {
        return Err(format!(
            "You can only modify commands that you have permission to use?\n{}",
            perm_res.to_markdown()
        )
        .into());
    }

    // Check if command is already enabled
    let mut tx = data.pool.begin().await?;

    let disabled = sqlx::query!(
        "SELECT disabled FROM guild_command_configurations WHERE guild_id = $1 AND command = $2",
        guild_id.to_string(),
        command
    )
    .fetch_optional(&mut *tx)
    .await?;

    if let Some(disabled) = disabled {
        // We have a module, now check
        if disabled.disabled.is_some() && !disabled.disabled.unwrap_or_default() {
            return Err("Command is already enabled".into());
        }

        sqlx::query!(
            "UPDATE guild_command_configurations SET disabled = false WHERE guild_id = $1 AND command = $2",
            guild_id.to_string(),
            command
        )
        .execute(&mut *tx)
        .await?;
    } else {
        // No module, create it
        sqlx::query!(
            "INSERT INTO guild_command_configurations (guild_id, command, disabled) VALUES ($1, $2, false)",
            guild_id.to_string(),
            command
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    tokio::spawn(async move {
        if let Err(err) = SILVERPELT_CACHE
            .command_permission_cache
            .invalidate_entries_if(move |k, _| k.0 == guild_id)
        {
            log::error!(
                "Failed to invalidate command permission cache for guild {}: {}",
                guild_id,
                err
            );
        } else {
            log::info!("Invalidated cache for guild {}", guild_id);
        }
    });

    ctx.say("Command enabled").await?;

    Ok(())
}

/// Enables a module. Note that globally disabled modules cannot be used even if enabled
#[poise::command(
    prefix_command,
    slash_command,
    user_cooldown = 1,
    guild_cooldown = 1,
    rename = "disable"
)]
pub async fn commands_disable(
    ctx: Context<'_>,
    #[description = "The command to disable"] command: String,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command must be run in a guild".into());
    };

    // Find command in cache
    let Some(base_command) = command.split_whitespace().next() else {
        return Err("No command provided".into());
    };

    let Some(module) = crate::SILVERPELT_CACHE
        .command_id_module_map
        .get(base_command)
    else {
        return Err("Command not found".into());
    };

    let Some(module) = crate::SILVERPELT_CACHE.module_cache.get(module) else {
        return Err("Module not found".into());
    };

    if !module.commands_toggleable {
        return Err(format!(
            "Commands within the module `{}` cannot be toggled (enabled/disable) at this time!",
            module.id
        )
        .into());
    }

    // Check if the user has permission to use the command
    let cache_http = &CacheHttpImpl::from_ctx(ctx.serenity_context());
    let perm_res = crate::silverpelt::cmd::check_command(
        base_command,
        &command,
        guild_id,
        ctx.author().id,
        &ctx.data().pool,
        cache_http,
        &Some(ctx),
        crate::silverpelt::cmd::CheckCommandOptions {
            ignore_command_disabled: true,
            ..Default::default()
        },
    )
    .await;

    if !perm_res.is_ok() {
        return Err(format!(
            "You can only modify commands that you have permission to use?\n{}",
            perm_res.to_markdown()
        )
        .into());
    }

    // Check if command is already enabled
    let mut tx = ctx.data().pool.begin().await?;

    let disabled = sqlx::query!(
        "SELECT disabled FROM guild_command_configurations WHERE guild_id = $1 AND command = $2",
        guild_id.to_string(),
        command
    )
    .fetch_optional(&mut *tx)
    .await?;

    if let Some(disabled) = disabled {
        // We have a command, now check
        if disabled.disabled.is_some() && disabled.disabled.unwrap_or_default() {
            return Err("Command is already disabled".into());
        }

        sqlx::query!(
            "UPDATE guild_command_configurations SET disabled = true WHERE guild_id = $1 AND command = $2",
            guild_id.to_string(),
            command
        )
        .execute(&mut *tx)
        .await?;
    } else {
        // No module, create it
        sqlx::query!(
            "INSERT INTO guild_command_configurations (guild_id, command, disabled) VALUES ($1, $2, true)",
            guild_id.to_string(),
            command
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    tokio::spawn(async move {
        if let Err(err) = SILVERPELT_CACHE
            .command_permission_cache
            .invalidate_entries_if(move |k, _| k.0 == guild_id)
        {
            log::error!(
                "Failed to invalidate command permission cache for guild {}: {}",
                guild_id,
                err
            );
        } else {
            log::info!("Invalidated cache for guild {}", guild_id);
        }
    });

    ctx.say("Command disabled").await?;

    Ok(())
}

/// Modifies the permissions and state of a command
#[poise::command(
    prefix_command,
    slash_command,
    user_cooldown = 1,
    guild_cooldown = 1,
    rename = "modperms"
)]
pub async fn commands_modperms(
    ctx: Context<'_>,
    #[description = "The command to modify"] command: String,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command must be run in a guild".into());
    };

    let data = ctx.data();

    // Find command in cache
    let Some(base_command) = command.split_whitespace().next() else {
        return Err("No command provided".into());
    };

    let Some(module) = crate::SILVERPELT_CACHE
        .command_id_module_map
        .get(base_command)
    else {
        return Err("Command not found".into());
    };

    let Some(module) = crate::SILVERPELT_CACHE.module_cache.get(module) else {
        return Err("Module not found".into());
    };

    // Check if the user has permission to use the command
    let cache_http = &CacheHttpImpl::from_ctx(ctx.serenity_context());
    let perm_res = crate::silverpelt::cmd::check_command(
        base_command,
        &command,
        guild_id,
        ctx.author().id,
        &data.pool,
        cache_http,
        &Some(ctx),
        crate::silverpelt::cmd::CheckCommandOptions {
            ignore_command_disabled: true,
            ..Default::default()
        },
    )
    .await;

    if !perm_res.is_ok() {
        return Err(format!(
            "You can only modify commands that you have permission to use?\n{}",
            perm_res.to_markdown()
        )
        .into());
    }

    // Fetch the command config. This is what is used as a base for the editing process
    let command_config = crate::silverpelt::module_config::get_exact_command_configuration(
        &data.pool,
        guild_id.to_string().as_str(),
        base_command,
    )
    .await?;

    let mut new_command_config = {
        if let Some(command_config) = command_config {
            command_config
        } else {
            GuildCommandConfiguration {
                id: "".to_string(),
                guild_id: guild_id.to_string(),
                command: command.clone(),
                disabled: None,
                perms: None,
            }
        }
    };

    // Also, fetch the commands extended data
    let cmd_extended_data = crate::silverpelt::module_config::get_command_extended_data(
        &crate::silverpelt::utils::permute_command_names(&command),
    )?;

    fn command_config_to_edit_message<'a>(
        command_config: &GuildCommandConfiguration,
    ) -> poise::CreateReply<'a> {
        let mut msg = format!("**`{}`**\n\n", command_config.command);

        if let Some(ref perms) = command_config.perms {
            msg.push_str(&format!("Permissions:\n{}\n", perms));
        } else {
            msg.push_str("Permissions: None (using default permissions)\n");
        }

        if let Some(disabled) = command_config.disabled {
            msg.push_str(&format!("Disabled: {}\n", disabled));
        } else {
            msg.push_str("Disabled: None (using default configuration)\n");
        }

        poise::CreateReply::new()
            .content(msg)
            .ephemeral(true)
            .components(vec![serenity::all::CreateActionRow::Buttons(vec![
                serenity::all::CreateButton::new("perms/editraw")
                    .style(serenity::all::ButtonStyle::Primary)
                    .label("Open Raw Permission Editor"),
                if command_config.disabled.unwrap_or_default() {
                    serenity::all::CreateButton::new("cmd/enable")
                        .style(serenity::all::ButtonStyle::Success)
                        .label("Enable Command")
                } else {
                    serenity::all::CreateButton::new("cmd/disable")
                        .style(serenity::all::ButtonStyle::Danger)
                        .label("Disable Command")
                },
                serenity::all::CreateButton::new("cmd/reset-toggle")
                    .style(serenity::all::ButtonStyle::Danger)
                    .label("Reset Command Toggle"),
                serenity::all::CreateButton::new("perms/reset")
                    .style(serenity::all::ButtonStyle::Danger)
                    .label("Reset Command Perms"),
                serenity::all::CreateButton::new("cmd/save")
                    .style(serenity::all::ButtonStyle::Secondary)
                    .label("Save Command Configuration"),
            ])])
    }

    let msg = ctx
        .send(command_config_to_edit_message(&new_command_config))
        .await?
        .into_message()
        .await?;

    let collector = msg
        .await_component_interactions(ctx.serenity_context().shard.clone())
        .author_id(ctx.author().id)
        .timeout(Duration::from_secs(600));

    let mut collect_stream = collector.stream();

    while let Some(item) = collect_stream.next().await {
        let item_id = item.data.custom_id.as_str();

        let mut response_deferred = false;
        match item_id {
            "cmd/enable" => {
                if !module.commands_toggleable {
                    item.create_response(
                        &ctx.serenity_context().http,
                        poise::serenity_prelude::CreateInteractionResponse::Message(
                            poise::CreateReply::new()
                                .content(format!(
                                    "Commands within the module `{}` cannot be toggled (enabled/disable) at this time!",
                                    module.id
                                ))
                                .to_slash_initial_response(
                                    serenity::all::CreateInteractionResponseMessage::default(),
                                ),
                        ),
                    )
                    .await?;
                    continue;
                }

                let perm_res = crate::silverpelt::cmd::check_command(
                    "commands",
                    "commands enable",
                    guild_id,
                    ctx.author().id,
                    &data.pool,
                    cache_http,
                    &Some(ctx),
                    crate::silverpelt::cmd::CheckCommandOptions::default(),
                )
                .await;

                if !perm_res.is_ok() {
                    item.create_response(
                        &ctx.serenity_context().http,
                        poise::serenity_prelude::CreateInteractionResponse::Message(
                            poise::CreateReply::new()
                                .content(format!(
                                    "Enabling commands requires permission to use the ``commands enable`` command!\n{}",
                                    perm_res.to_markdown()
                                ))
                                .to_slash_initial_response(
                                    serenity::all::CreateInteractionResponseMessage::default(),
                                ),
                        ),
                    )
                    .await?;
                    continue;
                }

                new_command_config.disabled = Some(false);
            }
            "cmd/disable" => {
                if !module.commands_toggleable {
                    item.create_response(
                        &ctx.serenity_context().http,
                        poise::serenity_prelude::CreateInteractionResponse::Message(
                            poise::CreateReply::new()
                                .content(format!(
                                    "Commands within the module `{}` cannot be toggled (enabled/disable) at this time!",
                                    module.id
                                ))
                                .to_slash_initial_response(
                                    serenity::all::CreateInteractionResponseMessage::default(),
                                ),
                        ),
                    )
                    .await?;
                    continue;
                }

                let perm_res = crate::silverpelt::cmd::check_command(
                    "commands",
                    "commands disable",
                    guild_id,
                    ctx.author().id,
                    &data.pool,
                    cache_http,
                    &Some(ctx),
                    crate::silverpelt::cmd::CheckCommandOptions::default(),
                )
                .await;

                if !perm_res.is_ok() {
                    item.create_response(
                        &ctx.serenity_context().http,
                        poise::serenity_prelude::CreateInteractionResponse::Message(
                            poise::CreateReply::new()
                                .content(format!(
                                    "Disabling commands requires permission to use the ``commands disable`` command!\n{}",
                                    perm_res.to_markdown()
                                ))
                                .to_slash_initial_response(
                                    serenity::all::CreateInteractionResponseMessage::default(),
                                ),
                        ),
                    )
                    .await?;
                    continue;
                }

                new_command_config.disabled = Some(true);
            }
            "cmd/reset-toggle" => {
                if !module.commands_toggleable {
                    item.create_response(
                        &ctx.serenity_context().http,
                        poise::serenity_prelude::CreateInteractionResponse::Message(
                            poise::CreateReply::new()
                                .content(format!(
                                    "Commands within the module `{}` cannot be toggled (enabled/disable) at this time!",
                                    module.id
                                ))
                                .to_slash_initial_response(
                                    serenity::all::CreateInteractionResponseMessage::default(),
                                ),
                        ),
                    )
                    .await?;
                    continue;
                }

                // If there is no change, then only do permission checking
                if cmd_extended_data.is_default_enabled {
                    let perm_res = crate::silverpelt::cmd::check_command(
                        "commands",
                        "commands enable",
                        guild_id,
                        ctx.author().id,
                        &data.pool,
                        cache_http,
                        &Some(ctx),
                        crate::silverpelt::cmd::CheckCommandOptions::default(),
                    )
                    .await;

                    if !perm_res.is_ok() {
                        item.create_response(
                                &ctx.serenity_context().http,
                                poise::serenity_prelude::CreateInteractionResponse::Message(
                                    poise::CreateReply::new()
                                        .content(format!(
                                            "Enabling commands requires permission to use the ``commands enable`` command!\n{}",
                                            perm_res.to_markdown()
                                        ))
                                        .to_slash_initial_response(
                                            serenity::all::CreateInteractionResponseMessage::default(),
                                        ),
                                ),
                            )
                            .await?;
                        continue;
                    }
                } else {
                    let perm_res = crate::silverpelt::cmd::check_command(
                        "commands",
                        "commands disable",
                        guild_id,
                        ctx.author().id,
                        &data.pool,
                        cache_http,
                        &Some(ctx),
                        crate::silverpelt::cmd::CheckCommandOptions::default(),
                    )
                    .await;

                    if !perm_res.is_ok() {
                        item.create_response(
                                &ctx.serenity_context().http,
                                poise::serenity_prelude::CreateInteractionResponse::Message(
                                    poise::CreateReply::new()
                                        .content(format!(
                                            "Disabling commands requires permission to use the ``commands disable`` command!\n{}",
                                            perm_res.to_markdown()
                                        ))
                                        .to_slash_initial_response(
                                            serenity::all::CreateInteractionResponseMessage::default(),
                                        ),
                                ),
                            )
                            .await?;
                        continue;
                    }
                }

                new_command_config.disabled = None;
            }
            "perms/reset" => {
                let perm_res = crate::silverpelt::cmd::check_command(
                    base_command,
                    &command,
                    guild_id,
                    ctx.author().id,
                    &data.pool,
                    cache_http,
                    &Some(ctx),
                    crate::silverpelt::cmd::CheckCommandOptions {
                        custom_command_configuration: Some(
                            crate::silverpelt::GuildCommandConfiguration {
                                perms: None,
                                ..new_command_config.clone()
                            },
                        ),
                        ..Default::default()
                    },
                )
                .await;

                if !perm_res.is_ok() {
                    item.create_response(
                            &ctx.serenity_context().http,
                            poise::serenity_prelude::CreateInteractionResponse::Message(
                                poise::CreateReply::new()
                                    .content(format!(
                                        "You must have permission to use the command with the permissions you have provided: {}",
                                        perm_res.to_markdown()
                                    ))
                                    .to_slash_initial_response(
                                        serenity::all::CreateInteractionResponseMessage::default(),
                                    ),
                            ),
                        )
                        .await?;
                    continue;
                }

                new_command_config.perms = None;
            }
            "perms/editraw" => {
                // Open a modal in response
                let perms = new_command_config.perms.clone().unwrap_or_default();

                let mut perms_json = serde_json::to_string(&perms).unwrap_or_default();

                if perms_json.is_empty() {
                    perms_json = "{}".to_string();
                }

                if perms_json.len() > 100 {
                    perms_json = perms_json.chars().take(97).collect::<String>() + "...";
                }

                let modal = serenity::all::CreateQuickModal::new("Edit Permissions")
                    .timeout(std::time::Duration::from_secs(300))
                    .field(
                        serenity::all::CreateInputText::new(
                            serenity::all::InputTextStyle::Short,
                            "Permissions",
                            "permissions",
                        )
                        .placeholder(perms_json),
                    );

                let resp = item.quick_modal(ctx.serenity_context(), modal).await?;
                response_deferred = true;

                let Some(resp) = resp else { continue };

                if resp.inputs.is_empty() {
                    continue;
                }

                let perms_str = &resp.inputs[0];

                let perms = serde_json::from_str::<crate::silverpelt::PermissionChecks>(perms_str);

                match perms {
                    Ok(perms) => {
                        let parsed =
                            crate::silverpelt::validators::parse_permission_checks(&perms)?;

                        let perm_res = crate::silverpelt::cmd::check_command(
                            base_command,
                            &command,
                            guild_id,
                            ctx.author().id,
                            &data.pool,
                            cache_http,
                            &Some(ctx),
                            crate::silverpelt::cmd::CheckCommandOptions {
                                custom_command_configuration: Some(
                                    crate::silverpelt::GuildCommandConfiguration {
                                        perms: Some(parsed.clone()),
                                        ..new_command_config.clone()
                                    },
                                ),
                                ..Default::default()
                            },
                        )
                        .await;

                        if !perm_res.is_ok() {
                            item.create_response(
                                    &ctx.serenity_context().http,
                                    poise::serenity_prelude::CreateInteractionResponse::Message(
                                        poise::CreateReply::new()
                                            .content(format!(
                                                "You must have permission to use the command with the permissions you have provided: {}",
                                                perm_res.to_markdown()
                                            ))
                                            .to_slash_initial_response(
                                                serenity::all::CreateInteractionResponseMessage::default(),
                                            ),
                                    ),
                                )
                                .await?;
                            continue;
                        }

                        new_command_config.perms = Some(parsed);
                    }
                    Err(err) => {
                        ctx.say(format!("Failed to parse permissions: {}", err))
                            .await?;
                        continue;
                    }
                }
            }
            "cmd/save" => {
                let perm_res: crate::silverpelt::permissions::PermissionResult =
                    crate::silverpelt::cmd::check_command(
                        base_command,
                        &command,
                        guild_id,
                        ctx.author().id,
                        &data.pool,
                        cache_http,
                        &Some(ctx),
                        crate::silverpelt::cmd::CheckCommandOptions {
                            ignore_command_disabled: true,
                            ignore_cache: true,
                            cache_result: false,
                            custom_command_configuration: Some(new_command_config.clone()),
                            ..Default::default()
                        },
                    )
                    .await;

                if !perm_res.is_ok() {
                    return Err(format!("You can only modify commands to something that you have permission to use!\n{}", perm_res.to_markdown()).into());
                }

                let mut tx = data.pool.begin().await?;

                // Check if guild command config exists now
                let count = sqlx::query!(
                    "SELECT COUNT(*) FROM guild_command_configurations WHERE guild_id = $1 AND command = $2",
                    guild_id.to_string(),
                    command
                )
                .fetch_one(&mut *tx)
                .await?
                .count
                .unwrap_or_default();

                let new_perms = serde_json::to_value(new_command_config.perms)?;

                if count > 0 {
                    sqlx::query!(
                        "UPDATE guild_command_configurations SET perms = $1, disabled = $2 WHERE guild_id = $3 AND command = $4",
                        new_perms,
                        new_command_config.disabled,
                        guild_id.to_string(),
                        command
                    )
                    .execute(&mut *tx)
                    .await?;
                } else {
                    sqlx::query!(
                        "INSERT INTO guild_command_configurations (guild_id, command, perms, disabled) VALUES ($1, $2, $3, $4)",
                        guild_id.to_string(),
                        command,
                        new_perms,
                        new_command_config.disabled
                    )
                    .execute(&mut *tx)
                    .await?;
                }

                item.create_response(
                    &ctx.serenity_context().http,
                    poise::serenity_prelude::CreateInteractionResponse::Message(
                        poise::CreateReply::new()
                            .content("Command configuration saved")
                            .to_slash_initial_response(
                                serenity::all::CreateInteractionResponseMessage::default(),
                            ),
                    ),
                )
                .await?;

                tx.commit().await?;

                tokio::spawn(async move {
                    if let Err(err) = SILVERPELT_CACHE
                        .command_permission_cache
                        .invalidate_entries_if(move |k, _| k.0 == guild_id)
                    {
                        log::error!(
                            "Failed to invalidate command permission cache for guild {}: {}",
                            guild_id,
                            err
                        );
                    } else {
                        log::info!("Invalidated cache for guild {}", guild_id);
                    }
                });

                break;
            }
            _ => {}
        }

        if !response_deferred {
            item.defer(&ctx.serenity_context().http).await?;
        }

        // Send the updated message
        item.edit_response(
            &ctx.serenity_context().http,
            command_config_to_edit_message(&new_command_config)
                .to_slash_initial_response_edit(serenity::all::EditInteractionResponse::default()),
        )
        .await?;
    }

    Ok(())
}

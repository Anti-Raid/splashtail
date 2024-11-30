use futures_util::StreamExt;
use permissions::types::PermissionCheck;
use silverpelt::types::GuildCommandConfiguration;
use silverpelt::Context;
use silverpelt::Error;
use std::time::Duration;

/// Settings related to commands
#[poise::command(
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
#[poise::command(slash_command, user_cooldown = 1, guild_cooldown = 1, rename = "check")]
pub async fn commands_check(
    ctx: Context<'_>,
    #[description = "The command to check"] command: String,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command must be run in a guild".into());
    };

    let data = ctx.data();

    // Check if the user has permission to use the command
    let perm_res = permission_checks::check_command(
        &data.silverpelt_cache,
        &command,
        guild_id,
        ctx.author().id,
        &data.pool,
        ctx.serenity_context(),
        &data.reqwest,
        &Some(ctx),
        permission_checks::CheckCommandOptions {
            ignore_command_disabled: true,
            channel_id: Some(ctx.channel_id()),
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
    slash_command,
    user_cooldown = 1,
    guild_cooldown = 1,
    rename = "enable"
)]
pub async fn commands_enable(
    ctx: Context<'_>,
    #[description = "The command to enable"] command: String,
) -> Result<(), Error> {
    let data = ctx.data();

    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command must be run in a guild".into());
    };

    if command.is_empty() {
        return Err("No command provided".into());
    }

    // Find command in cache
    let command_permutations = silverpelt::utils::permute_command_names(&command);

    let Some(module) = data
        .silverpelt_cache
        .command_id_module_map
        .get(&command_permutations[0])
    else {
        return Err("Command not found".into());
    };

    let Some(module) = data.silverpelt_cache.module_cache.get(module.value()) else {
        return Err("Module not found".into());
    };

    if !module.commands_toggleable() {
        return Err(format!(
            "Commands within the module `{}` cannot be toggled (enabled/disable) at this time!",
            module.id()
        )
        .into());
    }

    // Check if the user has permission to use the command
    let perm_res = permission_checks::check_command(
        &data.silverpelt_cache,
        &command,
        guild_id,
        ctx.author().id,
        &data.pool,
        ctx.serenity_context(),
        &data.reqwest,
        &Some(ctx),
        permission_checks::CheckCommandOptions {
            ignore_command_disabled: true,
            channel_id: Some(ctx.channel_id()),
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
            "UPDATE guild_command_configurations SET disabled = false, last_updated_by = $3, last_updated_at = NOW() WHERE guild_id = $1 AND command = $2",
            guild_id.to_string(),
            command,
            ctx.author().id.to_string()
        )
        .execute(&mut *tx)
        .await?;
    } else {
        // No module, create it
        sqlx::query!(
            "INSERT INTO guild_command_configurations (guild_id, command, disabled, created_by) VALUES ($1, $2, false, $3)",
            guild_id.to_string(),
            command,
            ctx.author().id.to_string()
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    ctx.say("Command enabled").await?;

    Ok(())
}

/// Enables a module. Note that globally disabled modules cannot be used even if enabled
#[poise::command(
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

    if command.is_empty() {
        return Err("No command provided".into());
    }

    let data = ctx.data();

    // Find command in cache
    let command_permutations = silverpelt::utils::permute_command_names(&command);

    let Some(module) = data
        .silverpelt_cache
        .command_id_module_map
        .get(&command_permutations[0])
    else {
        return Err("Command not found".into());
    };

    let Some(module) = data.silverpelt_cache.module_cache.get(module.value()) else {
        return Err("Module not found".into());
    };

    if !module.commands_toggleable() {
        return Err(format!(
            "Commands within the module `{}` cannot be toggled (enabled/disable) at this time!",
            module.id()
        )
        .into());
    }

    // Check if the user has permission to use the command
    let perm_res = permission_checks::check_command(
        &data.silverpelt_cache,
        &command,
        guild_id,
        ctx.author().id,
        &ctx.data().pool,
        ctx.serenity_context(),
        &data.reqwest,
        &Some(ctx),
        permission_checks::CheckCommandOptions {
            ignore_command_disabled: true,
            channel_id: Some(ctx.channel_id()),
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
            "UPDATE guild_command_configurations SET disabled = true, last_updated_by = $3, last_updated_at = NOW() WHERE guild_id = $1 AND command = $2",
            guild_id.to_string(),
            command,
            ctx.author().id.to_string()
        )
        .execute(&mut *tx)
        .await?;
    } else {
        // No module, create it
        sqlx::query!(
            "INSERT INTO guild_command_configurations (guild_id, command, disabled, created_by) VALUES ($1, $2, true, $3)",
            guild_id.to_string(),
            command,
            ctx.author().id.to_string()
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    ctx.say("Command disabled").await?;

    Ok(())
}

/// Modifies the permissions and state of a command
#[poise::command(
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

    if command.is_empty() {
        return Err("No command provided".into());
    }

    // Find command in cache
    let command_permutations = silverpelt::utils::permute_command_names(&command);

    let Some(module) = data
        .silverpelt_cache
        .command_id_module_map
        .get(&command_permutations[0])
    else {
        return Err("Command not found".into());
    };

    let Some(module) = data.silverpelt_cache.module_cache.get(module.value()) else {
        return Err("Module not found".into());
    };

    // Check if the user has permission to use the command
    let perm_res = permission_checks::check_command(
        &data.silverpelt_cache,
        &command,
        guild_id,
        ctx.author().id,
        &data.pool,
        ctx.serenity_context(),
        &data.reqwest,
        &Some(ctx),
        permission_checks::CheckCommandOptions {
            ignore_command_disabled: true,
            channel_id: Some(ctx.channel_id()),
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
    let command_config = silverpelt::module_config::get_exact_command_configuration(
        &data.pool,
        guild_id.to_string().as_str(),
        &command,
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
    let cmd_extended_data = silverpelt::module_config::get_command_extended_data(
        &data.silverpelt_cache,
        &silverpelt::utils::permute_command_names(&command),
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
            .components(vec![serenity::all::CreateActionRow::Buttons(
                vec![
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
                ]
                .into(),
            )])
    }

    let msg = ctx
        .send(command_config_to_edit_message(&new_command_config))
        .await?
        .into_message()
        .await?;

    let collector = msg
        .id
        .await_component_interactions(ctx.serenity_context().shard.clone())
        .author_id(ctx.author().id)
        .timeout(Duration::from_secs(600));

    let mut collect_stream = collector.stream();

    while let Some(item) = collect_stream.next().await {
        let item_id = item.data.custom_id.as_str();

        let mut response_deferred = false;
        match item_id {
            "cmd/enable" => {
                if !module.commands_toggleable() {
                    item.create_response(
                        &ctx.serenity_context().http,
                        poise::serenity_prelude::CreateInteractionResponse::Message(
                            poise::CreateReply::new()
                                .content(format!(
                                    "Commands within the module `{}` cannot be toggled (enabled/disable) at this time!",
                                    module.id()
                                ))
                                .to_slash_initial_response(
                                    serenity::all::CreateInteractionResponseMessage::default(),
                                ),
                        ),
                    )
                    .await?;
                    continue;
                }

                let perm_res = permission_checks::check_command(
                    &data.silverpelt_cache,
                    "commands enable",
                    guild_id,
                    ctx.author().id,
                    &data.pool,
                    ctx.serenity_context(),
                    &data.reqwest,
                    &Some(ctx),
                    permission_checks::CheckCommandOptions {
                        channel_id: Some(ctx.channel_id()),
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
                if !module.commands_toggleable() {
                    item.create_response(
                        &ctx.serenity_context().http,
                        poise::serenity_prelude::CreateInteractionResponse::Message(
                            poise::CreateReply::new()
                                .content(format!(
                                    "Commands within the module `{}` cannot be toggled (enabled/disable) at this time!",
                                    module.id()
                                ))
                                .to_slash_initial_response(
                                    serenity::all::CreateInteractionResponseMessage::default(),
                                ),
                        ),
                    )
                    .await?;
                    continue;
                }

                let perm_res = permission_checks::check_command(
                    &data.silverpelt_cache,
                    "commands disable",
                    guild_id,
                    ctx.author().id,
                    &data.pool,
                    ctx.serenity_context(),
                    &data.reqwest,
                    &Some(ctx),
                    permission_checks::CheckCommandOptions {
                        channel_id: Some(ctx.channel_id()),
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
                if !module.commands_toggleable() {
                    item.create_response(
                        &ctx.serenity_context().http,
                        poise::serenity_prelude::CreateInteractionResponse::Message(
                            poise::CreateReply::new()
                                .content(format!(
                                    "Commands within the module `{}` cannot be toggled (enabled/disable) at this time!",
                                    module.id()
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
                    let perm_res = permission_checks::check_command(
                        &data.silverpelt_cache,
                        "commands enable",
                        guild_id,
                        ctx.author().id,
                        &data.pool,
                        ctx.serenity_context(),
                        &data.reqwest,
                        &Some(ctx),
                        permission_checks::CheckCommandOptions {
                            channel_id: Some(ctx.channel_id()),
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
                    let perm_res = permission_checks::check_command(
                        &data.silverpelt_cache,
                        "commands disable",
                        guild_id,
                        ctx.author().id,
                        &data.pool,
                        ctx.serenity_context(),
                        &data.reqwest,
                        &Some(ctx),
                        permission_checks::CheckCommandOptions {
                            channel_id: Some(ctx.channel_id()),
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
                let perm_res = permission_checks::check_command(
                    &data.silverpelt_cache,
                    &command,
                    guild_id,
                    ctx.author().id,
                    &data.pool,
                    ctx.serenity_context(),
                    &data.reqwest,
                    &Some(ctx),
                    permission_checks::CheckCommandOptions {
                        custom_command_configuration: Some(GuildCommandConfiguration {
                            perms: None,
                            ..new_command_config.clone()
                        }),
                        channel_id: Some(ctx.channel_id()),
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

                let perms = serde_json::from_str::<PermissionCheck>(perms_str);

                match perms {
                    Ok(perms) => {
                        let parsed = permissions::parse::parse_permission_check(&perms).await?;

                        let perm_res = permission_checks::check_command(
                            &data.silverpelt_cache,
                            &command,
                            guild_id,
                            ctx.author().id,
                            &data.pool,
                            ctx.serenity_context(),
                            &data.reqwest,
                            &Some(ctx),
                            permission_checks::CheckCommandOptions {
                                custom_command_configuration: Some(GuildCommandConfiguration {
                                    perms: Some(parsed.clone()),
                                    ..new_command_config.clone()
                                }),
                                channel_id: Some(ctx.channel_id()),
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
                let perm_res = permission_checks::check_command(
                    &data.silverpelt_cache,
                    &command,
                    guild_id,
                    ctx.author().id,
                    &data.pool,
                    ctx.serenity_context(),
                    &data.reqwest,
                    &Some(ctx),
                    permission_checks::CheckCommandOptions {
                        ignore_command_disabled: true,
                        custom_command_configuration: Some(new_command_config.clone()),
                        channel_id: Some(ctx.channel_id()),
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
                        "UPDATE guild_command_configurations SET perms = $1, disabled = $2, last_updated_by = $3, last_updated_at = NOW() WHERE guild_id = $4 AND command = $5",
                        new_perms,
                        new_command_config.disabled,
                        ctx.author().id.to_string(),
                        guild_id.to_string(),
                        command
                    )
                    .execute(&mut *tx)
                    .await?;
                } else {
                    sqlx::query!(
                        "INSERT INTO guild_command_configurations (guild_id, command, perms, disabled, created_by) VALUES ($1, $2, $3, $4, $5)",
                        guild_id.to_string(),
                        command,
                        new_perms,
                        new_command_config.disabled,
                        ctx.author().id.to_string()
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

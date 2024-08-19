use std::time::Duration;

use futures_util::StreamExt;
use permissions::types::PermissionChecks;
use serenity::all::AutocompleteChoice;
use silverpelt::types::GuildModuleConfiguration;
use silverpelt::Context;
use silverpelt::Error;

async fn module_list_autocomplete<'a>(
    ctx: Context<'_>,
    partial: &'a str,
) -> Vec<AutocompleteChoice<'a>> {
    let data = ctx.data();
    let mut ac = Vec::new();

    for (_, module) in data.silverpelt_cache.module_cache.iter() {
        if module.name.to_lowercase().contains(&partial.to_lowercase())
            || module.id.to_lowercase().contains(&partial.to_lowercase())
        {
            ac.push(AutocompleteChoice::new(module.name, module.id));
        }
    }

    ac
}

#[poise::command(
    prefix_command,
    slash_command,
    user_cooldown = 1,
    guild_cooldown = 1,
    subcommands(
        "modules_list",
        "modules_enable",
        "modules_disable",
        "modules_modperms"
    )
)]
pub async fn modules(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Lists all module configurations currently setup
#[poise::command(
    prefix_command,
    slash_command,
    user_cooldown = 1,
    guild_cooldown = 1,
    rename = "list"
)]
pub async fn modules_list(ctx: Context<'_>) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command must be run in a guild".into());
    };

    let data = ctx.data();

    let module_configs = sqlx::query!(
        "SELECT module, disabled FROM guild_module_configurations WHERE guild_id = $1",
        guild_id.to_string()
    )
    .fetch_all(&data.pool)
    .await?;

    let mut msg = "**Module Configurations**\n\n".to_string();

    let mut done_modules = Vec::new();
    for module_config in module_configs {
        let Some(module) = data
            .silverpelt_cache
            .module_cache
            .get(&module_config.module)
        else {
            continue;
        };

        let module_id = module_config.module;

        if let Some(disabled) = module_config.disabled {
            msg.push_str(&format!(
                "**{}**: {} [module id = {}]\n",
                module.name,
                if disabled { "Disabled" } else { "Enabled" },
                module_id
            ));
        } else {
            msg.push_str(&format!(
                "**{}**: {} [default] [module id = {}]\n",
                module.name,
                if module.is_default_enabled {
                    "Enabled"
                } else {
                    "Disabled"
                },
                module_id
            ));
        }

        done_modules.push(module_id);
    }

    for (_, module) in data.silverpelt_cache.module_cache.iter() {
        if done_modules.contains(&module.id.to_string()) {
            continue;
        }

        msg.push_str(&format!(
            "**{}**: {} [default, config not modified] [module id = {}]\n",
            module.name,
            if module.is_default_enabled {
                "Enabled"
            } else {
                "Disabled"
            },
            module.id
        ));
    }

    ctx.say(msg).await?;

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
pub async fn modules_enable(
    ctx: Context<'_>,
    #[description = "The module to enable"]
    #[autocomplete = "module_list_autocomplete"]
    module: String,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command must be run in a guild".into());
    };

    let data = ctx.data();

    // Check that the module exists
    let Some(module_data) = data.silverpelt_cache.module_cache.get(&module) else {
        return Err(format!(
            "The module you are trying to disable ({}) does not exist",
            module
        )
        .into());
    };

    if !module_data.toggleable {
        return Err("This module cannot be enabled/disabled".into());
    }

    // Check for a module_configuration in db
    // If it doesn't exist, create it
    let mut tx = data.pool.begin().await?;

    let disabled = sqlx::query!(
        "SELECT disabled FROM guild_module_configurations WHERE guild_id = $1 AND module = $2 FOR UPDATE",
        guild_id.to_string(),
        module
    )
    .fetch_optional(&mut *tx)
    .await?;

    if let Some(disabled) = disabled {
        // We have a module, now check
        if disabled.disabled.is_some() && !disabled.disabled.unwrap_or_default() {
            return Err("Module is already enabled".into());
        }

        sqlx::query!(
            "UPDATE guild_module_configurations SET disabled = false WHERE guild_id = $1 AND module = $2",
            guild_id.to_string(),
            module
        )
        .execute(&mut *tx)
        .await?;
    } else {
        // No module, create it
        sqlx::query!(
            "INSERT INTO guild_module_configurations (guild_id, module, disabled) VALUES ($1, $2, false)",
            guild_id.to_string(),
            module
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    data.silverpelt_cache
        .module_enabled_cache
        .invalidate(&(guild_id, module))
        .await;

    ctx.say("Module enabled successfully!").await?;

    Ok(())
}

/// Disables a module. Note that certain modules may not be disablable
#[poise::command(
    prefix_command,
    slash_command,
    user_cooldown = 1,
    guild_cooldown = 1,
    rename = "disable"
)]
pub async fn modules_disable(
    ctx: Context<'_>,
    #[description = "The module to disable"]
    #[autocomplete = "module_list_autocomplete"]
    module: String,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command must be run in a guild".into());
    };

    let data = ctx.data();

    // Check that the module exists
    let Some(module_data) = data.silverpelt_cache.module_cache.get(&module) else {
        return Err(format!(
            "The module you are trying to disable ({}) does not exist",
            module
        )
        .into());
    };

    if !module_data.toggleable {
        return Err("This module cannot be enabled/disabled".into());
    }

    // Check for a module_configuration in db
    // If it doesn't exist, create it
    let mut tx = data.pool.begin().await?;

    let disabled = sqlx::query!(
        "SELECT disabled FROM guild_module_configurations WHERE guild_id = $1 AND module = $2 FOR UPDATE",
        guild_id.to_string(),
        module
    )
    .fetch_optional(&mut *tx)
    .await?;

    if let Some(disabled) = disabled {
        // We have a module, now check
        if disabled.disabled.is_some() && disabled.disabled.unwrap_or_default() {
            return Err("Module is already disabled".into());
        }

        sqlx::query!(
            "UPDATE guild_module_configurations SET disabled = true WHERE guild_id = $1 AND module = $2",
            guild_id.to_string(),
            module
        )
        .execute(&mut *tx)
        .await?;
    } else {
        // No module, create it
        sqlx::query!(
            "INSERT INTO guild_module_configurations (guild_id, module, disabled) VALUES ($1, $2, true)",
            guild_id.to_string(),
            module
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    data.silverpelt_cache
        .module_enabled_cache
        .invalidate(&(guild_id, module))
        .await;

    ctx.say("Module disabled successfully!").await?;

    Ok(())
}

/// Modifies the permissions and state of a module
#[poise::command(
    prefix_command,
    slash_command,
    user_cooldown = 1,
    guild_cooldown = 1,
    rename = "modperms"
)]
pub async fn modules_modperms(
    ctx: Context<'_>,
    #[description = "The module to modify"] module: String,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command must be run in a guild".into());
    };

    let data = ctx.data();

    let Some(module) = data.silverpelt_cache.module_cache.get(&module) else {
        return Err("Module not found".into());
    };

    let cache_http = botox::cache::CacheHttpImpl::from_ctx(ctx.serenity_context());
    let perm_res = silverpelt::cmd::check_command(
        &data.silverpelt_cache,
        &format!("acl__modules_modperms {}", module.id),
        guild_id,
        ctx.author().id,
        &data.pool,
        &cache_http,
        &data.reqwest,
        &Some(ctx),
        silverpelt::cmd::CheckCommandOptions::default(),
    )
    .await;

    if !perm_res.is_ok() {
        return Err(format!(
            "You can only modify modules that you have permission to modify?\n{}",
            perm_res.to_markdown()
        )
        .into());
    }

    let module_config = silverpelt::module_config::get_module_configuration(
        &data.pool,
        guild_id.to_string().as_str(),
        module.id,
    )
    .await?;

    let mut new_module_config = {
        if let Some(module_config) = module_config {
            module_config
        } else {
            GuildModuleConfiguration {
                id: "".to_string(),
                guild_id: guild_id.to_string(),
                module: module.id.to_string(),
                disabled: None,
                default_perms: None,
            }
        }
    };

    fn module_config_to_edit_message<'a>(
        module_config: &GuildModuleConfiguration,
    ) -> poise::CreateReply<'a> {
        let mut msg = format!("**`{}`**\n\n", module_config.module);

        if let Some(ref default_perms) = module_config.default_perms {
            msg.push_str(&format!("Default Permissions:\n{}\n", default_perms));
        } else {
            msg.push_str("Default Permissions: None (set these to allow broad control over the permissions of a module)\n");
        }

        if let Some(disabled) = module_config.disabled {
            msg.push_str(&format!("Disabled: {}\n", disabled));
        } else {
            msg.push_str("Disabled: None (using default configuration)\n");
        }

        poise::CreateReply::new()
            .content(msg)
            .ephemeral(true)
            .components(vec![serenity::all::CreateActionRow::Buttons(vec![
                serenity::all::CreateButton::new("default-perms/editraw")
                    .style(serenity::all::ButtonStyle::Primary)
                    .label("Open Raw Permission Editor"),
                if module_config.disabled.unwrap_or_default() {
                    serenity::all::CreateButton::new("module/enable")
                        .style(serenity::all::ButtonStyle::Success)
                        .label("Enable Module")
                } else {
                    serenity::all::CreateButton::new("module/disable")
                        .style(serenity::all::ButtonStyle::Danger)
                        .label("Disable Module")
                },
                serenity::all::CreateButton::new("module/reset-toggle")
                    .style(serenity::all::ButtonStyle::Danger)
                    .label("Reset Module Toggle"),
                serenity::all::CreateButton::new("module/default-perms/reset")
                    .style(serenity::all::ButtonStyle::Danger)
                    .label("Reset Default Perms"),
                serenity::all::CreateButton::new("module/save")
                    .style(serenity::all::ButtonStyle::Secondary)
                    .label("Save Module Configuration"),
            ])])
    }

    let msg = ctx
        .send(module_config_to_edit_message(&new_module_config))
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
            "module/enable" => {
                if !module.toggleable {
                    item.create_response(
                        &ctx.serenity_context().http,
                        poise::serenity_prelude::CreateInteractionResponse::Message(
                            poise::CreateReply::new()
                                .content(format!(
                                    "The module `{}` cannot be toggled (enabled/disable) at this time!",
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

                let perm_res = silverpelt::cmd::check_command(
                    &data.silverpelt_cache,
                    "modules enable",
                    guild_id,
                    ctx.author().id,
                    &data.pool,
                    &cache_http,
                    &data.reqwest,
                    &Some(ctx),
                    silverpelt::cmd::CheckCommandOptions::default(),
                )
                .await;

                if !perm_res.is_ok() {
                    item.create_response(
                        &ctx.serenity_context().http,
                        poise::serenity_prelude::CreateInteractionResponse::Message(
                            poise::CreateReply::new()
                                .content(format!(
                                    "Enabling modules requires permission to use the ``modules enable`` command!\n{}",
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

                new_module_config.disabled = Some(false);
            }
            "module/disable" => {
                if !module.toggleable {
                    item.create_response(
                        &ctx.serenity_context().http,
                        poise::serenity_prelude::CreateInteractionResponse::Message(
                            poise::CreateReply::new()
                                .content(format!(
                                    "The module `{}` cannot be toggled (enabled/disable) at this time!",
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

                let perm_res = silverpelt::cmd::check_command(
                    &data.silverpelt_cache,
                    "modules disable",
                    guild_id,
                    ctx.author().id,
                    &data.pool,
                    &cache_http,
                    &data.reqwest,
                    &Some(ctx),
                    silverpelt::cmd::CheckCommandOptions::default(),
                )
                .await;

                if !perm_res.is_ok() {
                    item.create_response(
                        &ctx.serenity_context().http,
                        poise::serenity_prelude::CreateInteractionResponse::Message(
                            poise::CreateReply::new()
                                .content(format!(
                                    "Disabling modules requires permission to use the ``modules disable`` command!\n{}",
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

                new_module_config.disabled = Some(true);
            }
            "module/reset-toggle" => {
                if !module.toggleable {
                    item.create_response(
                        &ctx.serenity_context().http,
                        poise::serenity_prelude::CreateInteractionResponse::Message(
                            poise::CreateReply::new()
                                .content(format!(
                                    "The module `{}` cannot be toggled (enabled/disable) at this time!",
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

                if module.is_default_enabled {
                    let perm_res = silverpelt::cmd::check_command(
                        &data.silverpelt_cache,
                        "modules enable",
                        guild_id,
                        ctx.author().id,
                        &data.pool,
                        &cache_http,
                        &data.reqwest,
                        &Some(ctx),
                        silverpelt::cmd::CheckCommandOptions::default(),
                    )
                    .await;

                    if !perm_res.is_ok() {
                        item.create_response(
                                &ctx.serenity_context().http,
                                poise::serenity_prelude::CreateInteractionResponse::Message(
                                    poise::CreateReply::new()
                                        .content(format!(
                                            "Enabling modules requires permission to use the ``modules enable`` command!\n{}",
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
                    let perm_res = silverpelt::cmd::check_command(
                        &data.silverpelt_cache,
                        "modules disable",
                        guild_id,
                        ctx.author().id,
                        &data.pool,
                        &cache_http,
                        &data.reqwest,
                        &Some(ctx),
                        silverpelt::cmd::CheckCommandOptions::default(),
                    )
                    .await;

                    if !perm_res.is_ok() {
                        item.create_response(
                                &ctx.serenity_context().http,
                                poise::serenity_prelude::CreateInteractionResponse::Message(
                                    poise::CreateReply::new()
                                        .content(format!(
                                            "Disabling modules requires permission to use the ``modules disable`` command!\n{}",
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

                new_module_config.disabled = None;
            }
            "module/default-perms/reset" => {
                let perm_res = silverpelt::cmd::check_command(
                    &data.silverpelt_cache,
                    &format!("acl__{}_defaultperms_check", module.id),
                    guild_id,
                    ctx.author().id,
                    &data.pool,
                    &cache_http,
                    &data.reqwest,
                    &Some(ctx),
                    silverpelt::cmd::CheckCommandOptions {
                        custom_module_configuration: Some(GuildModuleConfiguration {
                            default_perms: None,
                            disabled: Some(false),
                            ..new_module_config.clone()
                        }),
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
                                    "You must have permission to use `acl__{}_defaultperms_check` with the permissions you have provided: {}",
                                    module.id,
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

                new_module_config.default_perms = None;
            }
            "default-perms/editraw" => {
                // Open a modal in response
                let perms = new_module_config.default_perms.clone().unwrap_or_default();

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

                let perms = serde_json::from_str::<PermissionChecks>(perms_str);

                match perms {
                    Ok(perms) => {
                        let parsed = silverpelt::validators::parse_permission_checks(
                            guild_id,
                            data.pool.clone(),
                            &perms,
                        )
                        .await?;

                        let perm_res = silverpelt::cmd::check_command(
                            &data.silverpelt_cache,
                            &format!("acl__{}_defaultperms_check", module.id),
                            guild_id,
                            ctx.author().id,
                            &data.pool,
                            &cache_http,
                            &data.reqwest,
                            &Some(ctx),
                            silverpelt::cmd::CheckCommandOptions {
                                custom_module_configuration: Some(GuildModuleConfiguration {
                                    disabled: Some(false),
                                    default_perms: Some(parsed.clone()),
                                    ..new_module_config.clone()
                                }),
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
                                            "You must have permission to use `acl__{}_defaultperms_check` with the permissions you have provided: {}",
                                            module.id,
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

                        new_module_config.default_perms = Some(parsed);
                    }
                    Err(err) => {
                        ctx.say(format!("Failed to parse permissions: {}", err))
                            .await?;
                        continue;
                    }
                }
            }
            "module/save" => {
                let mut tx = data.pool.begin().await?;

                // Check if guild module config exists now
                let count = sqlx::query!(
                    "SELECT COUNT(*) FROM guild_module_configurations WHERE guild_id = $1 AND module = $2",
                    guild_id.to_string(),
                    module.id
                )
                .fetch_one(&mut *tx)
                .await?
                .count
                .unwrap_or_default();

                let new_perms = serde_json::to_value(new_module_config.default_perms)?;

                if count > 0 {
                    sqlx::query!(
                        "UPDATE guild_module_configurations SET default_perms = $1, disabled = $2 WHERE guild_id = $3 AND module = $4",
                        new_perms,
                        new_module_config.disabled,
                        guild_id.to_string(),
                        module.id
                    )
                    .execute(&mut *tx)
                    .await?;
                } else {
                    sqlx::query!(
                        "INSERT INTO guild_module_configurations (guild_id, module, default_perms, disabled) VALUES ($1, $2, $3, $4)",
                        guild_id.to_string(),
                        module.id,
                        new_perms,
                        new_module_config.disabled
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
            module_config_to_edit_message(&new_module_config)
                .to_slash_initial_response_edit(serenity::all::EditInteractionResponse::default()),
        )
        .await?;
    }

    Ok(())
}

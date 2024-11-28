use log::{error, info};
use silverpelt::{data::Data, Context, Error};

/// Code to call on startup
pub async fn on_startup(ctx: serenity::all::Context) -> Result<(), crate::Error> {
    let data = ctx.data::<Data>();
    // For every GuildId with templates, fire a OnStartup event
    let guilds = sqlx::query!("SELECT guild_id FROM guild_templates")
        .fetch_all(&data.pool)
        .await?;

    for guild in guilds {
        let Ok(guild_id) = guild.guild_id.parse::<serenity::all::GuildId>() else {
            continue;
        };

        templating::cache::clear_cache(guild_id).await;

        match silverpelt::ar_event::dispatch_event_to_modules(
            &silverpelt::ar_event::EventHandlerContext {
                guild_id,
                data: data.clone(),
                event: silverpelt::ar_event::AntiraidEvent::OnStartup(vec![]),
                serenity_context: ctx.clone(),
            },
        )
        .await
        .map_err(|e| {
            format!("Failed to dispatch event: {}", {
                let mut strs = String::new();

                for err in e {
                    strs.push_str(&format!("{}\n", err));
                }

                strs
            })
        }) {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to dispatch OnStartup event: {}", e);
            }
        }
    }

    Ok(())
}

/// Standard error handler for Anti-Raid
pub async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Command { error, ctx, .. } => {
            error!("Error in command `{}`: {:?}", ctx.command().name, error,);

            let err = ctx
                .send(
                    poise::CreateReply::new()
                        .embed(
                            serenity::all::CreateEmbed::new()
                                .color(serenity::all::Color::RED)
                                .title("An error has occurred")
                                .description(error.to_string()),
                        )
                        .components(vec![serenity::all::CreateActionRow::Buttons(
                            vec![serenity::all::CreateButton::new_link(
                                &config::CONFIG.meta.support_server_invite,
                            )
                            .label("Support Server")]
                            .into(),
                        )]),
                )
                .await;

            if let Err(e) = err {
                error!("Message send error for FrameworkError::Command: {}", e);
            }
        }
        poise::FrameworkError::CommandCheckFailed { error, ctx, .. } => {
            error!(
                "[Possible] error in command `{}`: {:?}",
                ctx.command().qualified_name,
                error,
            );

            if let Some(error) = error {
                error!("Error in command `{}`: {:?}", ctx.command().name, error,);

                let err = ctx
                    .send(
                        poise::CreateReply::new()
                            .embed(
                                serenity::all::CreateEmbed::new()
                                    .color(serenity::all::Color::RED)
                                    .title("Command Check Failed")
                                    .description(error.to_string()),
                            )
                            .components(vec![serenity::all::CreateActionRow::Buttons(
                                vec![serenity::all::CreateButton::new_link(
                                    &config::CONFIG.meta.support_server_invite,
                                )
                                .label("Support Server")]
                                .into(),
                            )]),
                    )
                    .await;

                if let Err(e) = err {
                    error!(
                        "Message send error for FrameworkError::CommandCheckFailed: {}",
                        e
                    );
                }
            }
        }
        poise::FrameworkError::CommandPanic { payload, ctx, .. } => {
            error!(
                "Command `{}` panicked: {:?}",
                ctx.command().qualified_name,
                payload,
            );

            let err = ctx
                .send(
                    poise::CreateReply::new()
                    .embed(
                        serenity::all::CreateEmbed::new()
                            .color(serenity::all::Color::RED)
                            .title("Command Panic")
                            .description(format!("The command panicked. Please report this on our support server.\n\n```{}`", payload.unwrap_or("No payload provided".to_string()))),
                    )
                    .components(vec![serenity::all::CreateActionRow::Buttons(vec![
                        serenity::all::CreateButton::new_link(
                            &config::CONFIG.meta.support_server_invite,
                        )
                        .label("Support Server"),
                    ].into())]),
                )
                .await;

            if let Err(e) = err {
                error!("Message send error for FrameworkError::CommandPanic: {}", e);
            }
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                error!("Error while handling error: {}", e);
            }
        }
    }
}

pub fn setup_message<'a>() -> poise::CreateReply<'a> {
    poise::CreateReply::new()
    .embed(
        serenity::all::CreateEmbed::new()
        .title("Thank you for adding AntiRaid")
        .description(r#"While you have successfully added AntiRaid to your server, it won't do much until you take some time to configure it to your needs.

Please check out the `User Guide` and use the `Website` to tailor AntiRaid to the needs of your server! And, if you need help, feel free to join our `Support Server`!  

*Note: Feel free to rerun the command you were trying to run once you're content with your AntiRaid configuration*
        "#)
    )
    .components(
        vec![
            serenity::all::CreateActionRow::Buttons(
                vec![
                    serenity::all::CreateButton::new_link(
                        config::CONFIG.sites.docs.clone(),
                    )
                    .label("User Guide"),
                    serenity::all::CreateButton::new_link(
                        config::CONFIG.sites.frontend.clone(),
                    )
                    .label("Website"),
                    serenity::all::CreateButton::new_link(
                        config::CONFIG.meta.support_server_invite.clone(),
                    )
                    .label("Support Server")
                ].into()
            )
        ]
    )
}

pub async fn command_check(ctx: Context<'_>) -> Result<bool, Error> {
    let guild_id = ctx.guild_id();

    let Some(guild_id) = guild_id else {
        return Err("This command can only be run from servers".into());
    };

    let data = ctx.data();

    let guild_onboarding_status = sqlx::query!(
        "SELECT finished_onboarding FROM guilds WHERE id = $1",
        guild_id.to_string()
    )
    .fetch_optional(&data.pool)
    .await?;

    if let Some(guild_onboarding_status) = guild_onboarding_status {
        if !guild_onboarding_status.finished_onboarding {
            // Send setup message instead
            ctx.send(setup_message()).await?;

            // Set onboarding status to true
            sqlx::query!(
                "UPDATE guilds SET finished_onboarding = true WHERE id = $1",
                guild_id.to_string()
            )
            .execute(&data.pool)
            .await?;

            return Ok(false);
        }
    } else {
        // Guild not found, create it
        sqlx::query!(
            "INSERT INTO guilds (id, finished_onboarding) VALUES ($1, true)",
            guild_id.to_string()
        )
        .execute(&data.pool)
        .await?;

        // Send setup message instead
        ctx.send(setup_message()).await?;
        return Ok(false);
    }

    let user = sqlx::query!(
        "SELECT COUNT(*) FROM users WHERE user_id = $1",
        guild_id.to_string()
    )
    .fetch_one(&data.pool)
    .await?;

    if user.count.unwrap_or_default() == 0 {
        // User not found, create it
        sqlx::query!(
            "INSERT INTO users (user_id) VALUES ($1)",
            guild_id.to_string()
        )
        .execute(&data.pool)
        .await?;
    }

    let command = ctx.command();

    let res = permission_checks::check_command(
        &data.silverpelt_cache,
        &command.qualified_name,
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

    if res.is_ok() {
        return Ok(true);
    }

    ctx.send(
        poise::CreateReply::new().embed(
            serenity::all::CreateEmbed::new()
                .color(serenity::all::Color::RED)
                .title("You don't have permission to use this command?")
                .description(res.to_markdown())
                .field("Code", format!("`{}`", res.code()), false),
        ),
    )
    .await?;

    Ok(false)
}

pub fn get_commands(
    silverpelt_cache: &silverpelt::cache::SilverpeltCache,
) -> Vec<poise::Command<Data, Error>> {
    let mut cmds = Vec::new();

    let mut _cmd_names = Vec::new();
    for module in silverpelt_cache.module_cache.iter() {
        log::info!("Loading module {}", module.id());

        match module.validate() {
            Ok(_) => {}
            Err(e) => {
                panic!("Error validating module {}: {}", module.id(), e);
            }
        }

        if module.virtual_module() {
            continue;
        }

        for (mut cmd, extended_data) in module.raw_commands() {
            let root_is_virtual = match extended_data.get("") {
                Some(root) => root.virtual_command,
                None => false,
            };

            if root_is_virtual {
                continue;
            }

            cmd.category = Some(module.id().into());

            let mut subcommands = Vec::new();
            // Ensure subcommands are also linked to a category
            for subcommand in cmd.subcommands {
                let ext_data = extended_data.get(&*subcommand.name).unwrap_or_else(|| {
                    panic!("Subcommand {} does not have extended data", subcommand.name)
                });

                if ext_data.virtual_command {
                    continue;
                }

                subcommands.push(poise::Command {
                    category: Some(module.id().into()),
                    ..subcommand
                });
            }

            cmd.subcommands = subcommands;

            // Check for duplicate command names
            if _cmd_names.contains(&cmd.name) {
                error!("Duplicate command name: {:#?}", cmd);
                panic!("Duplicate command name: {}", cmd.qualified_name);
            }

            _cmd_names.push(cmd.name.clone());

            // Check for duplicate command aliases
            for alias in cmd.aliases.iter() {
                if _cmd_names.contains(alias) {
                    panic!(
                        "Duplicate command alias: {} from command {}",
                        alias, cmd.name
                    );
                }

                _cmd_names.push(alias.clone());
            }

            // Good to go
            cmds.push(cmd);
        }
    }

    cmds
}

pub fn get_tasks(ctx: &serenity::all::Context, data: &Data) -> Vec<botox::taskman::Task> {
    // Get all tasks
    let mut tasks = Vec::new();
    for module in data.silverpelt_cache.module_cache.iter() {
        let module = module.value();
        for (task, confirm_task) in module.background_tasks() {
            let (confirmed, reason) = (confirm_task)(ctx);
            if confirmed {
                info!(
                    "Adding task {} with confirm_task reason: {}",
                    task.name, reason
                );
            } else {
                info!(
                    "Skipping task {} as it is disabled for reason: {}",
                    task.name, reason
                );
                continue;
            }

            tasks.push(task);
        }
    }

    tasks
}

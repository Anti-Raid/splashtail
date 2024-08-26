use botox::cache::CacheHttpImpl;
use log::{error, info};
use silverpelt::{cache::SilverpeltCache, data::Data, Context, Error, EventHandlerContext, Module};
use std::sync::Arc;
use tokio::task::JoinSet;

/// Standard error handler for Anti-Raid
pub async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Command { error, ctx, .. } => {
            error!("Error in command `{}`: {:?}", ctx.command().name, error,);

            let err = ctx
                .send(
                    poise::CreateReply::new().embed(
                        serenity::all::CreateEmbed::new()
                            .color(serenity::all::Color::RED)
                            .title("An error has occurred")
                            .description(error.to_string()),
                    ),
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
                        poise::CreateReply::new().embed(
                            serenity::all::CreateEmbed::new()
                                .color(serenity::all::Color::RED)
                                .title("Command Check Failed")
                                .description(error.to_string()),
                        ),
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
                    poise::CreateReply::new().embed(
                        serenity::all::CreateEmbed::new()
                            .color(serenity::all::Color::RED)
                            .title("Command Panic")
                            .description(format!("The command panicked. Please report this to the bots owner.\n\n```{}`", payload.unwrap_or("No payload provided".to_string()))),
                    ),
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

pub async fn command_check(ctx: Context<'_>) -> Result<bool, Error> {
    let user_id = ctx.author().id;
    let guild_id = ctx.guild_id();

    let data = ctx.data();

    let allowed = data.props.is_whitelisted(guild_id, user_id).await?;

    if !allowed {
        // We already send in the event handler
        if let poise::Context::Application(_) = ctx {
            return Ok(false);
        }

        ctx.send(data.props.maint_message())
            .await
            .map_err(|e| format!("Error sending reply: {}", e))?;

        return Ok(false);
    }

    let Some(guild_id) = guild_id else {
        return Err("This command can only be run from servers".into());
    };

    let data = ctx.data();

    let guild = sqlx::query!(
        "SELECT COUNT(*) FROM guilds WHERE id = $1",
        guild_id.to_string()
    )
    .fetch_one(&data.pool)
    .await?;

    if guild.count.unwrap_or_default() == 0 {
        // Guild not found, create it
        sqlx::query!("INSERT INTO guilds (id) VALUES ($1)", guild_id.to_string())
            .execute(&data.pool)
            .await?;
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

    let res = silverpelt::cmd::check_command(
        &data.silverpelt_cache,
        &command.qualified_name,
        guild_id,
        ctx.author().id,
        &data.pool,
        &CacheHttpImpl::from_ctx(ctx.serenity_context()),
        &data.reqwest,
        &Some(ctx),
        silverpelt::cmd::CheckCommandOptions {
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

pub fn get_commands(modules: Vec<Module>) -> Vec<poise::Command<Data, Error>> {
    let mut cmds = Vec::new();

    let mut _cmd_names = Vec::new();
    for module in modules {
        log::info!("Loading module {}", module.id);

        if !module.is_parsed() {
            panic!("Module {} is not parsed", module.id);
        }

        if module.virtual_module {
            continue;
        }

        for (mut cmd, extended_data) in module.commands {
            let root_is_virtual = match extended_data.get("") {
                Some(root) => root.virtual_command,
                None => false,
            };

            if root_is_virtual {
                continue;
            }

            cmd.category = Some(module.id.to_string());

            let mut subcommands = Vec::new();
            // Ensure subcommands are also linked to a category
            for subcommand in cmd.subcommands {
                let ext_data = extended_data
                    .get(subcommand.name.as_str())
                    .unwrap_or_else(|| {
                        panic!("Subcommand {} does not have extended data", subcommand.name)
                    });

                if ext_data.virtual_command {
                    continue;
                }

                subcommands.push(poise::Command {
                    category: Some(module.id.to_string()),
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

pub fn get_tasks(modules: Vec<Module>, ctx: &serenity::all::Context) -> Vec<botox::taskman::Task> {
    // Get all tasks
    let mut tasks = Vec::new();
    for module in modules {
        for (task, confirm_task) in module.background_tasks {
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

pub async fn dispatch_event_to_modules(
    silverpelt_cache: &'static SilverpeltCache,
    event_handler_context: Arc<EventHandlerContext>,
) -> JoinSet<Result<(), Error>> {
    let mut set = JoinSet::new();

    for (id, module) in silverpelt_cache.module_cache.iter() {
        let module_enabled = match silverpelt::module_config::is_module_enabled(
            &event_handler_context.data.silverpelt_cache,
            &event_handler_context.data.pool,
            event_handler_context.guild_id,
            id,
        )
        .await
        {
            Ok(enabled) => enabled,
            Err(e) => {
                error!("Error getting module enabled status: {}", e);
                continue;
            }
        };

        if !module_enabled {
            continue;
        }

        for event_handler in module.event_handlers.iter() {
            let ehr = event_handler_context.clone();
            set.spawn(async move { event_handler(&ehr).await });
        }
    }

    set
}

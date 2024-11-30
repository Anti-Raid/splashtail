use futures_util::future::FutureExt;
use permissions::types::PermissionResult;
use silverpelt::Context;
use silverpelt::Error;

pub async fn filter(
    ctx: &Context<'_>,
    state: &HelpState,
    cmd: &poise::Command<silverpelt::data::Data, silverpelt::Error>,
) -> Result<bool, Error> {
    let Some(ref module) = cmd.category else {
        return Err("Internal error: command has no category".into());
    };

    // TODO: Actually handle checking command permissions
    if module == "root"
        && !config::CONFIG
            .discord_auth
            .root_users
            .contains(&ctx.author().id)
    {
        return Ok(false);
    }

    if state.filter_by_perms {
        let Some(guild_id) = ctx.guild_id() else {
            return Err("You must be in a guild to use ``filter_by_perms``".into());
        };

        let data = ctx.data();

        let res = permission_checks::check_command(
            &data.silverpelt_cache,
            &cmd.qualified_name,
            guild_id,
            ctx.author().id,
            &ctx.data().pool,
            ctx.serenity_context(),
            &data.reqwest,
            &Some(*ctx),
            permission_checks::CheckCommandOptions::default(),
        )
        .await;

        return match res {
            PermissionResult::Ok {} => Ok(true),
            PermissionResult::OkWithMessage { .. } => Ok(true),
            PermissionResult::DiscordError { error } => Err(error.into()),
            PermissionResult::GenericError { error } => Err(error.into()),
            _ => Ok(false),
        };
    }

    Ok(true)
}

#[derive(Default)]
pub struct HelpState {
    filter_by_perms: bool, // Slow, should only be enabled when needed
}

#[poise::command(slash_command)]
pub async fn help(
    ctx: Context<'_>,
    command: Option<String>,
    #[description = "Only show commands you have permission to use"] filter_by_perms: Option<bool>,
) -> Result<(), Error> {
    let data = ctx.data();
    let data = data.clone();
    botox::help::help(
        ctx,
        command,
        "%",
        botox::help::HelpOptions {
            get_category: Some(Box::new(move |category_id| {
                if let Some(cat_name) = category_id {
                    // Get the module from the name
                    let cat_module = data.silverpelt_cache.module_cache.get(&cat_name);

                    if let Some(cat_module) = cat_module {
                        Some(cat_module.name().to_string())
                    } else {
                        Some("Misc Commands".to_string())
                    }
                } else {
                    Some("Misc Commands".to_string())
                }
            })),
            state: HelpState {
                filter_by_perms: filter_by_perms.unwrap_or(false),
            },
            filter: Some(Box::new(move |ctx, state, cmd| {
                filter(ctx, state, cmd).boxed()
            })),
        },
    )
    .await
}

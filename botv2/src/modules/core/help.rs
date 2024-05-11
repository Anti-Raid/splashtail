use crate::Context;
use crate::Error;
use futures::future::FutureExt;

pub fn get_category(category_id: Option<String>) -> Option<String> {
    if let Some(cat_name) = category_id {
        // Get the module from the name
        let cat_module = crate::SILVERPELT_CACHE.module_id_cache.get(&cat_name);

        if let Some(cat_module) = cat_module {
            Some(cat_module.name.to_string())
        } else {
            Some("Misc Commands".to_string())
        }
    } else {
        Some("Misc Commands".to_string())
    }
}

pub async fn filter(
    ctx: &Context<'_>,
    state: &HelpState,
    cmd: &poise::Command<crate::Data, crate::Error>,
) -> Result<bool, Error> {
    let Some(ref module) = cmd.category else {
        return Err("Internal error: command has no category".into());
    };

    // TODO: Actually handle checking command permissions
    if module == "root"
        && !crate::config::CONFIG
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

        let res = crate::silverpelt::cmd::check_command(
            cmd.name.as_str(),
            &cmd.qualified_name,
            guild_id,
            ctx.author().id,
            &ctx.data().pool,
            &botox::cache::CacheHttpImpl::from_ctx(ctx.serenity_context()),
            &Some(*ctx),
            crate::silverpelt::cmd::CheckCommandOptions::default(),
        )
        .await;

        return match res {
            crate::silverpelt::permissions::PermissionResult::Ok {} => Ok(true),
            crate::silverpelt::permissions::PermissionResult::OkWithMessage { .. } => Ok(true),
            crate::silverpelt::permissions::PermissionResult::DiscordError { error } => {
                Err(error.into())
            }
            crate::silverpelt::permissions::PermissionResult::GenericError { error } => {
                Err(error.into())
            }
            _ => Ok(false),
        };
    }

    Ok(true)
}

#[derive(Default)]
pub struct HelpState {
    filter_by_perms: bool, // Slow, should only be enabled when needed
}

#[poise::command(track_edits, prefix_command, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    command: Option<String>,
    #[description = "Only show commands you have permission to use"] filter_by_perms: Option<bool>,
) -> Result<(), Error> {
    botox::help::help(
        ctx,
        command,
        "%",
        botox::help::HelpOptions {
            get_category: Some(Box::new(get_category)),
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

#[poise::command(category = "Help", prefix_command, slash_command, user_cooldown = 1)]
pub async fn simplehelp(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    botox::help::simplehelp(ctx, command).await
}

use crate::Context;
use crate::Error;

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

#[poise::command(track_edits, prefix_command, slash_command)]
pub async fn help(ctx: Context<'_>, command: Option<String>) -> Result<(), Error> {
    botox::help::help(
        ctx,
        command,
        "%",
        botox::help::HelpOptions {
            get_category: Some(Box::new(get_category)),
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

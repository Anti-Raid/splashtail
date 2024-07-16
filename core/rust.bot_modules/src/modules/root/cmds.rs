use crate::{Context, Error};
use splashcore_rs::value::Value;

#[poise::command(
    prefix_command,
    subcommands(
        "register",
        "cub",
        "inspector__fake_bots_list",
        "inspector__fake_bots_add",
        "inspector__fake_bots_update",
        "inspector__fake_bots_delete"
    )
)]
pub async fn sudo(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(prefix_command)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

/// Returns a list of users/guilds who can use the bot
#[poise::command(prefix_command)]
pub async fn cub(ctx: Context<'_>) -> Result<(), Error> {
    let rec = sqlx::query!("SELECT id, type, name, protected FROM can_use_bot")
        .fetch_all(&ctx.data().pool)
        .await?;

    let mut cub_list = "**Can Use Bot List**\n".to_string();

    for r in rec {
        cub_list.push_str(&format!(
            "Name: {}, Type: {}, ID: {}, Protected: {}\n",
            r.name, r.r#type, r.id, r.protected
        ));
    }

    cub_list.push_str("**Root Users**\n");
    for root_user in config::CONFIG.discord_auth.root_users.iter() {
        cub_list.push_str(&format!("- {} [<@{}>]\n", root_user, root_user));
    }

    ctx.say(cub_list).await?;

    Ok(())
}

#[poise::command(prefix_command)]
pub async fn inspector__fake_bots_list(ctx: Context<'_>) -> Result<(), crate::Error> {
    crate::silverpelt::settings_poise::settings_viewer(&ctx, &super::settings::INSPECTOR_FAKE_BOTS)
        .await
}

#[poise::command(prefix_command)]
pub async fn inspector__fake_bots_add(
    ctx: Context<'_>,
    #[description = "ID of the bot"] id: String,
    #[description = "Name of the bot"] name: String,
    #[description = "Official bot IDs, comma-seperated"] official_ids: String,
) -> Result<(), crate::Error> {
    crate::silverpelt::settings_poise::settings_creator(
        &ctx,
        &super::settings::INSPECTOR_FAKE_BOTS,
        indexmap::indexmap! {
            "bot_id".to_string() => Value::String(id),
            "name".to_string() => Value::String(name),
            "official_bot_ids".to_string() => {
                let official_ids = official_ids.split(',').map(|x| Value::String(x.trim().to_string())).collect::<Vec<Value>>();
                Value::List(official_ids)
            },
        },
    )
    .await
}

#[poise::command(prefix_command)]
pub async fn inspector__fake_bots_update(
    ctx: Context<'_>,
    #[description = "ID of the bot"] id: String,
    #[description = "Name of the bot"] name: String,
    #[description = "Official bot IDs, comma-seperated"] official_ids: String,
) -> Result<(), crate::Error> {
    crate::silverpelt::settings_poise::settings_updater(
        &ctx,
        &super::settings::INSPECTOR_FAKE_BOTS,
        indexmap::indexmap! {
            "bot_id".to_string() => Value::String(id),
            "name".to_string() => Value::String(name),
            "official_bot_ids".to_string() => {
                let official_ids = official_ids.split(',').map(|x| Value::String(x.trim().to_string())).collect::<Vec<Value>>();
                Value::List(official_ids)
            },
        },
    )
    .await
}

#[poise::command(prefix_command)]
pub async fn inspector__fake_bots_delete(
    ctx: Context<'_>,
    #[description = "ID of the bot"] id: String,
) -> Result<(), crate::Error> {
    crate::silverpelt::settings_poise::settings_deleter(
        &ctx,
        &super::settings::INSPECTOR_FAKE_BOTS,
        Value::String(id),
    )
    .await
}

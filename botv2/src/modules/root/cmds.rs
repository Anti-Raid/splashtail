use crate::{Context, Error};

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

    let mut cub_list = "**Can Use Bot List**".to_string();

    for r in rec {
        cub_list.push_str(&format!(
            "Name: {}, Type: {}, ID: {}, Protected: {}\n",
            r.name, r.r#type, r.id, r.protected
        ));
    }

    cub_list.push_str("**Root Users**\n");
    for root_user in crate::config::CONFIG.discord_auth.root_users.iter() {
        cub_list.push_str(&format!("- {} [<@{}>]", root_user, root_user));
    }

    ctx.say(cub_list).await?;

    Ok(())
}

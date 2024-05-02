use crate::{Context, Error};
use splashcore_rs::utils::{parse_numeric_list, REPLACE_ROLE, REPLACE_USER};

/// Punishment list  base command
#[poise::command(prefix_command, slash_command, subcommands("punishments_add"))]
pub async fn punishments(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Adds a common/standard punishment
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = "5",
    rename = "add"
)]
#[allow(clippy::too_many_arguments)]
pub async fn punishments_add(
    ctx: Context<'_>,
    #[description = "The number of stings that must be reached"] stings: i32,
    #[description = "What action to take"] action: super::core::ActionsChoices,
    #[description = "Ignored Roles, comma seperated"] ignored_roles: Option<String>,
    #[description = "Ignored Users, comma seperated"] ignored_users: Option<String>,
    #[description = "Specify custom modifiers, this is an advanced feature"] modifiers: Option<String>,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command can only be used in a guild".into());
    };

    let modifiers_str = modifiers.unwrap_or_default();
    let mut modifiers = vec![];

    for m in modifiers_str.split(',') {
        modifiers.push(m.trim().to_string());
    }

    if let Some(ignored_roles) = ignored_roles {
        let ignored_roles = parse_numeric_list::<serenity::all::RoleId>(&ignored_roles, &REPLACE_ROLE)?;

        for role in ignored_roles {
            // Role modifiers are in format -r:role_id
            let modifier = format!("-r:{}", role);
            
            if !modifiers.contains(&modifier) {
                modifiers.push(modifier);
            }
        }
    }

    if let Some(ignored_users) = ignored_users {
        let ignored_users = parse_numeric_list::<serenity::all::UserId>(&ignored_users, &REPLACE_USER)?;

        for user in ignored_users {
            // User modifiers are in format -u:user_id
            let modifier = format!("-u:{}", user);
            
            if !modifiers.contains(&modifier) {
                modifiers.push(modifier);
            }
        }
    }

    let action = action.resolve();

    let data = ctx.data();

    sqlx::query!(
        "INSERT INTO punishments__guild_punishment_list (guild_id, creator, stings, action, modifiers) VALUES ($1, $2, $3, $4, $5)",
        guild_id.to_string(),
        ctx.author().id.to_string(),
        stings,
        action.to_string(),
        &modifiers,
    )
    .execute(&data.pool)
    .await?;

    ctx.say("Punishment added").await?;
    Ok(())
}
use std::str::FromStr;

use crate::{Context, Error};
use splashcore_rs::utils::{
    parse_duration_string, parse_numeric_list, parse_pg_interval, REPLACE_ROLE, REPLACE_USER,
};

/// Punishment list  base command
#[poise::command(
    prefix_command,
    slash_command,
    subcommands(
        "punishments_add",
        "punishments_viewsources",
        "punishments_list",
        "punishments_delete"
    )
)]
pub async fn punishments(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// List all sources that stings can come from
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = "5",
    rename = "viewsources"
)]
#[allow(clippy::too_many_arguments)]
pub async fn punishments_viewsources(ctx: Context<'_>) -> Result<(), Error> {
    let mut embed = serenity::all::CreateEmbed::new();

    embed = embed.title("Sting Sources");

    for source in super::sting_source::STING_SOURCES.iter() {
        let id = source.key();
        let source = source.value();
        embed = embed.field(
            id.clone(),
            format!("{} {}", source.id, source.description),
            false,
        );
    }

    ctx.send(poise::CreateReply::new().embed(embed)).await?;

    Ok(())
}

/// List all punishments
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = "5",
    rename = "list"
)]
pub async fn punishments_list(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("This command can only be used in a guild")?;

    let data = ctx.data();

    let punishments = sqlx::query!(
        "SELECT id, creator, stings, action, modifiers, created_at, duration FROM punishments__guild_punishment_list WHERE guild_id = $1",
        guild_id.to_string(),
    )
    .fetch_all(&data.pool)
    .await?;

    let mut embeds = Vec::new();
    let mut embed = serenity::all::CreateEmbed::default();

    embed = embed.title("Punishments");

    for (fields, punishment) in punishments.iter().enumerate() {
        if fields > 5 {
            embeds.push(embed);
            embed = serenity::all::CreateEmbed::default();
        }

        let action = super::core::Action::from_str(punishment.action.as_str())
            .unwrap_or(super::core::Action::Unknown);

        embed = embed.field(
            format!("At {} stings...", punishment.stings),
            format!(
                "Action: {}\nModifiers: {}\nCreator: <@{}>\nCreated At: <t:{}:R>\nID: {}",
                {
                    if let Some(ref duration) = punishment.duration {
                        format!("{} for {}", action, parse_pg_interval(duration.clone()))
                    } else {
                        format!("{}", action)
                    }
                },
                punishment.modifiers.join(", "),
                punishment.creator,
                punishment.created_at.timestamp(),
                punishment.id,
            ),
            false,
        );
    }

    let mut cr = poise::CreateReply::new();

    for e in embeds {
        cr = cr.embed(e);
    }

    cr = cr.embed(embed);

    ctx.send(cr).await?;

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
    #[description = "What action to take"] action: super::core::ActionChoices,
    #[description = "How long should the punishment be (timeout only)"] duration: Option<String>,
    #[description = "Ignored Roles, comma seperated"] ignored_roles: Option<String>,
    #[description = "Ignored Users, comma seperated"] ignored_users: Option<String>,
    #[description = "Specify custom modifiers, this is an advanced feature"] modifiers: Option<
        String,
    >,
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
        let ignored_roles =
            parse_numeric_list::<serenity::all::RoleId>(&ignored_roles, &REPLACE_ROLE)?;

        for role in ignored_roles {
            // Role modifiers are in format -r:role_id
            let modifier = format!("-r:{}", role);

            if !modifiers.contains(&modifier) {
                modifiers.push(modifier);
            }
        }
    }

    if let Some(ignored_users) = ignored_users {
        let ignored_users =
            parse_numeric_list::<serenity::all::UserId>(&ignored_users, &REPLACE_USER)?;

        for user in ignored_users {
            // User modifiers are in format -u:user_id
            let modifier = format!("-u:{}", user);

            if !modifiers.contains(&modifier) {
                modifiers.push(modifier);
            }
        }
    }

    let action = action.resolve();

    let duration = if let Some(duration) = duration {
        let (duration, unit) = parse_duration_string(&duration)?;
        Some((duration * unit.to_seconds()) as f64)
    } else {
        None
    };

    let data = ctx.data();

    sqlx::query!(
        "INSERT INTO punishments__guild_punishment_list (guild_id, creator, stings, action, duration, modifiers) VALUES ($1, $2, $3, $4, make_interval(secs => $5), $6)",
        guild_id.to_string(),
        ctx.author().id.to_string(),
        stings,
        action.to_string(),
        duration,
        &modifiers,
    )
    .execute(&data.pool)
    .await?;

    ctx.say("Punishment added").await?;
    Ok(())
}

pub async fn punishment_delete_autocomplete<'a>(
    ctx: crate::Context<'_>,
    partial: &'a str,
) -> Vec<serenity::all::AutocompleteChoice<'a>> {
    let data = ctx.data();

    let guild_id = ctx.guild_id();

    if guild_id.is_none() {
        return Vec::new();
    }

    let guild_id = guild_id.unwrap();

    let punishments = match sqlx::query!(
        "SELECT id, stings, action FROM punishments__guild_punishment_list WHERE guild_id = $1",
        guild_id.to_string(),
    )
    .fetch_all(&data.pool)
    .await
    {
        Ok(punishments) => punishments,
        Err(e) => {
            log::error!("Failed to fetch punishments: {}", e);
            return Vec::new();
        }
    };

    let mut choices = Vec::new();

    for punishment in punishments {
        let action = super::core::Action::from_str(punishment.action.as_str())
            .unwrap_or(super::core::Action::Unknown);
        let name = format!("At, {} stings, {}", punishment.stings, action);
        if name.contains(partial) {
            choices.push(serenity::all::AutocompleteChoice::new(
                name,
                punishment.id.to_string(),
            ));
        }
    }

    choices
}

/// Deletes a punishment
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = "5",
    rename = "delete"
)]
pub async fn punishments_delete(
    ctx: Context<'_>,
    #[description = "The ID of the punishment to delete"]
    #[autocomplete = "punishment_delete_autocomplete"]
    id: String,
) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("This command can only be used in a guild")?;

    let data = ctx.data();

    let id = id.parse::<sqlx::types::Uuid>()?;

    let mut tx = data.pool.begin().await?;

    let count = sqlx::query!(
        "SELECT COUNT(*) FROM punishments__guild_punishment_list WHERE guild_id = $1 AND id = $2",
        guild_id.to_string(),
        id,
    )
    .fetch_one(&mut *tx)
    .await?
    .count
    .unwrap_or_default();

    if count == 0 {
        return Err("Punishment not found".into());
    }

    sqlx::query!(
        "DELETE FROM punishments__guild_punishment_list WHERE guild_id = $1 AND id = $2",
        guild_id.to_string(),
        id,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    ctx.say("Punishment deleted").await?;
    Ok(())
}

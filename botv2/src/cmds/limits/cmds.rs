use poise::{
    serenity_prelude::CreateEmbed,
    CreateReply,
};
use serenity::{all::{UserId, Mentionable}, builder::CreateAttachment};

use crate::{Context, Error};

/// Limits base command
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    subcommands("limits_add", "limits_view", "limits_remove", "limits_hit")
)]
pub async fn limits(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Add a limit to the server
#[poise::command(prefix_command, slash_command, guild_only, rename = "add")]
pub async fn limits_add(
    ctx: Context<'_>,
    #[description = "The name of the limit"]
    limit_name: String,
    #[description = "The type of limit to impose on moderators"]
    limit_type: crate::modules::limits::core::UserLimitTypesChoices,
    #[description = "The amount of times the limit can be hit"]
    limit_per: i32,
    #[description = "The time interval infractions are counted in"]
    limit_time: i64,
    #[description = "The time unit for the time interval [seconds/minutes/hours/days]"]
    limit_time_unit: crate::impls::utils::Unit,
    #[description = "The action to take when the limit is hit"]
    limit_action: crate::modules::limits::core::UserLimitActionsChoices,
) -> Result<(), Error> {
    let limit_type = limit_type.resolve();
    let limit_action = limit_action.resolve();

    // Add limit to db
    sqlx::query!(
        "
            INSERT INTO limits (
                guild_id,
                limit_name,
                limit_type,
                limit_action,
                limit_per,
                limit_time
            )
            VALUES (
                $1, 
                $2, 
                $3, 
                $4, 
                $5,
                make_interval(secs => $6)
            )
        ",
        ctx.guild_id().ok_or("Could not get guild id")?.to_string(),
        limit_name,
        limit_type.to_string(),
        limit_action.to_string(),
        limit_per,
        (limit_time * limit_time_unit.to_seconds()) as f64
    )
    .execute(&ctx.data().pool)
    .await?;

    ctx.say("Added limit successfully").await?;

    Ok(())
}

/// View the limits setup for this server
#[poise::command(prefix_command, slash_command, guild_only, rename = "view")]
pub async fn limits_view(ctx: Context<'_>) -> Result<(), Error> {
    let limits = crate::modules::limits::core::Limit::from_guild(
        &ctx.data().pool,
        ctx.guild_id().ok_or("Could not get guild id")?,
    )
    .await?;

    if limits.is_empty() {
        ctx.say("No limits setup for this server, use ``/limits add`` to add one!")
            .await?;
        return Ok(());
    }

    let mut embeds = vec![];

    let mut added: i32 = 0;
    let mut i = 0;

    for limit in limits {
        added += 1;

        if added >= 15 {
            added = 0;
            i += 1;
        }

        if embeds.len() <= i {
            embeds.push(CreateEmbed::default().title("Limits").color(0x00ff00));
        }

        embeds[i] = embeds[i].clone().field(
            limit.limit_name,
            format!(
                "If over {amount} ``{cond}`` triggered between {time} interval: ``{then}`` [{id}]",
                amount = limit.limit_per,
                cond = limit.limit_type.to_cond(),
                time = crate::impls::utils::parse_pg_interval(limit.limit_time),
                then = limit.limit_action.to_cond(),
                id = limit.limit_id
            ),
            false,
        );
    }

    let reply = CreateReply {
        embeds,
        ..Default::default()
    };

    ctx.send(reply).await?;

    Ok(())
}

/// Remove a limit from the server
#[poise::command(prefix_command, slash_command, guild_only, rename = "remove")]
pub async fn limits_remove(
    ctx: Context<'_>,
    #[description = "The limit id to remove"]
    #[autocomplete = "super::autocompletes::limits_autocomplete"]
    limit_id: String,
) -> Result<(), Error> {
    // Look for limit using COUNT
    let count = sqlx::query!(
        "
            SELECT COUNT(*) FROM limits
            WHERE guild_id = $1
            AND limit_id = $2
        ",
        ctx.guild_id().ok_or("Could not get guild id")?.to_string(),
        limit_id
    )
    .fetch_one(&ctx.data().pool)
    .await?;

    if count.count.unwrap_or_default() == 0 {
        return Err("Could not find limit".into());
    }

    // Remove limit
    sqlx::query!(
        "
            DELETE FROM limits
            WHERE guild_id = $1
            AND limit_id = $2
        ",
        ctx.guild_id().ok_or("Could not get guild id")?.to_string(),
        limit_id
    )
    .execute(&ctx.data().pool)
    .await?;

    ctx.say("Removed limit successfully").await?;

    Ok(())
}

/// Action management
#[poise::command(prefix_command, slash_command, guild_only, subcommands("limitactions_view"))]
pub async fn limitactions(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// View actions taken by users that have been recorded
#[poise::command(prefix_command, slash_command, guild_only, rename = "view")]
pub async fn limitactions_view(
    ctx: Context<'_>,
    #[description = "User ID (optional)"] user_id: Option<UserId>,
) -> Result<(), Error> {
    let actions = {
        if let Some(user_id) = user_id {
            crate::modules::limits::core::Action::user(
                &ctx.data().pool,
                ctx.guild_id().ok_or("Could not get guild id")?,
                user_id,
            )
            .await?
        } else {
            crate::modules::limits::core::Action::guild(
                &ctx.data().pool,
                ctx.guild_id().ok_or("Could not get guild id")?,
            )
            .await?
        }
    };

    if actions.is_empty() {
        ctx.say("No actions recorded").await?;
        return Ok(());
    }

    if actions.len() > 64 {
        let actions = serde_json::to_string(&actions).map_err(|_| "Could not serialize actions")?;

        // Create a attachment
        let attachment = CreateAttachment::bytes(actions.into_bytes(), "actions.json");

        ctx.send(CreateReply::default().attachment(attachment)).await?;

        return Ok(());
    }

    let mut embeds = vec![];
    let mut added: i32 = 0;
    let mut i = 0;

    for action in actions {
        added += 1;

        if added >= 8 {
            added = 0;
            i += 1;
        }

        if embeds.len() <= i {
            embeds.push(CreateEmbed::default().title("Actions").color(0x00ff00));
        }

        embeds[i] = embeds[i].clone().field(
            action.action_id.clone(),
            format!(
                "``{limit_type}`` on ``{action_target}`` by {user_id} at <t:{timestamp}:R> [{id}]\n**Hit Limits:** {limits_hit:#?}",
                limit_type = action.limit_type,
                action_target = action.action_target,
                user_id = action.user_id.mention().to_string() + " (" + &action.user_id.to_string() + ")",
                timestamp = action.created_at.timestamp(),
                id = action.action_id,
                limits_hit = action.limits_hit
            ),
            false,
        );
    }

    let reply = CreateReply {
        embeds,
        ..Default::default()    
    };

    ctx.send(reply).await?;

    Ok(())
}

/// View hit limits
#[poise::command(prefix_command, slash_command, guild_only, rename = "hit")]
pub async fn limits_hit(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let hit_limits = crate::modules::limits::core::PastHitLimits::guild(
        &ctx.data().pool,
        ctx.guild_id().ok_or("Could not get guild id")?,
    ).await?;        

    if hit_limits.is_empty() {
        ctx.say("No hit limits recorded").await?;
        return Ok(());
    }

    if hit_limits.len() > 64 {
        let hit_limits = serde_json::to_string(&hit_limits).map_err(|_| "Could not serialize hit_limits")?;

        // Create a attachment
        let attachment = CreateAttachment::bytes(hit_limits.into_bytes(), "hit_limits.json");

        ctx.send(CreateReply::default().attachment(attachment)).await?;

        return Ok(());
    }

    let mut embeds = vec![];

    let mut added: i32 = 0;
    let mut i = 0;

    for hit_limit in hit_limits {
        added += 1;

        if added >= 8 {
            added = 0;
            i += 1;
        }

        if embeds.len() <= i {
            embeds.push(CreateEmbed::default().title("Past Limits History").color(0x00ff00));
        }

        let mut notes = String::new();

        for note in hit_limit.notes {
            notes.push_str(&format!("- ``{}`` ", note));
        }

        let mut causes = String::new();

        for cause in hit_limit.cause {
            causes.push_str(
                &format!(
                    "``{limit_type}`` on ``{action_target}`` by {user_id} at <t:{timestamp}:R> [{id}]\n**Hit Limits:** {limits_hit:#?}",
                    limit_type = cause.limit_type,
                    action_target = cause.action_target,
                    user_id = cause.user_id.mention().to_string() + " (" + &cause.user_id.to_string() + ")",
                    timestamp = cause.created_at.timestamp(),
                    id = cause.action_id,
                    limits_hit = cause.limits_hit
                ),
            );
        }

        embeds[i] = embeds[i].clone().field(
            hit_limit.id.clone(),
            format!(
                "Limit ``{limit_id}`` reached by ``{user_id}`` at <t:{timestamp}:R> [{id}]\n**Notes:** {notes}\n**Causes:** {causes}",
                limit_id = hit_limit.limit_id,
                user_id = hit_limit.user_id.mention().to_string() + " (" + &hit_limit.user_id.to_string() + ")",
                timestamp = hit_limit.created_at.timestamp(),
                id = hit_limit.id,
                notes = notes,
                causes = causes
            ),
            false,
        );
    }

    let reply = CreateReply {
        embeds,
        ..Default::default()
    };

    ctx.send(reply).await?;


    Ok(())
}
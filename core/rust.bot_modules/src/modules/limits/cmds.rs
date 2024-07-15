use crate::{Context, Error};
use poise::{serenity_prelude::CreateEmbed, CreateReply};
use serenity::{
    all::{Mentionable, UserId},
    builder::CreateAttachment,
};
use splashcore_rs::value::Value;

/// Limits base command
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    subcommands(
        "limits_view",
        "limits_add",
        "limits_update",
        "limits_remove",
        "limits_hit"
    )
)]
pub async fn limits(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// View the limits setup for this server
#[poise::command(prefix_command, slash_command, guild_only, rename = "view")]
pub async fn limits_view(ctx: Context<'_>) -> Result<(), Error> {
    crate::silverpelt::settings_poise::settings_viewer(&ctx, &super::settings::GUILD_LIMITS).await
}

/// Add a limit
#[poise::command(prefix_command, slash_command, guild_only, rename = "add")]
pub async fn limits_add(
    ctx: Context<'_>,
    #[description = "The name of the limit"] limit_name: String,
    #[description = "The type of limit to impose on moderators"]
    limit_type: super::core::LimitTypesChoices,
    #[description = "The amount of times the limit can be hit"] limit_per: i32,
    #[description = "The time interval infractions are counted in"] limit_time: i64,
    #[description = "The time unit for the time interval [seconds/minutes/hours/days]"]
    limit_time_unit: splashcore_rs::utils::Unit,
    #[description = "The number of stings to give on hitting the limit"] stings: i32,
) -> Result<(), Error> {
    crate::silverpelt::settings_poise::settings_creator(
        &ctx,
        &super::settings::GUILD_LIMITS,
        indexmap::indexmap! {
            "limit_name".to_string() => Value::String(limit_name),
            "limit_type".to_string() => Value::String(limit_type.resolve().to_string()),
            "limit_per".to_string() => Value::Integer(limit_per.into()),
            "limit_time".to_string() => Value::Interval(chrono::Duration::seconds(limit_time * limit_time_unit.to_seconds_i64())),
            "stings".to_string() => Value::Integer(stings.into()),
        },
    )
    .await
}

/// Update an existing limit
#[poise::command(prefix_command, slash_command, guild_only, rename = "update")]
#[allow(clippy::too_many_arguments)]
pub async fn limits_update(
    ctx: Context<'_>,
    #[description = "The ID of the limit"] limit_id: String,
    #[description = "The name of the limit"] limit_name: String,
    #[description = "The type of limit to impose on moderators"]
    limit_type: super::core::LimitTypesChoices,
    #[description = "The amount of times the limit can be hit"] limit_per: i32,
    #[description = "The time interval infractions are counted in"] limit_time: i64,
    #[description = "The time unit for the time interval [seconds/minutes/hours/days]"]
    limit_time_unit: splashcore_rs::utils::Unit,
    #[description = "The number of stings to give on hitting the limit"] stings: i32,
) -> Result<(), Error> {
    crate::silverpelt::settings_poise::settings_updater(
        &ctx,
        &super::settings::GUILD_LIMITS,
        indexmap::indexmap! {
            "limit_id".to_string() => Value::String(limit_id),
            "limit_name".to_string() => Value::String(limit_name),
            "limit_type".to_string() => Value::String(limit_type.resolve().to_string()),
            "limit_per".to_string() => Value::Integer(limit_per.into()),
            "limit_time".to_string() => Value::Interval(chrono::Duration::seconds(limit_time * limit_time_unit.to_seconds_i64())),
            "stings".to_string() => Value::Integer(stings.into()),
        },
    )
    .await
}

/// Remove a limit from the server
#[poise::command(prefix_command, slash_command, guild_only, rename = "remove")]
pub async fn limits_remove(
    ctx: Context<'_>,
    #[description = "The limit id to remove"]
    #[autocomplete = "super::autocompletes::limits_autocomplete"]
    limit_id: String,
) -> Result<(), Error> {
    crate::silverpelt::settings_poise::settings_deleter(
        &ctx,
        &super::settings::GUILD_LIMITS,
        Value::String(limit_id),
    )
    .await
}

/// Action management
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    subcommands("limitactions_view")
)]
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
            super::core::UserAction::user(
                &ctx.data(),
                ctx.guild_id().ok_or("Could not get guild id")?,
                user_id,
            )
            .await?
        } else {
            super::core::UserAction::guild(
                &ctx.data(),
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

        ctx.send(CreateReply::default().attachment(attachment))
            .await?;

        return Ok(());
    }

    let mut embeds = vec![];
    let mut added: i32 = 0;
    let mut i = 0;

    // TODO: Support sending past hit limits pertaining to action ids in attachments as well
    let past_hit_limits = super::core::PastHitLimits::guild(
        &ctx.data(),
        ctx.guild_id().ok_or("Could not get guild id")?,
    )
    .await?;

    for action in actions {
        added += 1;

        if added >= 8 {
            added = 0;
            i += 1;
        }

        if embeds.len() <= i {
            embeds.push(CreateEmbed::default().title("Actions").color(0x00ff00));
        }

        let action_id = action.action_id;

        let target = if let Some(target) = action.target {
            target
        } else {
            "None".to_string()
        };

        let mut limits_hit = vec![];
        for past_hit_limit in &past_hit_limits {
            if past_hit_limit.cause.iter().any(|cause| *cause == action_id) {
                limits_hit.push(past_hit_limit.id.clone());
            }
        }

        embeds[i] = embeds[i].clone().field(
            action_id.clone(),
            format!(
                "``{limit_type}`` by {user_id} on {target} at <t:{timestamp}:R> (for {no_stings} stings) | {action_data} [{id}]\n**Hit Limits:** {limits_hit:#?}",
                limit_type = action.limit_type,
                action_data = serde_json::to_string(&action.action_data).map_err(|_| "Could not serialize action_data")?,
                user_id = action.user_id.mention().to_string() + " (" + &action.user_id.to_string() + ")",
                target = target,
                timestamp = action.created_at.timestamp(),
                no_stings = action.stings,
                id = action_id,
                limits_hit = limits_hit
            ),
            false,
        );
    }

    let mut reply = CreateReply::new();

    for embed in embeds {
        reply = reply.embed(embed);
    }

    ctx.send(reply).await?;

    Ok(())
}

/// View hit limits
#[poise::command(prefix_command, slash_command, guild_only, rename = "hit")]
pub async fn limits_hit(ctx: Context<'_>) -> Result<(), Error> {
    let hit_limits = super::core::PastHitLimits::guild(
        &ctx.data(),
        ctx.guild_id().ok_or("Could not get guild id")?,
    )
    .await?;

    if hit_limits.is_empty() {
        ctx.say("No hit limits recorded").await?;
        return Ok(());
    }

    if hit_limits.len() > 64 {
        let hit_limits =
            serde_json::to_string(&hit_limits).map_err(|_| "Could not serialize hit_limits")?;

        // Create a attachment
        let attachment = CreateAttachment::bytes(hit_limits.into_bytes(), "hit_limits.json");

        ctx.send(CreateReply::default().attachment(attachment))
            .await?;

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
            embeds.push(
                CreateEmbed::default()
                    .title("Past Limits History")
                    .color(0x00ff00),
            );
        }

        let mut notes = String::new();

        for note in hit_limit.notes {
            notes.push_str(&format!("- ``{}`` ", note));
        }

        let mut causes = String::new();

        for cause in hit_limit.cause {
            let cause = super::core::UserAction::by_id(
                &ctx.data(),
                ctx.guild_id().ok_or("Could not get guild id")?,
                &cause,
            )
            .await?;

            causes.push_str(&format!(
                "``{limit_type}`` by {user_id} at <t:{timestamp}:R> [{id}] | {action_data}",
                limit_type = cause.limit_type,
                action_data = serde_json::to_string(&cause.action_data)
                    .map_err(|_| "Could not serialize action_data")?,
                user_id =
                    cause.user_id.mention().to_string() + " (" + &cause.user_id.to_string() + ")",
                timestamp = cause.created_at.timestamp(),
                id = cause.action_id,
            ));
        }

        embeds[i] = embeds[i].clone().field(
            hit_limit.id.clone(),
            format!(
                "Limits ``{limit_ids}`` reached by ``{user_id}`` at <t:{timestamp}:R> [{id}]\n**Notes:** {notes}\n**Causes:** {causes}",
                limit_ids = hit_limit.limit_ids.join(", "),
                user_id = hit_limit.user_id.mention().to_string() + " (" + &hit_limit.user_id.to_string() + ")",
                timestamp = hit_limit.created_at.timestamp(),
                id = hit_limit.id,
                notes = notes,
                causes = causes
            ),
            false,
        );
    }

    let mut reply = CreateReply::new();

    for embed in embeds {
        reply = reply.embed(embed);
    }

    ctx.send(reply).await?;

    Ok(())
}

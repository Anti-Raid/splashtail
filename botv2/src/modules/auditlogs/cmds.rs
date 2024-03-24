use crate::{Context, Error};
use serenity::all::{Channel, ChannelType};
use splashcore_rs::crypto::gen_random;

#[poise::command(prefix_command, slash_command, user_cooldown = 1, subcommands("add_channel", "add_discordhook"))]
pub async fn auditlogs(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(prefix_command, slash_command, user_cooldown = 1)]
pub async fn add_channel(
    ctx: Context<'_>,
    #[description = "Channel to send logs to"] channel: Channel,
    #[description = "Specific events you want to filter by"] events: Option<String>,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command can only be used in a guild".into());
    };

    let gc = match channel {
        Channel::Guild(c) => c,
        _ => return Err("The channel you have selected appears to be from a Direct Message (DM). This is NOT supported at this time".into()),
    };

    match gc.kind {
        ChannelType::Forum => return Err("Cannot use a Forum channel for audit logs. Try making a thread and using the thread (or use a different channel)".into()),
        ChannelType::Category => return Err("Cannot use a Category channel for audit logs. Try using a text or voice channel".into()),
        _ => {}
    };

    // Check if we have permissions in said channel
    let user_perms = gc.permissions_for_user(ctx.cache(), ctx.author().id)?;

    if !user_perms.view_channel() {
        return Err("You do not have permission to view this channel".into());
    }

    let bot_perms = gc.permissions_for_user(ctx.cache(), ctx.cache().current_user().id)?;

    if !bot_perms.view_channel() {
        return Err("I do not have permission to view this channel".into());
    }
    if !bot_perms.send_messages() {
        return Err("I do not have permission to send messages in this channel".into());
    }
    if !bot_perms.embed_links() {
        return Err("I do not have permission to embed links in this channel".into());
    }
    if !bot_perms.read_message_history() {
        return Err("I do not have permission to read message history in this channel".into());
    }
    if !bot_perms.manage_messages() {
        return Err("I do not have permission to manage messages in this channel".into());
    }

    // Find the value in gwevents::core::event_list
    let mut supported_events = Vec::new();

    for event in gwevent::core::event_list() {
        if super::events::not_audit_loggable_event().contains(event) {
            continue;
        }

        supported_events.push(event.to_string());
    }

    let events_split = if let Some(events) = events {
        let mut events_vec = Vec::new();

        for event in events.split(',') {
            let trimmed = event.trim().to_string();

            if trimmed.is_empty() {
                continue;
            }

            let event = trimmed.to_uppercase();

            if !supported_events.contains(&event) {
                return Err(format!("Event `{}` is not a valid event. Please pick one of the following: {}", trimmed, supported_events.join(", ")).into());
            }

            events_vec.push(event);
        }

        Some(events_vec)
    } else {
        None
    };

    let sink_id = gen_random(24);
    sqlx::query!(
        "INSERT INTO auditlogs__sinks (id, guild_id, type, sink, events, broken, created_by, last_updated_by) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        sink_id,
        guild_id.to_string(),
        "channel",
        gc.id.to_string(),
        events_split.as_deref(),
        false,
        ctx.author().id.to_string(),
        ctx.author().id.to_string(),
    )
    .execute(&ctx.data().pool)
    .await?;

    Ok(())
}

#[poise::command(prefix_command, slash_command, user_cooldown = 1)]
pub async fn add_discordhook(
    ctx: Context<'_>,
    #[description = "Webhook URL to send logs to"] webhook: String,
    #[description = "Specific events you want to filter by"] events: Option<String>,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command can only be used in a guild".into());
    };

    if serenity::utils::parse_webhook(&webhook.parse()?).is_none() {
        return Err("Invalid webhook URL".into());
    }
    
    // Find the value in gwevents::core::event_list
    let mut supported_events = Vec::new();

    for event in gwevent::core::event_list() {
        if super::events::not_audit_loggable_event().contains(event) {
            continue;
        }

        supported_events.push(event.to_string());
    }

    let events_split = if let Some(events) = events {
        let mut events_vec = Vec::new();

        for event in events.split(',') {
            let trimmed = event.trim().to_string();

            if trimmed.is_empty() {
                continue;
            }

            let event = trimmed.to_uppercase();

            if !supported_events.contains(&event) {
                return Err(format!("Event `{}` is not a valid event. Please pick one of the following: {}", trimmed, supported_events.join(", ")).into());
            }

            events_vec.push(event);
        }

        Some(events_vec)
    } else {
        None
    };

    let sink_id = gen_random(24);
    sqlx::query!(
        "INSERT INTO auditlogs__sinks (id, guild_id, type, sink, events, broken, created_by, last_updated_by) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        sink_id,
        guild_id.to_string(),
        "discord_webhook",
        webhook,
        events_split.as_deref(),
        false,
        ctx.author().id.to_string(),
        ctx.author().id.to_string(),
    )
    .execute(&ctx.data().pool)
    .await?;

    Ok(())
}
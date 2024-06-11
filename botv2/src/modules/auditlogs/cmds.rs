use crate::silverpelt::value::Value;
use crate::{Context, Error};
use secrecy::ExposeSecret;
use serenity::all::{Channel, ChannelType};

#[poise::command(
    prefix_command,
    slash_command,
    user_cooldown = 1,
    subcommands("list_sinks", "add_channel", "add_discordhook", "remove_sink")
)]
pub async fn auditlogs(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(prefix_command, slash_command, user_cooldown = 1)]
pub async fn list_sinks(ctx: Context<'_>) -> Result<(), Error> {
    crate::silverpelt::settings::poise::settings_viewer(&ctx, &super::sinks::sink()).await
}

#[poise::command(prefix_command, slash_command, user_cooldown = 1)]
pub async fn add_channel(
    ctx: Context<'_>,
    #[description = "Channel to send logs to"] channel: Channel,
    #[description = "Specific events you want to filter by"] events: Option<String>,
    #[description = "Whether or not to create a webhook or not. Default is true"]
    use_webhook: Option<bool>,
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

    let events = if let Some(events) = events {
        let events: Vec<String> = events.split(',').map(|x| x.to_string()).collect();
        Some(events)
    } else {
        None
    };

    ctx.defer().await?;

    let sink_id = if use_webhook.unwrap_or(true) {
        if !bot_perms.manage_webhooks() {
            return Err("I do not have permission to manage webhooks in this channel".into());
        }

        let webhook = gc
            .create_webhook(
                ctx.http(),
                serenity::all::CreateWebhook::new("AntiRaid Audit Logs"),
            )
            .await?;

        let webhook_url = {
            if let Some(token) = webhook.token {
                format!(
                    "https://discord.com/api/webhooks/{}/{}",
                    webhook.id,
                    token.expose_secret()
                )
            } else if let Some(url) = webhook.url {
                url.expose_secret().to_string()
            } else {
                webhook.url()?
            }
        };

        sqlx::query!(
            "INSERT INTO auditlogs__sinks (guild_id, type, sink, events, broken, created_by, last_updated_by) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
            guild_id.to_string(),
            "discord_webhook",
            webhook_url,
            events.as_deref(),
            false,
            ctx.author().id.to_string(),
            ctx.author().id.to_string(),
        )
        .fetch_one(&ctx.data().pool)
        .await?
        .id
        .to_string()
    } else {
        sqlx::query!(
            "INSERT INTO auditlogs__sinks (guild_id, type, sink, events, broken, created_by, last_updated_by) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
            guild_id.to_string(),
            "channel",
            gc.id.to_string(),
            events.as_deref(),
            false,
            ctx.author().id.to_string(),
            ctx.author().id.to_string(),
        )
        .fetch_one(&ctx.data().pool)
        .await?
        .id
        .to_string()
    };

    ctx.say(format!(
        "Successfully added a new Discord webhook sink for audit logs with ID `{}`",
        sink_id
    ))
    .await?;

    Ok(())
}

#[poise::command(prefix_command, slash_command, user_cooldown = 1)]
pub async fn add_discordhook(
    ctx: Context<'_>,
    #[description = "Webhook URL to send logs to"] webhook: String,
    #[description = "Specific events you want to filter by"] events: Option<String>,
) -> Result<(), Error> {
    let events = if let Some(events) = events {
        let events: Vec<String> = events.split(',').map(|x| x.to_string()).collect();
        Some(events)
    } else {
        None
    };

    crate::silverpelt::settings::poise::settings_creator(
        &ctx,
        &super::sinks::sink(),
        indexmap::indexmap! {
            "type".to_string() => Value::String("webhook".to_string()),
            "sink".to_string() => Value::String(webhook),
            "events".to_string() => {
                match events {
                    Some(events) => {
                        let mut value_events = Vec::new();

                        for evt in events {
                            value_events.push(Value::String(evt));
                        }

                        Value::List(value_events)
                    }
                    None => Value::None
                }
            },
            "broken".to_string() => Value::Boolean(false),
        },
    )
    .await?;

    Ok(())
}

#[poise::command(prefix_command, slash_command, user_cooldown = 1)]
pub async fn remove_sink(
    ctx: Context<'_>,
    #[description = "Sink ID to remove"] sink_id: String,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command can only be used in a guild".into());
    };

    let res = sqlx::query!(
        "DELETE FROM auditlogs__sinks WHERE guild_id = $1 AND id = $2",
        guild_id.to_string(),
        sink_id.parse::<sqlx::types::Uuid>()?,
    )
    .execute(&ctx.data().pool)
    .await?;

    if res.rows_affected() == 0 {
        return Err("No sink found with that ID".into());
    }

    ctx.say("Successfully removed sink").await?;

    Ok(())
}

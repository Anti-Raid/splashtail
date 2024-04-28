use crate::{Context, Error};
use poise::CreateReply;
use serenity::all::{Channel, ChannelType, CreateEmbed};
use botox::crypto::gen_random;
use futures_util::StreamExt;
use std::time::Duration;
use secrecy::ExposeSecret;

#[poise::command(prefix_command, slash_command, user_cooldown = 1, subcommands("list_sinks", "add_channel", "add_discordhook", "remove_sink"))]
pub async fn auditlogs(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(prefix_command, slash_command, user_cooldown = 1)]
pub async fn list_sinks(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command can only be used in a guild".into());
    };

    let sinks = sqlx::query!(
        "SELECT id, type AS typ, events, broken, created_at, created_by, last_updated_by FROM auditlogs__sinks WHERE guild_id = $1",
        guild_id.to_string(),
    )
    .fetch_all(&ctx.data().pool)
    .await?;

    if sinks.is_empty() {
        return Err("No sinks found. You can create a sink (a channel/webhook that will recieve logged events) using `/auditlogs addchannel` or `/auditlogs add_discordhook`".into());
    }

    struct SinkLister {
        id: String,
        typ: String,
        events: Option<Vec<String>>,
        broken: bool,
        created_at: String,
        created_by: String,
        last_updated_by: String,
    }

    let mut sink_lister = Vec::new();

    for sink in sinks {
        sink_lister.push(SinkLister {
            id: sink.id,
            typ: sink.typ,
            events: sink.events,
            broken: sink.broken,
            created_at: format!("<t:{}:F>", sink.created_at),
            created_by: sink.created_by,
            last_updated_by: sink.last_updated_by,
        });
    }

    fn create_sink_list_embed(
        sink: &SinkLister,
    ) -> CreateEmbed {
        let mut embed = CreateEmbed::default();
        embed = embed.title(format!("Sink ID: {}", sink.id));
        embed = embed.field("Type", sink.typ.clone(), false);

        if let Some(events) = &sink.events {
            embed = embed.field("Events", events.join(", "), false);
        }

        embed = embed.field("Broken", sink.broken.to_string(), false);
        embed = embed.field("Created At", sink.created_at.clone(), false);
        embed = embed.field("Created By", sink.created_by.clone(), false);
        embed = embed.field("Last Updated By", sink.last_updated_by.clone(), false);

        embed
    }

    fn create_action_row<'a>(
        index: usize,
        total: usize,
    ) -> serenity::all::CreateActionRow<'a> {
        serenity::all::CreateActionRow::Buttons(
            vec![
                serenity::all::CreateButton::new(
                    "previous",
                )
                .style(serenity::all::ButtonStyle::Primary)
                .label("Previous")
                .disabled(index == 0),
                serenity::all::CreateButton::new(
                    "next",
                )
                .style(serenity::all::ButtonStyle::Primary)
                .label("Next")
                .disabled(index >= total - 1),
                serenity::all::CreateButton::new(
                    "first",
                )
                .style(serenity::all::ButtonStyle::Primary)
                .label("First")
                .disabled(false),
            ]
        )
    }

    let mut index = 0;

    let msg = ctx.send(
        CreateReply::new()
        .embed(create_sink_list_embed(&sink_lister[index]))
        .components(
            vec![
                create_action_row(index, sink_lister.len())
            ]
        )
    )
    .await?
    .into_message()
    .await?;

    let collector = msg
    .await_component_interactions(ctx.serenity_context().shard.clone())
    .author_id(ctx.author().id)
    .timeout(Duration::from_secs(180));

    let mut collect_stream = collector.stream();

    while let Some(item) = collect_stream.next().await {
        let item_id = item.data.custom_id.as_str();

        match item_id {
            "previous" => {
                if index == 0 {
                    continue;
                }

                index -= 1;
            }
            "next" => {
                if index >= sink_lister.len() - 1 {
                    continue;
                }

                index += 1;
            }
            "first" => {
                index = 0;
            }
            _ => {}
        }

        item.defer(&ctx.serenity_context().http).await?;

        let cr = CreateReply::new()
        .embed(create_sink_list_embed(&sink_lister[index]))
        .components(
            vec![
                create_action_row(index, sink_lister.len())
            ]
        );

        item.edit_response(
            &ctx.serenity_context().http,
            cr.to_slash_initial_response_edit(serenity::all::EditInteractionResponse::default()),
        )
        .await?;
    }

    Ok(())
}

pub async fn check_all_events(events: Vec<String>) -> Result<(), crate::Error> {
    let res = tokio::time::timeout(
        std::time::Duration::from_millis(1000),
        tokio::task::spawn_blocking(
            move || {
                let supported_events = gwevent::core::event_list();

                for event in events {
                    let trimmed = event.trim().to_string();

                    if trimmed.is_empty() {
                        continue;
                    }

                    // All Anti-Raid events are filterable
                    if trimmed.starts_with("AR/") {
                        continue;
                    }

                    // Regex compile check
                    if trimmed.starts_with("R/") {
                        if let Err(e) = regex::Regex::new(&trimmed) {
                            return Err(format!("Event `{}` is not a valid regex. Error: {}", trimmed, e));
                        }
                    }

                    let event = trimmed.to_uppercase();

                    if !supported_events.contains(&event.as_str()) {
                        return Err(format!("Event `{}` is not a valid event. Please pick one of the following: {}", trimmed, supported_events.join(", ")));
                    }
                }

                Ok(())
            }
        )
    )
    .await??;

    res.map_err(|e| e.into())
}

#[poise::command(prefix_command, slash_command, user_cooldown = 1)]
pub async fn add_channel(
    ctx: Context<'_>,
    #[description = "Channel to send logs to"] channel: Channel,
    #[description = "Specific events you want to filter by"] events: Option<String>,
    #[description = "Whether or not to create a webhook or not. Default is true"] use_webhook: Option<bool>,
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
        check_all_events(events.clone()).await?;
        Some(events)
    } else {
        None
    };

    let sink_id = gen_random(24);

    ctx.defer().await?;

    if use_webhook.unwrap_or(true) {
        if !bot_perms.manage_webhooks() {
            return Err("I do not have permission to manage webhooks in this channel".into());
        }

        let webhook = gc.create_webhook(
            ctx.http(),
            serenity::all::CreateWebhook::new("AntiRaid Audit Logs")
        ).await?;

        let webhook_url = {
            if let Some(token) = webhook.token {
                format!("https://discord.com/api/webhooks/{}/{}", webhook.id, token.expose_secret())
            } else if let Some(url) = webhook.url {
                url.expose_secret().to_string()
            } else {
                webhook.url()?
            }
        };

        sqlx::query!(
            "INSERT INTO auditlogs__sinks (id, guild_id, type, sink, events, broken, created_by, last_updated_by) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            sink_id,
            guild_id.to_string(),
            "discord_webhook",
            webhook_url,
            events.as_deref(),
            false,
            ctx.author().id.to_string(),
            ctx.author().id.to_string(),
        )
        .execute(&ctx.data().pool)
        .await?;
    } else {
        sqlx::query!(
            "INSERT INTO auditlogs__sinks (id, guild_id, type, sink, events, broken, created_by, last_updated_by) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            sink_id,
            guild_id.to_string(),
            "channel",
            gc.id.to_string(),
            events.as_deref(),
            false,
            ctx.author().id.to_string(),
            ctx.author().id.to_string(),
        )
        .execute(&ctx.data().pool)
        .await?;
    }

    ctx.say(
        format!("Successfully added a new Discord webhook sink for audit logs with ID `{}`", sink_id)
    )
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
    
    let events = if let Some(events) = events {
        let events: Vec<String> = events.split(',').map(|x| x.to_string()).collect();
        check_all_events(events.clone()).await?;
        Some(events)
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
        events.as_deref(),
        false,
        ctx.author().id.to_string(),
        ctx.author().id.to_string(),
    )
    .execute(&ctx.data().pool)
    .await?;

    ctx.say(
        format!("Successfully added a new Discord webhook sink for audit logs with ID `{}`", sink_id)
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
        sink_id,
    )
    .execute(&ctx.data().pool)
    .await?;

    if res.rows_affected() == 0 {
        return Err("No sink found with that ID".into());
    }

    ctx.say("Successfully removed sink").await?;

    Ok(())
}
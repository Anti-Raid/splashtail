use crate::silverpelt::value::Value;
use crate::{Context, Error};
use serenity::all::ChannelId;

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
    #[description = "Channel to send logs to"] channel: ChannelId,
    #[description = "Specific events you want to filter by"] events: Option<String>,
) -> Result<(), Error> {
    crate::silverpelt::settings::poise::settings_creator(
        &ctx,
        &super::sinks::sink(),
        indexmap::indexmap! {
            "type".to_string() => Value::String("channel".to_string()),
            "sink".to_string() => Value::String(channel.to_string()),
            "events".to_string() => {
                let events = if let Some(events) = events {
                    let events: Vec<String> = events.split(',').map(|x| x.to_string()).collect();
                    Some(events)
                } else {
                    None
                };

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
pub async fn add_discordhook(
    ctx: Context<'_>,
    #[description = "Webhook URL to send logs to"] webhook: String,
    #[description = "Specific events you want to filter by"] events: Option<String>,
) -> Result<(), Error> {
    crate::silverpelt::settings::poise::settings_creator(
        &ctx,
        &super::sinks::sink(),
        indexmap::indexmap! {
            "type".to_string() => Value::String("discordhook".to_string()),
            "sink".to_string() => Value::String(webhook),
            "events".to_string() => {
                let events = if let Some(events) = events {
                    let events: Vec<String> = events.split(',').map(|x| x.to_string()).collect();
                    Some(events)
                } else {
                    None
                };

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

use crate::{Context, Error};
use serenity::all::ChannelId;
use splashcore_rs::value::Value;

#[poise::command(
    prefix_command,
    slash_command,
    user_cooldown = 1,
    subcommands(
        "list_sinks",
        "add_channel",
        "add_sink",
        "add_discordhook",
        "edit_sink",
        "remove_sink"
    )
)]
pub async fn auditlogs(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(prefix_command, slash_command, user_cooldown = 1)]
pub async fn list_sinks(ctx: Context<'_>) -> Result<(), Error> {
    crate::silverpelt::settings_poise::settings_viewer(&ctx, &super::settings::sink()).await
}

#[poise::command(prefix_command, slash_command, user_cooldown = 1)]
pub async fn add_sink(
    ctx: Context<'_>,
    #[description = "Sink type to set"] r#type: String,
    #[description = "Sink to set"] sink: String,
    #[description = "Specific events you want to filter by"] events: Option<String>,
    #[description = "Mark as broken (temporarily disables the webhook)"] broken: bool,
) -> Result<(), Error> {
    crate::silverpelt::settings_poise::settings_creator(
        &ctx,
        &super::settings::sink(),
        indexmap::indexmap! {
            "type".to_string() => Value::String(r#type),
            "sink".to_string() => Value::String(sink),
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
            "broken".to_string() => Value::Boolean(broken),
        },
    )
    .await?;

    Ok(())
}

#[poise::command(prefix_command, slash_command, user_cooldown = 1)]
pub async fn add_channel(
    ctx: Context<'_>,
    #[description = "Channel to send logs to"] channel: ChannelId,
    #[description = "Specific events you want to filter by"] events: Option<String>,
) -> Result<(), Error> {
    crate::silverpelt::settings_poise::settings_creator(
        &ctx,
        &super::settings::sink(),
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
    crate::silverpelt::settings_poise::settings_creator(
        &ctx,
        &super::settings::sink(),
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
pub async fn edit_sink(
    ctx: Context<'_>,
    #[description = "Sink ID to edit"] id: String,
    #[description = "Sink type to set"] r#type: String,
    #[description = "Sink to set"] sink: String,
    #[description = "Specific events you want to filter by"] events: Option<String>,
    #[description = "Mark as broken (temporarily disables the webhook)"] broken: bool,
) -> Result<(), Error> {
    crate::silverpelt::settings_poise::settings_updater(
        &ctx,
        &super::settings::sink(),
        indexmap::indexmap! {
            "id".to_string() => Value::String(id),
            "type".to_string() => Value::String(r#type),
            "sink".to_string() => Value::String(sink),
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
            "broken".to_string() => Value::Boolean(broken),
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
    crate::silverpelt::settings_poise::settings_deleter(
        &ctx,
        &super::settings::sink(),
        Value::String(sink_id),
    )
    .await?;

    Ok(())
}

// Event modifier related commands
use poise::serenity_prelude::ChannelId;
use splashcore_rs::value::Value;

use crate::{Context, Error};

/// Lists all event modifiers
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    guild_cooldown = 60,
    required_permissions = "MANAGE_GUILD"
)]
pub async fn eventmods_list(ctx: Context<'_>) -> Result<(), Error> {
    crate::silverpelt::settings_poise::settings_viewer(&ctx, &super::settings::event_modifiers())
        .await
}

/// Creates a event modifier on a webhook
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    guild_cooldown = 60,
    required_permissions = "MANAGE_GUILD"
)]
#[allow(clippy::too_many_arguments)]
pub async fn eventmods_create(
    ctx: Context<'_>,
    #[description = "The webhook ID"] webhook_id: String,
    #[description = "The events to match against, comma/space seperated"] events: String,
    #[description = "Blacklist the events"] blacklisted: bool,
    #[description = "Whitelist the events. Other events will not be allowed"] whitelisted: bool,
    #[description = "Priority. Use 0 for normal priority"] priority: Option<i32>,
    // Lazy = "prefer to parse the current argument as the other params first"
    #[description = "Repository ID, will match all if unset"]
    #[lazy]
    repo_id: Option<String>,
    #[description = "Redirect channel ID"] redirect_channel: Option<ChannelId>,
) -> Result<(), Error> {
    crate::silverpelt::settings_poise::settings_creator(
        &ctx,
        &super::settings::event_modifiers(),
        indexmap::indexmap! {
            "webhook_id".to_string() => Value::String(webhook_id),
            "events".to_string() => {
                let events: Vec<String> = events.split(',').map(|x| x.to_string()).collect();

                let mut value_events = Vec::new();

                for evt in events {
                    value_events.push(Value::String(evt));
                }

                Value::List(value_events)
            },
            "blacklisted".to_string() => Value::Boolean(blacklisted),
            "whitelisted".to_string() => Value::Boolean(whitelisted),
            "priority".to_string() => Value::Integer(priority.unwrap_or_default() as i64),
            "repo_id".to_string() => {
                if let Some(repo_id) = repo_id {
                    Value::String(repo_id)
                } else {
                    Value::None
                }
            },
            "redirect_channel".to_string() => {
                if let Some(redirect_channel) = redirect_channel {
                    Value::String(redirect_channel.to_string())
                } else {
                    Value::None
                }
            }
        },
    )
    .await
}

/// Updates a event modifier on a webhook
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    guild_cooldown = 60,
    required_permissions = "MANAGE_GUILD"
)]
#[allow(clippy::too_many_arguments)]
pub async fn eventmods_update(
    ctx: Context<'_>,
    #[description = "The modifier ID"] modifier_id: String,
    #[description = "The webhook ID"] webhook_id: String,
    #[description = "The events to match against, comma/space seperated"] events: String,
    #[description = "Blacklist the events"] blacklisted: bool,
    #[description = "Whitelist the events. Other events will not be allowed"] whitelisted: bool,
    #[description = "Priority. Use 0 for normal priority"] priority: Option<i32>,
    // Lazy = "prefer to parse the current argument as the other params first"
    #[description = "Repository ID, will match all if unset"]
    #[lazy]
    repo_id: Option<String>,
    #[description = "Redirect channel ID"] redirect_channel: Option<ChannelId>,
) -> Result<(), Error> {
    crate::silverpelt::settings_poise::settings_updater(
        &ctx,
        &super::settings::event_modifiers(),
        indexmap::indexmap! {
            "id".to_string() => Value::String(modifier_id),
            "webhook_id".to_string() => Value::String(webhook_id),
            "events".to_string() => {
                let events: Vec<String> = events.split(',').map(|x| x.to_string()).collect();

                let mut value_events = Vec::new();

                for evt in events {
                    value_events.push(Value::String(evt));
                }

                Value::List(value_events)
            },
            "blacklisted".to_string() => Value::Boolean(blacklisted),
            "whitelisted".to_string() => Value::Boolean(whitelisted),
            "priority".to_string() => Value::Integer(priority.unwrap_or_default() as i64),
            "repo_id".to_string() => {
                if let Some(repo_id) = repo_id {
                    Value::String(repo_id)
                } else {
                    Value::None
                }
            },
            "redirect_channel".to_string() => {
                if let Some(redirect_channel) = redirect_channel {
                    Value::String(redirect_channel.to_string())
                } else {
                    Value::None
                }
            }
        },
    )
    .await
}

/// Deletes a event modifier by id
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    guild_cooldown = 60,
    required_permissions = "MANAGE_GUILD"
)]
pub async fn eventmods_delete(
    ctx: Context<'_>,
    #[description = "The modifier ID"] modifier_id: String,
) -> Result<(), Error> {
    crate::silverpelt::settings_poise::settings_deleter(
        &ctx,
        &super::settings::event_modifiers(),
        Value::String(modifier_id),
    )
    .await
}

use serenity::all::{Channel, ChannelType};

pub async fn check_all_events(events: Vec<String>) -> Result<(), crate::Error> {
    let res = tokio::time::timeout(
        std::time::Duration::from_millis(250),
        tokio::task::spawn_blocking(move || {
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
                        return Err(format!(
                            "Event `{}` is not a valid regex. Error: {}",
                            trimmed, e
                        ));
                    }
                }

                let event = trimmed.to_uppercase();

                if !supported_events.contains(&event.as_str()) {
                    return Err(format!(
                        "Event `{}` is not a valid event. Please pick one of the following: {}",
                        trimmed,
                        supported_events.join(", ")
                    ));
                }
            }

            Ok(())
        }),
    )
    .await??;

    res.map_err(|e| e.into())
}

pub async fn check_channel(
    cache_http: &crate::CacheHttpImpl,
    channel_id: serenity::model::id::ChannelId,
    guild_id: serenity::model::id::GuildId,
) -> Result<(), crate::Error> {
    let channel = channel_id
        .to_channel(&cache_http)
        .await
        .map_err(|e| format!("Error fetching channel: {}", e))?;

    let gc = match channel {
            Channel::Guild(c) => c,
            _ => return Err("The channel you have selected appears to be from a Direct Message (DM). This is NOT supported at this time".into()),
        };

    match gc.kind {
        ChannelType::Forum => return Err("Cannot use a Forum channel for audit logs. Try making a thread and using the thread (or use a different channel)".into()),
        ChannelType::Category => return Err("Cannot use a Category channel for audit logs. Try using a text or voice channel".into()),
        _ => {}
    };

    if gc.guild_id != guild_id {
        return Err("The channel you have selected is not in the guild you are in".into());
    }

    // Check if we have permissions in said channel
    let bot_perms =
        gc.permissions_for_user(&cache_http.cache, cache_http.cache.current_user().id)?;

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

    Ok(())
}

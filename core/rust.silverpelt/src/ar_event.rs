use crate::data::Data;
use crate::Error;
use std::sync::Arc;

pub use typetag; // Re-exported

pub struct EventHandlerContext<'a> {
    pub guild_id: serenity::all::GuildId,
    pub data: Arc<Data>,
    pub event: AntiraidEvent<'a>,
    pub serenity_context: serenity::all::Context,
}

/// This can be used to trigger a custom event
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CustomEvent {
    pub event_name: String,
    pub event_titlename: String,
    pub event_data: serde_json::Value,
}

#[derive(Debug)]
#[must_use]
pub enum AntiraidEvent<'a> {
    /// A regular discord event
    Discord(&'a serenity::all::FullEvent),

    /// A sting create event. Dispatched when a sting is created
    StingCreate(super::stings::Sting),

    /// A sting expiry event. Dispatched when a sting expires
    StingExpire(super::stings::Sting),

    /// A sting delete event. Dispatched when a sting is manually deleted
    StingDelete(super::stings::Sting),

    /// A punishment create event. Dispatched when a punishment is created
    PunishmentCreate(super::punishments::Punishment),

    /// A punishment expiration event. Dispatched when a punishment expires
    PunishmentExpire(super::punishments::Punishment),

    /// An on startup event is fired *at least once* when the bot starts up or the set of templates are modified
    ///
    /// The inner Vec<String> is the list of templates modified/reloaded
    OnStartup(Vec<String>),

    /// A custom event
    Custom(CustomEvent),
}

/// Dispatches an event to all modules sequentially
///
/// This works well because Anti-Raid uses very few event listeners (only 2)
pub async fn dispatch_event_to_modules<'a>(
    event_handler_context: &EventHandlerContext<'a>,
) -> Result<(), Vec<Error>> {
    let mut errors = Vec::new();

    for refs in event_handler_context
        .data
        .silverpelt_cache
        .module_cache
        .iter()
    {
        let module = refs.value();

        // To reduce DB calls / actually expensive work, check for event listeners first and then check the filter
        let Some(event_listeners) = module.event_listeners() else {
            continue;
        };

        if !event_listeners.event_handler_filter(&event_handler_context.event) {
            continue;
        }

        let module_enabled = {
            match crate::module_config::is_module_enabled(
                &event_handler_context.data.silverpelt_cache,
                &event_handler_context.data.pool,
                event_handler_context.guild_id,
                module.id(),
            )
            .await
            {
                Ok(enabled) => enabled,
                Err(e) => {
                    errors.push(format!("Error getting module enabled status: {}", e).into());
                    continue;
                }
            }
        };

        if !module_enabled {
            continue;
        }

        match event_listeners.event_handler(event_handler_context).await {
            Ok(_) => {}
            Err(e) => {
                errors.push(e);
            }
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(())
}

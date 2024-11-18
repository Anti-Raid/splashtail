use crate::data::Data;
use crate::Error;
use log::error;
use std::sync::Arc;
use tokio::task::JoinSet;

pub use typetag; // Re-exported

pub const SYSTEM_GUILD_ID: serenity::all::GuildId = serenity::all::GuildId::new(1);

pub struct EventHandlerContext {
    pub guild_id: serenity::all::GuildId,
    pub data: Arc<Data>,
    pub event: AntiraidEvent,
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
pub enum AntiraidEvent {
    /// A TrustedWebEvent is dispatched when a trusted web event is received
    ///
    /// This replaces the old `animus magic toggles` system with one that is more type safe and easier to use
    ///
    /// Note that guild_id may be either a guild id or SYSTEM_GUILD_ID
    ///
    /// Format: (event_name, event_data)
    TrustedWebEvent((String, serde_json::Value)),

    /// A regular discord event
    Discord(serenity::all::FullEvent),

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

/// Dispatches an event to all modules asynchronously
pub async fn dispatch_event_to_modules(
    event_handler_context: Arc<EventHandlerContext>,
) -> Result<(), Vec<Error>> {
    // Inner dispatch helper
    async fn dispatch_for_module(
        event_handler_context: Arc<EventHandlerContext>,
        event_listeners: Box<dyn crate::module::ModuleEventListeners>,
    ) -> Result<(), Error> {
        event_listeners
            .event_handler(&event_handler_context)
            .await?;
        Ok(())
    }

    let mut set = JoinSet::new();

    let mut futs = Vec::new();
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
            if event_handler_context.guild_id == SYSTEM_GUILD_ID {
                true
            } else {
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
            }
        };

        if !module_enabled {
            continue;
        }

        let ehr = event_handler_context.clone();
        futs.push(dispatch_for_module(ehr, event_listeners));
    }

    for fut in futs {
        set.spawn(fut);
    }

    while let Some(res) = set.join_next().await {
        match res {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => {
                error!("Error in dispatch_event_to_modules: {}", e);
                errors.push(e);
            }
            Err(e) => {
                error!("Error in dispatch_event_to_modules: {}", e);
                errors.push(
                    format!(
                        "Error dispatching event: `dispatch_event_to_modules`: {}",
                        e
                    )
                    .into(),
                );
            }
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(())
}

/// Dispatches an event to all modules sequentially
///
/// Note: if you don't own the EventHandlerContext or do not have an Arc<EventHandlerContext>, this is the only method you can use to dispatch events
pub async fn dispatch_event_to_modules_seq(
    event_handler_context: &EventHandlerContext,
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
            if event_handler_context.guild_id == SYSTEM_GUILD_ID {
                true
            } else {
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

/// Dispatches an event to all modules asynchronously and flattens the errors to make it easier to handle
pub async fn dispatch_event_to_modules_errflatten(
    event_handler_context: Arc<EventHandlerContext>,
) -> Result<(), Error> {
    let res = dispatch_event_to_modules(event_handler_context).await;

    match res {
        Ok(_) => Ok(()),
        Err(errors) => {
            let mut error_string = String::new();
            for error in errors {
                error_string.push_str(&format!("{}\n", error));
            }

            Err(error_string.into())
        }
    }
}

mod cache;
mod core;
mod events;
mod handler;
mod settings;
mod strategy;

pub struct Module;

impl silverpelt::module::Module for Module {
    fn id(&self) -> &'static str {
        "limits"
    }

    fn name(&self) -> &'static str {
        "Limits"
    }

    fn description(&self) -> &'static str {
        "Experimental server ratelimiting module. Not yet suitable for production use. Should be combined with anti-nuke bots for best efficacy"
    }

    fn config_options(&self) -> Vec<module_settings::types::ConfigOption> {
        vec![
            (*settings::USER_ACTIONS).clone(),
            (*settings::GUILD_GLOBALS).clone(),
            (*settings::GUILD_LIMITS).clone(),
        ]
    }

    fn event_listeners(&self) -> Option<Box<dyn silverpelt::module::ModuleEventListeners>> {
        Some(Box::new(EventListener))
    }
}

struct EventListener;

#[async_trait::async_trait]
impl silverpelt::module::ModuleEventListeners for EventListener {
    async fn event_handler(
        &self,
        ectx: &silverpelt::ar_event::EventHandlerContext,
    ) -> Result<(), silverpelt::Error> {
        events::event_listener(ectx).await
    }

    fn event_handler_filter(&self, event: &silverpelt::ar_event::AntiraidEvent) -> bool {
        match event {
            silverpelt::ar_event::AntiraidEvent::Discord(_) => true,
            silverpelt::ar_event::AntiraidEvent::Custom(ref ce) => {
                ce.target() == std_events::limit::LIMIT_TARGET_ID
            }
            _ => false,
        }
    }
}

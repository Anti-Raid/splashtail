mod cache;
mod checks;
mod events;
mod settings;

pub struct Module;

impl silverpelt::module::Module for Module {
    fn id(&self) -> &'static str {
        "auditlogs"
    }

    fn name(&self) -> &'static str {
        "Audit Logs"
    }

    fn description(&self) -> &'static str {
        "Customizable and comprehensive audit logging module supporting 72+ discord events"
    }

    fn event_listeners(&self) -> Option<Box<dyn silverpelt::module::ModuleEventListeners>> {
        Some(Box::new(EventHandler))
    }

    fn config_options(&self) -> Vec<module_settings::types::ConfigOption> {
        vec![(*settings::SINK).clone()]
    }
}

struct EventHandler;

#[async_trait::async_trait]
impl silverpelt::module::ModuleEventListeners for EventHandler {
    async fn event_handler(
        &self,
        ectx: &silverpelt::ar_event::EventHandlerContext,
    ) -> Result<(), silverpelt::Error> {
        events::event_listener(ectx).await
    }

    fn event_handler_filter(&self, event: &silverpelt::ar_event::AntiraidEvent) -> bool {
        match event {
            silverpelt::ar_event::AntiraidEvent::TrustedWebEvent((event_name, _)) => {
                event_name == "checkAllEvents"
            } // We need trusted web events
            silverpelt::ar_event::AntiraidEvent::Discord(_) => true,
            silverpelt::ar_event::AntiraidEvent::Custom(ref ce) => {
                ce.target() == std_events::auditlog::AUDITLOG_TARGET_ID
            }
            silverpelt::ar_event::AntiraidEvent::StingCreate(_) => true,
            _ => false,
        }
    }
}

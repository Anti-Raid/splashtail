pub mod core;
pub mod events;
pub mod settings;

pub struct Module;

impl silverpelt::module::Module for Module {
    fn id(&self) -> &'static str {
        "punishment_autotriggers"
    }

    fn name(&self) -> &'static str {
        "Punishment Autotriggers"
    }

    fn description(&self) -> &'static str {
        "Auto-trigger punishments based on stings."
    }

    fn is_default_enabled(&self) -> bool {
        true
    }

    fn event_listeners(&self) -> Option<Box<dyn silverpelt::module::ModuleEventListeners>> {
        Some(Box::new(EventListener))
    }

    fn config_options(&self) -> Vec<module_settings::types::ConfigOption> {
        vec![(*settings::AUTOTRIGGERS).clone()]
    }
}

struct EventListener;

#[async_trait::async_trait]
impl silverpelt::module::ModuleEventListeners for EventListener {
    async fn event_handler(
        &self,
        ctx: &silverpelt::ar_event::EventHandlerContext,
    ) -> Result<(), silverpelt::Error> {
        events::event_listener(ctx).await
    }

    fn event_handler_filter(&self, event: &silverpelt::ar_event::AntiraidEvent) -> bool {
        matches!(event, silverpelt::ar_event::AntiraidEvent::StingCreate(_)) // We only care about sting creates
    }
}

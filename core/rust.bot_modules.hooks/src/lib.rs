mod events;

pub struct Module;

impl silverpelt::module::Module for Module {
    fn id(&self) -> &'static str {
        "hooks"
    }

    fn name(&self) -> &'static str {
        "Hooks"
    }

    fn description(&self) -> &'static str {
        "Hooks allow for running a Lua template when a specific set of events occur."
    }

    fn is_default_enabled(&self) -> bool {
        true // Enable it by default
    }

    fn event_listeners(&self) -> Option<Box<dyn silverpelt::module::ModuleEventListeners>> {
        Some(Box::new(EventHandler))
    }

    fn full_command_list(&self) -> Vec<silverpelt::module::CommandObj> {
        modules_ext::create_full_command_list(self)
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
            silverpelt::ar_event::AntiraidEvent::TrustedWebEvent(_) => true, // We need trusted web events
            silverpelt::ar_event::AntiraidEvent::Discord(_) => true,
            silverpelt::ar_event::AntiraidEvent::Custom(_) => true,
            silverpelt::ar_event::AntiraidEvent::StingCreate(_) => true,
            silverpelt::ar_event::AntiraidEvent::StingExpire(_) => true,
            silverpelt::ar_event::AntiraidEvent::StingDelete(_) => true,
            silverpelt::ar_event::AntiraidEvent::PunishmentCreate(_) => true,
            silverpelt::ar_event::AntiraidEvent::PunishmentExpire(_) => true,
            silverpelt::ar_event::AntiraidEvent::OnStartup(_) => true,
        }
    }
}

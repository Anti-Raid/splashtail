pub mod events;

pub struct Module;

impl silverpelt::module::Module for Module {
    fn id(&self) -> &'static str {
        "temporary_punishments"
    }

    fn name(&self) -> &'static str {
        "Temporary Punishments"
    }

    fn description(&self) -> &'static str {
        "Simple default temporary punishment handling. Can be disabled for servers that want custom handling through e.g. hooks."
    }

    fn is_default_enabled(&self) -> bool {
        true
    }

    fn raw_commands(&self) -> Vec<silverpelt::module::CommandObj> {
        vec![]
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
        ctx: &silverpelt::ar_event::EventHandlerContext,
    ) -> Result<(), silverpelt::Error> {
        events::event_listener(ctx).await
    }

    fn event_handler_filter(&self, event: &silverpelt::ar_event::AntiraidEvent) -> bool {
        matches!(
            event,
            silverpelt::ar_event::AntiraidEvent::PunishmentExpire(_)
        ) // We only care about punishment expires
    }
}

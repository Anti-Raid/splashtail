pub mod cmd;
pub mod core;
pub mod events;

use indexmap::indexmap;

pub struct Module;

impl silverpelt::module::Module for Module {
    fn id(&self) -> &'static str {
        "punishments"
    }

    fn name(&self) -> &'static str {
        "Punishments"
    }

    fn description(&self) -> &'static str {
        "Auto-trigger punishments based on stings."
    }

    fn is_default_enabled(&self) -> bool {
        true
    }

    fn raw_commands(&self) -> Vec<silverpelt::module::CommandObj> {
        vec![(
            cmd::punishments(),
            indexmap! {
                "add" => silverpelt::types::CommandExtendedData::kittycat_or_admin("punishments", "add"),
                "list" => silverpelt::types::CommandExtendedData::kittycat_or_admin("punishments", "list"),
                "delete" => silverpelt::types::CommandExtendedData::kittycat_or_admin("punishments", "delete"),
            },
        )]
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
        matches!(event, silverpelt::ar_event::AntiraidEvent::StingCreate(_)) // We only care about sting creates
    }
}

mod cache;
mod dehoist;
mod events;
mod guildprotect;
mod settings;
mod types;

pub struct Module;

impl silverpelt::module::Module for Module {
    fn id(&self) -> &'static str {
        "inspector"
    }

    fn name(&self) -> &'static str {
        "Inspector"
    }

    fn description(&self) -> &'static str {
        "Provides passive anti-spam options"
    }

    fn event_listeners(&self) -> Option<Box<dyn silverpelt::module::ModuleEventListeners>> {
        Some(Box::new(EventListener))
    }

    fn s3_paths(&self) -> Vec<String> {
        vec!["inspector/guild_icons/{guild_id}".to_string()]
    }

    fn config_options(&self) -> Vec<module_settings::types::ConfigOption> {
        vec![
            (*settings::INSPECTOR_GLOBAL_OPTIONS).clone(),
            (*settings::INSPECTOR_SPECIFIC_OPTIONS).clone(),
        ]
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
        match event {
            silverpelt::ar_event::AntiraidEvent::Discord(_) => true,
            silverpelt::ar_event::AntiraidEvent::TrustedWebEvent((event_name, _)) => {
                event_name == "inspector.clearCache" || event_name == "inspector.resetFakeBotsCache"
            }
            _ => false, // Ignore all other events
        }
    }
}

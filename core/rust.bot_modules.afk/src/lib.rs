mod events;
mod settings;
mod tasks;

use futures_util::future::FutureExt;

pub struct Module;

impl silverpelt::module::Module for Module {
    fn id(&self) -> &'static str {
        "afk"
    }

    fn name(&self) -> &'static str {
        "AFK"
    }

    fn description(&self) -> &'static str {
        "Exactly what it says. Away from keyboard related commands"
    }

    fn background_tasks(&self) -> Vec<silverpelt::BackgroundTask> {
        vec![(
            botox::taskman::Task {
                name: "AFK Expiration Task",
                description: "Handle expired AFKs",
                duration: std::time::Duration::from_secs(300),
                enabled: true,
                run: Box::new(move |ctx| tasks::afk_task(ctx).boxed()),
            },
            |_| (true, "AFK expiration is always enabled".to_string()),
        )]
    }

    fn event_listeners(&self) -> Option<Box<dyn silverpelt::module::ModuleEventListeners>> {
        Some(Box::new(EventHandler))
    }

    fn config_options(&self) -> Vec<module_settings::types::ConfigOption> {
        vec![(*settings::AFKS).clone()]
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
        matches!(event, silverpelt::ar_event::AntiraidEvent::Discord(_)) // We only care about discord events
    }
}

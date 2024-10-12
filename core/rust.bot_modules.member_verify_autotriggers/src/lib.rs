mod events;
mod settings;

pub struct Module;

impl silverpelt::module::Module for Module {
    fn id(&self) -> &'static str {
        "member_verify_autotriggers"
    }

    fn name(&self) -> &'static str {
        "Member Verify Autotriggers"
    }

    fn description(&self) -> &'static str {
        "Auto-trigger actions to happen when a server member is verified (e.g. CAPTCHA success)."
    }

    fn is_default_enabled(&self) -> bool {
        false
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
        matches!(event, silverpelt::ar_event::AntiraidEvent::MemberVerify(_)) // We only care about member verify
    }
}

mod cmds;
mod consts;
mod settings;
mod templater;

pub struct Module;

impl silverpelt::module::Module for Module {
    fn id(&self) -> &'static str {
        "captcha"
    }

    fn name(&self) -> &'static str {
        "Captcha"
    }

    fn description(&self) -> &'static str {
        "CAPTCHA support for Anti-Raid. Highly experimental and not complete yet"
    }

    fn raw_commands(&self) -> Vec<silverpelt::module::CommandObj> {
        vec![
            (
                cmds::captcha_test(),
                indexmap::indexmap! {
                    "" => silverpelt::types::CommandExtendedData::kittycat_or_admin("captcha", "test")
                },
            ),
            (
                cmds::verify(),
                indexmap::indexmap! {
                    "" => silverpelt::types::CommandExtendedData::none()
                },
            ),
        ]
    }

    fn event_listeners(&self) -> Option<Box<dyn silverpelt::module::ModuleEventListeners>> {
        Some(Box::new(EventHandler))
    }

    fn config_options(&self) -> Vec<module_settings::types::ConfigOption> {
        vec![(*settings::CAPTCHA).clone()]
    }
}

struct EventHandler;

#[async_trait::async_trait]
impl silverpelt::module::ModuleEventListeners for EventHandler {
    fn event_handler_filter(&self, event: &silverpelt::ar_event::AntiraidEvent) -> bool {
        match event {
            silverpelt::ar_event::AntiraidEvent::Discord(_) => true,
            _ => false,
        }
    }
}

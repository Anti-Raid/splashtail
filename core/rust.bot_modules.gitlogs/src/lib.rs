mod settings;

pub struct Module;

impl silverpelt::module::Module for Module {
    fn id(&self) -> &'static str {
        "gitlogs"
    }

    fn name(&self) -> &'static str {
        "Git Logs"
    }

    fn description(&self) -> &'static str {
        "Advanced github logging for your development-related servers! Complete with event whitelists and redirecting specific events to different channels and other customizability."
    }

    fn config_options(&self) -> Vec<module_settings::types::ConfigOption> {
        vec![
            (*settings::WEBHOOKS).clone(),
            (*settings::REPOS).clone(),
            (*settings::EVENT_MODIFIERS).clone(),
        ]
    }
}

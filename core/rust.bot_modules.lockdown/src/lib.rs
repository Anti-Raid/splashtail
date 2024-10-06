pub mod cache;
pub mod core;
pub mod settings;

pub struct Module;

impl silverpelt::module::Module for Module {
    fn id(&self) -> &'static str {
        "lockdown"
    }

    fn name(&self) -> &'static str {
        "Lockdown"
    }

    fn description(&self) -> &'static str {
        "Lockdown module for quickly locking/unlocking your whole server or individual channels"
    }

    fn config_options(&self) -> Vec<module_settings::types::ConfigOption> {
        vec![
            (*settings::LOCKDOWN_SETTINGS).clone(),
            (*settings::LOCKDOWNS).clone(),
        ]
    }
}

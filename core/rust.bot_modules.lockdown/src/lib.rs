pub mod cache;
pub mod cmds;
pub mod core;
pub mod settings;

use indexmap::indexmap;

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

    fn raw_commands(&self) -> Vec<silverpelt::module::CommandObj> {
        vec![
            (
                cmds::lockdown_settings(),
                indexmap! {
                    "view" => silverpelt::types::CommandExtendedData::kittycat_or_admin("lockdown_settings", "view"),
                    "create" => silverpelt::types::CommandExtendedData::kittycat_or_admin("lockdown_settings", "create"),
                    "update" => silverpelt::types::CommandExtendedData::kittycat_or_admin("lockdown_settings", "update"),
                    "delete" => silverpelt::types::CommandExtendedData::kittycat_or_admin("lockdown_settings", "delete"),
                },
            ),
            (
                cmds::lockdown(),
                indexmap! {
                    "list" => silverpelt::types::CommandExtendedData::kittycat_or_admin("lockdown", "list"),
                    "lock" => silverpelt::types::CommandExtendedData::kittycat_or_admin("lockdown", "lock"),
                    "unlock" => silverpelt::types::CommandExtendedData::kittycat_or_admin("lockdown", "unlock"),
                },
            ),
        ]
    }

    fn config_options(&self) -> Vec<module_settings::types::ConfigOption> {
        vec![
            (*settings::LOCKDOWN_SETTINGS).clone(),
            (*settings::LOCKDOWNS).clone(),
        ]
    }
}

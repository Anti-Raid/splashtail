pub mod cache;
pub mod cmds;
pub mod core;
pub mod settings;

use indexmap::indexmap;

pub fn module() -> silverpelt::Module {
    silverpelt::Module {
        id: "lockdown",
        name: "Lockdown",
        description:
            "Lockdown module for quickly locking/unlocking your whole server or individual channels",
        toggleable: true,
        commands_toggleable: true,
        virtual_module: false,
        web_hidden: false,
        is_default_enabled: false,
        commands: vec![
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
        ],
        config_options: vec![
            (*settings::LOCKDOWN_SETTINGS).clone(),
            (*settings::LOCKDOWNS).clone(),
        ],
        ..Default::default()
    }
}

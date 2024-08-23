pub mod cache;
pub mod cmds;
pub mod quick;
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
                cmds::lockserver(),
                indexmap! {
                    "list" => silverpelt::types::CommandExtendedData::kittycat_or_admin("lockserver", "list"),
                    "lock" => silverpelt::types::CommandExtendedData::kittycat_or_admin("lockserver", "lock"),
                    "update" => silverpelt::types::CommandExtendedData::kittycat_or_admin("lockserver", "update"),
                    "unlock" => silverpelt::types::CommandExtendedData::kittycat_or_admin("lockserver", "unlock"),
                },
            ),
        ],
        config_options: vec![
            (*settings::LOCKDOWN_SETTINGS).clone(),
            (*settings::QUICK_SERVER_LOCKDOWNS).clone(),
        ],
        ..Default::default()
    }
}

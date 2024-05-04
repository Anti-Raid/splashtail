pub mod core;
pub mod cmd;
pub mod sting_source;

use indexmap::indexmap;

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "punishments",
        name: "Punishments",
        description: "Customizable setting of punishments based on number of stings.",
        toggleable: true,
        commands_configurable: true,
        virtual_module: false,
        web_hidden: false,
        is_default_enabled: false,
        commands: vec![
            (
                cmd::punishments(),
                indexmap! {
                    "add" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("punishments", "add"),
                    "viewsources" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("punishments", "viewsources"),
                },
            ),
        ],
        on_startup: vec![],
        ..Default::default()
    }
}

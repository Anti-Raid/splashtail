pub mod cmd;
pub mod core;
pub mod sting_source;

use indexmap::indexmap;

pub fn module() -> silverpelt::Module {
    silverpelt::Module {
        id: "punishments",
        name: "Punishments",
        description: "Customizable setting and executing of punishments based on stings.",
        toggleable: true,
        commands_toggleable: true,
        virtual_module: false,
        web_hidden: false,
        is_default_enabled: true,
        commands: vec![(
            cmd::punishments(),
            indexmap! {
                "add" => silverpelt::types::CommandExtendedData::kittycat_or_admin("punishments", "add"),
                "viewsources" => silverpelt::types::CommandExtendedData::kittycat_or_admin("punishments", "viewsources"),
                "list" => silverpelt::types::CommandExtendedData::kittycat_or_admin("punishments", "list"),
                "delete" => silverpelt::types::CommandExtendedData::kittycat_or_admin("punishments", "delete"),
            },
        )],
        on_startup: vec![],
        ..Default::default()
    }
}

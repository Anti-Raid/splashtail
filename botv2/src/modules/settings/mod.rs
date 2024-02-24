use indexmap::indexmap;

mod modules;

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "settings",
        name: "Settings",
        description: "Configure the bot to your liking",
        configurable: true,
        commands_configurable: true,
        web_hidden: false,
        is_default_enabled: true,
        commands: vec![
            (
                modules::modules(),
                indexmap! {
                    "enable" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("modules", "enable"),
                    "disable" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("modules", "disable"),
                },
            ),
        ],
        event_handlers: vec![], // Settings has no event listeners
    }
}

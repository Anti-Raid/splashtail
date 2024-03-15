mod cmd;
use indexmap::indexmap;

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "moderation",
        name: "Moderation",
        description: "Basic customizable moderation plugin for your server.",
        toggleable: true,
        commands_configurable: true,
        web_hidden: false,
        is_default_enabled: true,
        commands: vec![
            (
                cmd::kick(),
                indexmap! {
                    "" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("moderation", "kick"),
                },
            ),
            (
                cmd::ban(),
                indexmap! {
                    "" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("moderation", "ban"),
                },
            )
        ],
        event_handlers: vec![],
    }
}

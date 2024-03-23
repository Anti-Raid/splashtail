mod cmd;
use indexmap::indexmap;

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "moderation",
        name: "Moderation",
        description: "Basic customizable moderation plugin for your server.",
        toggleable: true,
        commands_configurable: true,
        virtual_module: false,
        web_hidden: false,
        is_default_enabled: true,
        commands: vec![
            (
                cmd::prune_user(),
                indexmap! {
                    "" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("moderation", "prune_user"),
                },
            ),
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
            ),
            (
                cmd::tempban(),
                indexmap! {
                    "" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("moderation", "tempban"),
                },
            ),
            (
                cmd::unban(),
                indexmap! {
                    "" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("moderation", "unban"),
                },
            ),
            (
                cmd::timeout(),
                indexmap! {
                    "" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("moderation", "timeout"),
                },
            )
        ],
        event_handlers: vec![],
    }
}

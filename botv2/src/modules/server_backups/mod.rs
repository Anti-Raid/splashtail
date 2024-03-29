mod cmds;
use indexmap::indexmap;

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "server_backups",
        name: "Server Backups",
        description: "Customizable advanced server backup system for your server",
        toggleable: true,
        commands_configurable: true,
        virtual_module: false,
        web_hidden: false,
        is_default_enabled: true,
        commands: vec![(
            cmds::backups(),
            indexmap! {
                "" => crate::silverpelt::CommandExtendedData::kittycat_simple("server_backups", "*"),
                "create" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("server_backups", "create"),
                "list" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("server_backups", "list"),
                "delete" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("server_backups", "delete"),
                "restore" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("server_backups", "restore"),
            },
        )],
        event_handlers: vec![], // Root has no event listeners
        background_tasks: vec![], // No background tasks
    }
}

mod cmds;
use indexmap::indexmap;

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "server_backups",
        name: "Server Backups",
        commands: vec![
            (cmds::backups(), indexmap! {
                "" => crate::silverpelt::CommandExtendedData::kittycat_simple("server_backups", "*"),
                "create" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("server_backups", "create"),
                "list" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("server_backups", "list"),
                "restore" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("server_backups", "restore"),
            }),
        ],
        event_handlers: vec![], // Root has no event listeners
    }
}

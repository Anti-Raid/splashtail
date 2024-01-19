mod cmds;
use indexmap::indexmap;

pub fn module() -> super::Module {
    super::Module {
        id: "server_backups",
        name: "Server Backups",
        commands: vec![
            (cmds::backups(), indexmap! {
                "" => super::CommandExtendedData::kittycat_simple("backups", "*"),
                "create" => super::CommandExtendedData::kittycat_or_admin("backups", "create"),
                "list" => super::CommandExtendedData::kittycat_or_admin("backups", "list"),
                "restore" => super::CommandExtendedData::kittycat_or_admin("backups", "restore"),
            }),
        ],
        event_handlers: vec![], // Root has no event listeners
    }
}

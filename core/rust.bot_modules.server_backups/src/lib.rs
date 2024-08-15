mod cmds;
use indexmap::indexmap;

pub fn module() -> silverpelt::Module {
    silverpelt::Module {
        id: "server_backups",
        name: "Server Backups",
        description: "Customizable advanced server backup system for your server",
        toggleable: true,
        commands_toggleable: true,
        virtual_module: false,
        web_hidden: false,
        is_default_enabled: true,
        commands: vec![(
            cmds::backups(),
            indexmap! {
                "" => silverpelt::types::CommandExtendedData::kittycat_simple("server_backups", "*"),
                "create" => silverpelt::types::CommandExtendedData::kittycat_or_admin("server_backups", "create"),
                "list" => silverpelt::types::CommandExtendedData::kittycat_or_admin("server_backups", "list"),
                "delete" => silverpelt::types::CommandExtendedData::kittycat_or_admin("server_backups", "delete"),
                "restore" => silverpelt::types::CommandExtendedData::kittycat_or_admin("server_backups", "restore"),
            },
        )],
        ..Default::default()
    }
}

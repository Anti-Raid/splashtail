mod cmds;
use indexmap::indexmap;

pub struct Module;

impl silverpelt::module::Module for Module {
    fn id(&self) -> &'static str {
        "server_backups"
    }

    fn name(&self) -> &'static str {
        "Server Backups"
    }

    fn description(&self) -> &'static str {
        "Customizable advanced server backup system for your server"
    }

    fn is_default_enabled(&self) -> bool {
        true
    }

    fn raw_commands(&self) -> Vec<silverpelt::module::CommandObj> {
        vec![(
            cmds::backups(),
            indexmap! {
                "" => silverpelt::types::CommandExtendedData::kittycat_simple("server_backups", "*"),
                "create" => silverpelt::types::CommandExtendedData::kittycat_or_admin("server_backups", "create"),
                "list" => silverpelt::types::CommandExtendedData::kittycat_or_admin("server_backups", "list"),
                "delete" => silverpelt::types::CommandExtendedData::kittycat_or_admin("server_backups", "delete"),
                "restore" => silverpelt::types::CommandExtendedData::kittycat_or_admin("server_backups", "restore"),
            },
        )]
    }
}

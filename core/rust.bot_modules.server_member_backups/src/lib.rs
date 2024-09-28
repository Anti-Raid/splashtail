pub struct Module;

impl silverpelt::module::Module for Module {
    fn id(&self) -> &'static str {
        "server_member_backups"
    }

    fn name(&self) -> &'static str {
        "Server Member Backups"
    }

    fn description(&self) -> &'static str {
        "Backups members on your server to allow for them to be restored in the event of a raid, nuke or other mass member deletions."
    }

    fn raw_commands(&self) -> Vec<silverpelt::module::CommandObj> {
        vec![]
    }
}

pub mod cmd;
pub mod core;

use indexmap::indexmap;

pub struct Module;

impl silverpelt::module::Module for Module {
    fn id(&self) -> &'static str {
        "punishments"
    }

    fn name(&self) -> &'static str {
        "Punishments"
    }

    fn description(&self) -> &'static str {
        "Customizable setting and executing of punishments based on stings."
    }

    fn is_default_enabled(&self) -> bool {
        true
    }

    fn raw_commands(&self) -> Vec<silverpelt::module::CommandObj> {
        vec![(
            cmd::punishments(),
            indexmap! {
                "add" => silverpelt::types::CommandExtendedData::kittycat_or_admin("punishments", "add"),
                "list" => silverpelt::types::CommandExtendedData::kittycat_or_admin("punishments", "list"),
                "delete" => silverpelt::types::CommandExtendedData::kittycat_or_admin("punishments", "delete"),
            },
        )]
    }
}

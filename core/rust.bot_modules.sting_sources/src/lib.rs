pub mod settings;

pub struct Module;

impl silverpelt::module::Module for Module {
    fn id(&self) -> &'static str {
        "sting_sources"
    }

    fn name(&self) -> &'static str {
        "Sting Sources"
    }

    fn description(&self) -> &'static str {
        "Customizable management of sting sources"
    }

    fn web_hidden(&self) -> bool {
        true // Not yet ready for release yet
    }

    fn is_default_enabled(&self) -> bool {
        true
    }

    fn raw_commands(&self) -> Vec<silverpelt::module::CommandObj> {
        vec![]
    }
}

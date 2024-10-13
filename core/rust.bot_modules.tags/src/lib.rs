mod cmds;
mod settings;

pub struct Module;

impl silverpelt::module::Module for Module {
    fn id(&self) -> &'static str {
        "tags"
    }

    fn name(&self) -> &'static str {
        "Tags"
    }

    fn is_default_enabled(&self) -> bool {
        true
    }

    fn description(&self) -> &'static str {
        "Make custom tags for your server to allow quicker responses to commonly asked question or even more customizable actions thanks to Luau scripting!"
    }

    fn raw_commands(&self) -> Vec<silverpelt::module::CommandObj> {
        vec![(
            cmds::tag(),
            indexmap::indexmap! {
                "" => silverpelt::types::CommandExtendedData::none()
            },
        )]
    }

    fn config_options(&self) -> Vec<module_settings::types::ConfigOption> {
        vec![(*settings::CUSTOM_TAGS).clone()]
    }
}

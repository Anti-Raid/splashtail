#![allow(non_snake_case)]
mod cmds;
mod settings;

pub struct Module;

impl silverpelt::module::Module for Module {
    fn id(&self) -> &'static str {
        "root"
    }

    fn name(&self) -> &'static str {
        "Root/Staff-Only Commands"
    }

    fn description(&self) -> &'static str {
        "Commands that are only available to staff members. Publicly viewable for transparency."
    }

    fn toggleable(&self) -> bool {
        false
    }

    fn commands_toggleable(&self) -> bool {
        false
    }

    fn is_default_enabled(&self) -> bool {
        true
    }

    fn root_module(&self) -> bool {
        true
    }

    fn raw_commands(&self) -> Vec<silverpelt::module::CommandObj> {
        vec![(
            cmds::sudo(),
            indexmap::indexmap! {
                "register" => silverpelt::types::CommandExtendedData::none(),
                "inspector__fake_bots_list" => silverpelt::types::CommandExtendedData::none(),
                "inspector__fake_bots_add" => silverpelt::types::CommandExtendedData::none(),
                "inspector__fake_bots_update" => silverpelt::types::CommandExtendedData::none(),
                "inspector__fake_bots_delete" => silverpelt::types::CommandExtendedData::none(),
                "last_task_expiry_view" => silverpelt::types::CommandExtendedData::none(),
                "last_task_expiry_create" => silverpelt::types::CommandExtendedData::none(),
                "last_task_expiry_update" => silverpelt::types::CommandExtendedData::none(),
                "last_task_expiry_delete" => silverpelt::types::CommandExtendedData::none(),
            },
        )]
    }

    fn config_options(&self) -> Vec<module_settings::types::ConfigOption> {
        vec![
            (*settings::INSPECTOR_FAKE_BOTS).clone(),
            (*settings::LAST_TASK_EXPIRY).clone(),
        ]
    }
}

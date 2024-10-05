#![allow(non_snake_case)]

mod am_toggles;
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
                "can_use_bot_list" => silverpelt::types::CommandExtendedData::none(),
                "can_use_bot_add" => silverpelt::types::CommandExtendedData::none(),
                "can_use_bot_update" => silverpelt::types::CommandExtendedData::none(),
                "can_use_bot_delete" => silverpelt::types::CommandExtendedData::none(),
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

    fn event_listeners(&self) -> Option<Box<dyn silverpelt::module::ModuleEventListeners>> {
        Some(Box::new(EventListener))
    }

    fn config_options(&self) -> Vec<module_settings::types::ConfigOption> {
        vec![
            (*settings::CAN_USE_BOT).clone(),
            (*settings::INSPECTOR_FAKE_BOTS).clone(),
            (*settings::LAST_TASK_EXPIRY).clone(),
        ]
    }
}

struct EventListener;

#[async_trait::async_trait]
impl silverpelt::module::ModuleEventListeners for EventListener {
    async fn on_startup(&self, data: &silverpelt::data::Data) -> Result<(), silverpelt::Error> {
        am_toggles::setup(data).await
    }

    fn event_handler_filter(&self, _event: &silverpelt::ar_event::AntiraidEvent) -> bool {
        false // No events to filter
    }
}

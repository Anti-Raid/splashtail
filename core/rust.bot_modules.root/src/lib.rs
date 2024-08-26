#![allow(non_snake_case)]

use futures_util::FutureExt;

mod am_toggles;
mod cmds;
mod settings;

pub fn module() -> silverpelt::Module {
    silverpelt::Module {
        id: "root",
        name: "Root/Staff-Only Commands",
        description:
            "Commands that are only available to staff members. Publicly viewable for transparency.",
        toggleable: false,
        commands_toggleable: false,
        virtual_module: false,
        web_hidden: false,
        is_default_enabled: true,
        // These commands do not follow the typical permission system anyways
        commands: vec![(
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
        )],
        on_startup: vec![Box::new(move |data| am_toggles::setup(data).boxed())],
        config_options: vec![
            (*settings::CAN_USE_BOT).clone(),
            (*settings::INSPECTOR_FAKE_BOTS).clone(),
            (*settings::LAST_TASK_EXPIRY).clone(),
        ],
        root_module: true,
        ..Default::default()
    }
}

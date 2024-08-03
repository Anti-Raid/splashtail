#![allow(non_snake_case)]

use futures_util::FutureExt;

mod am_toggles;
mod cmds;
mod settings;

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "root",
        name: "Root/Staff-Only Commands",
        description: "Commands that are only available to staff members.",
        toggleable: false,
        commands_toggleable: false,
        virtual_module: false,
        web_hidden: false, // TODO: This should be true later
        is_default_enabled: true,
        // These commands do not follow the typical permission system anyways
        commands: vec![(
            cmds::sudo(),
            indexmap::indexmap! {
                "register" => crate::silverpelt::CommandExtendedData::none(),
                "can_use_bot_list" => crate::silverpelt::CommandExtendedData::none(),
                "can_use_bot_add" => crate::silverpelt::CommandExtendedData::none(),
                "can_use_bot_update" => crate::silverpelt::CommandExtendedData::none(),
                "can_use_bot_delete" => crate::silverpelt::CommandExtendedData::none(),
                "inspector__fake_bots_list" => crate::silverpelt::CommandExtendedData::none(),
                "inspector__fake_bots_add" => crate::silverpelt::CommandExtendedData::none(),
                "inspector__fake_bots_update" => crate::silverpelt::CommandExtendedData::none(),
                "inspector__fake_bots_delete" => crate::silverpelt::CommandExtendedData::none(),
            },
        )],
        on_startup: vec![Box::new(move |data| am_toggles::setup(data).boxed())],
        config_options: vec![
            (*settings::CAN_USE_BOT).clone(),
            (*settings::INSPECTOR_FAKE_BOTS).clone(),
        ],
        ..Default::default()
    }
}

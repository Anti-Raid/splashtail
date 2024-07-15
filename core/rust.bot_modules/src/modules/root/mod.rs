#![allow(non_snake_case)]

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
                "cub" => crate::silverpelt::CommandExtendedData::none(),
                "maintenance_list" => crate::silverpelt::CommandExtendedData::none(),
                "maintenance_create" => crate::silverpelt::CommandExtendedData::none(),
                "maintenance_update" => crate::silverpelt::CommandExtendedData::none(),
                "maintenance_delete" => crate::silverpelt::CommandExtendedData::none(),
                "inspector__fake_bots_list" => crate::silverpelt::CommandExtendedData::none(),
                "inspector__fake_bots_add" => crate::silverpelt::CommandExtendedData::none(),
                "inspector__fake_bots_update" => crate::silverpelt::CommandExtendedData::none(),
                "inspector__fake_bots_delete" => crate::silverpelt::CommandExtendedData::none(),
            },
        )],
        config_options: vec![
            (*settings::MAINTENANCE).clone(),
            (*settings::INSPECTOR_FAKE_BOTS).clone(),
        ],
        ..Default::default()
    }
}

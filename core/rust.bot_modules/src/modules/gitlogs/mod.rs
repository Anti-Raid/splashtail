use indexmap::indexmap;

mod core;
mod settings;

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "gitlogs",
        name: "Git Logs",
        description: "Advanced github logging for your development-related servers! Complete with event whitelists and redirecting specific events to different channels and other customizability.",
        toggleable: true,
        commands_toggleable: true,
        virtual_module: false,
        web_hidden: false,
        is_default_enabled: false,
        commands: vec![
            (core::gitlogs(), indexmap! {
                "" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("gitlogs", "*"),
                "webhooks_list" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("gitlogs", "webhooks_list"),
                "webhooks_create" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("gitlogs", "webhooks_create"),
                "webhooks_update" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("gitlogs", "webhooks_update"),
                "webhooks_delete" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("gitlogs", "webhooks_delete"),
                "repo_list" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("gitlogs", "repo_list"),
                "repo_create" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("gitlogs", "repo_create"),
                "repo_update" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("gitlogs", "repo_update"),
                "repo_delete" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("gitlogs", "repo_delete"),
                "eventmods_list" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("gitlogs", "eventmods_list"),
                "eventmods_create" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("gitlogs", "eventmods_create"),
                "eventmods_update" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("gitlogs", "eventmods_update"),
                "eventmods_delete" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("gitlogs", "eventmods_delete"),
            }),
        ],
        config_options: vec![
            settings::webhooks(),
            settings::repos(),
            settings::event_modifiers(),
        ],
        ..Default::default()
    }
}

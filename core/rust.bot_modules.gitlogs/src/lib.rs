use indexmap::indexmap;

mod core;
mod settings;

pub fn module() -> silverpelt::Module {
    silverpelt::Module {
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
                "" => silverpelt::types::CommandExtendedData::kittycat_or_admin("gitlogs", "*"),
                "webhooks_list" => silverpelt::types::CommandExtendedData::kittycat_or_admin("gitlogs", "webhooks_list"),
                "webhooks_create" => silverpelt::types::CommandExtendedData::kittycat_or_admin("gitlogs", "webhooks_create"),
                "webhooks_update" => silverpelt::types::CommandExtendedData::kittycat_or_admin("gitlogs", "webhooks_update"),
                "webhooks_delete" => silverpelt::types::CommandExtendedData::kittycat_or_admin("gitlogs", "webhooks_delete"),
                "repo_list" => silverpelt::types::CommandExtendedData::kittycat_or_admin("gitlogs", "repo_list"),
                "repo_create" => silverpelt::types::CommandExtendedData::kittycat_or_admin("gitlogs", "repo_create"),
                "repo_update" => silverpelt::types::CommandExtendedData::kittycat_or_admin("gitlogs", "repo_update"),
                "repo_delete" => silverpelt::types::CommandExtendedData::kittycat_or_admin("gitlogs", "repo_delete"),
                "eventmods_list" => silverpelt::types::CommandExtendedData::kittycat_or_admin("gitlogs", "eventmods_list"),
                "eventmods_create" => silverpelt::types::CommandExtendedData::kittycat_or_admin("gitlogs", "eventmods_create"),
                "eventmods_update" => silverpelt::types::CommandExtendedData::kittycat_or_admin("gitlogs", "eventmods_update"),
                "eventmods_delete" => silverpelt::types::CommandExtendedData::kittycat_or_admin("gitlogs", "eventmods_delete"),
            }),
        ],
        config_options: vec![
            (*settings::WEBHOOKS).clone(),
            (*settings::REPOS).clone(),
            (*settings::EVENT_MODIFIERS).clone(),
        ],
        ..Default::default()
    }
}

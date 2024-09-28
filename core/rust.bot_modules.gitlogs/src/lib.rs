use indexmap::indexmap;

mod core;
mod settings;

pub struct Module;

impl silverpelt::module::Module for Module {
    fn id(&self) -> &'static str {
        "gitlogs"
    }

    fn name(&self) -> &'static str {
        "Git Logs"
    }

    fn description(&self) -> &'static str {
        "Advanced github logging for your development-related servers! Complete with event whitelists and redirecting specific events to different channels and other customizability."
    }

    fn raw_commands(&self) -> Vec<silverpelt::module::CommandObj> {
        vec![(
            core::gitlogs(),
            indexmap! {
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
            },
        )]
    }

    fn config_options(&self) -> Vec<module_settings::types::ConfigOption> {
        vec![
            (*settings::WEBHOOKS).clone(),
            (*settings::REPOS).clone(),
            (*settings::EVENT_MODIFIERS).clone(),
        ]
    }
}

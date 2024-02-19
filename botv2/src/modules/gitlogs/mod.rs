use indexmap::indexmap;

mod backups;
mod core;
mod eventmods;

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "gitlogs",
        name: "Git Logs",
        description: "Advanced github logging for your development-related servers! Complete with event whitelists and redirecting specific events to different channels and other customizability.",
        configurable: true,
        commands_configurable: true,
        web_hidden: false,
        is_default_enabled: false,
        commands: vec![
            (core::gitlogs(), indexmap! {
                "" => crate::silverpelt::CommandExtendedData::kittycat_simple("gitlogs", "*"),
                "list" => crate::silverpelt::CommandExtendedData::kittycat_simple("gitlogs", "list"),
                "newhook" => crate::silverpelt::CommandExtendedData::kittycat_simple("gitlogs", "hooks"),
                "delhook" => crate::silverpelt::CommandExtendedData::kittycat_simple("gitlogs", "hooks"),
                "newrepo" => crate::silverpelt::CommandExtendedData::kittycat_simple("gitlogs", "repos"),
                "delrepo" => crate::silverpelt::CommandExtendedData::kittycat_simple("gitlogs", "repos"),
                "backup" => crate::silverpelt::CommandExtendedData::kittycat_simple("gitlogs", "backup_create"),
                "restore" => crate::silverpelt::CommandExtendedData::kittycat_simple("gitlogs", "backup_restore"),
                "eventmod" => crate::silverpelt::CommandExtendedData::kittycat_simple("gitlogs", "eventmod"),
            })
        ],
        event_handlers: vec![],
    }
}

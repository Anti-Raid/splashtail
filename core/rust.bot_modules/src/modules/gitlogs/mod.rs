use indexmap::indexmap;

mod core;
mod eventmods;
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
                "list" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("gitlogs", "list"),
                "newhook" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("gitlogs", "hooks"),
                "delhook" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("gitlogs", "hooks"),
            }),
            (
                eventmods::eventmods_list(),
                indexmap! {
                    "" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("gitlogs", "eventmods_list"),
                },
            ),
            (
                eventmods::eventmods_create(),
                indexmap! {
                    "" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("gitlogs", "eventmods_create"),
                },
            ),
            (
                eventmods::eventmods_update(),
                indexmap! {
                    "" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("gitlogs", "eventmods_update"),
                },
            ),
            (
                eventmods::eventmods_delete(),
                indexmap! {
                    "" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("gitlogs", "eventmods_delete"),
                },
            ),
            (
                core::repo_list(),
                indexmap! {
                    "" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("gitlogs", "repo_list"),
                },
            ),
            (
                core::repo_create(),
                indexmap! {
                    "" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("gitlogs", "repo_create"),
                },
            ),
            (
                core::repo_update(),
                indexmap! {
                    "" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("gitlogs", "repo_update"),
                },
            ),
            (
                core::repo_delete(),
                indexmap! {
                    "" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("gitlogs", "repo_delete"),
                },
            ),
        ],
        config_options: vec![
            settings::repos(),
            settings::event_modifiers(),
        ],
        ..Default::default()
    }
}

use indexmap::indexmap;

mod modules;
mod perms;
mod cmd;

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "settings",
        name: "Settings",
        description: "Configure the bot to your liking",
        toggleable: false,
        commands_configurable: true,
        virtual_module: false,
        web_hidden: false,
        is_default_enabled: true,
        commands: vec![
            (
                modules::modules(),
                indexmap! {
                    "list" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("modules", "list"),
                    "enable" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("modules", "enable"),
                    "disable" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("modules", "disable"),
                },
            ),
            (
                perms::perms(),
                indexmap! {
                    "modrole" => crate::silverpelt::CommandExtendedData {
                        default_perms: crate::silverpelt::PermissionChecks {
                            checks: vec![
                                crate::silverpelt::PermissionCheck {
                                    kittycat_perms: vec!["perms.editrole".to_string(), "perms.manage".to_string()],
                                    native_perms: vec![],
                                    inner_and: true,
                                    outer_and: false,
                                },
                                crate::silverpelt::PermissionCheck {
                                    kittycat_perms: vec![],
                                    native_perms: vec![serenity::model::permissions::Permissions::MANAGE_ROLES],
                                    inner_and: true,
                                    outer_and: false,
                                }
                            ],
                            checks_needed: 1,
                        },
                        ..Default::default()
                    },
                },
            ),
        ],
        event_handlers: vec![], // Settings has no event listeners
    }
}

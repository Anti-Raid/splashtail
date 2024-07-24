use futures::future::FutureExt;
use indexmap::indexmap;

mod am_toggles;
mod commands;
mod events;
mod guildroles;
mod modules;

#[allow(clippy::module_inception)]
mod settings;

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "settings",
        name: "Settings",
        description: "Configure the bot to your liking",
        toggleable: false,
        commands_toggleable: true,
        virtual_module: false,
        web_hidden: false,
        is_default_enabled: true,
        commands: vec![
            (
                modules::modules(),
                indexmap! {
                    "" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("modules", "*"),
                    "list" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("modules", "list"),
                    "enable" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("modules", "enable"),
                    "disable" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("modules", "disable"),
                    "modperms" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("modules", "modperms"),
                },
            ),
            (
                commands::commands(),
                indexmap! {
                    "check" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("commands", "check"),
                    "enable" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("commands", "enable"),
                    "disable" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("commands", "disable"),
                    "modperms" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("commands", "modperms"),
                },
            ),
            (
                guildroles::guildroles(),
                indexmap! {
                    "list" => crate::silverpelt::CommandExtendedData::kittycat_simple("guildroles", "list"),
                    "add" => crate::silverpelt::CommandExtendedData {
                        default_perms: crate::silverpelt::PermissionChecks::Simple {
                            checks: vec![
                                crate::silverpelt::PermissionCheck {
                                    kittycat_perms: vec!["guildroles.add".to_string()],
                                    native_perms: vec![],
                                    inner_and: false,
                                    outer_and: true,
                                },
                                crate::silverpelt::PermissionCheck {
                                    kittycat_perms: vec![],
                                    native_perms: vec![serenity::model::permissions::Permissions::MANAGE_ROLES],
                                    inner_and: false,
                                    outer_and: false,
                                }
                            ],
                        },
                        ..Default::default()
                    },
                    "edit" => crate::silverpelt::CommandExtendedData {
                        default_perms: crate::silverpelt::PermissionChecks::Simple {
                            checks: vec![
                                crate::silverpelt::PermissionCheck {
                                    kittycat_perms: vec!["guildroles.edit".to_string()],
                                    native_perms: vec![],
                                    inner_and: false,
                                    outer_and: true,
                                },
                                crate::silverpelt::PermissionCheck {
                                    kittycat_perms: vec![],
                                    native_perms: vec![serenity::model::permissions::Permissions::MANAGE_ROLES],
                                    inner_and: false,
                                    outer_and: false,
                                }
                            ],
                        },
                        ..Default::default()
                    },
                    "remove" => crate::silverpelt::CommandExtendedData::kittycat_simple("guildroles", "remove"),
                },
            ),
        ],
        on_startup: vec![Box::new(move |data| am_toggles::setup(data).boxed())],
        event_handlers: vec![Box::new(move |ectx| events::event_listener(ectx).boxed())],
        config_options: vec![(*settings::GUILD_ROLES).clone()],
        ..Default::default()
    }
}

use futures_util::future::FutureExt;
use indexmap::indexmap;
use permissions::types::{PermissionCheck, PermissionChecks};
use silverpelt::types::CommandExtendedData;

mod am_toggles;
mod commands;
mod guildroles;
mod modules;

#[allow(clippy::module_inception)]
mod settings;

pub fn module() -> silverpelt::Module {
    silverpelt::Module {
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
                    "" => CommandExtendedData::kittycat_or_admin("modules", "*"),
                    "list" => CommandExtendedData::kittycat_or_admin("modules", "list"),
                    "enable" => CommandExtendedData::kittycat_or_admin("modules", "enable"),
                    "disable" => CommandExtendedData::kittycat_or_admin("modules", "disable"),
                    "modperms" => CommandExtendedData::kittycat_or_admin("modules", "modperms"),
                },
            ),
            (
                commands::commands(),
                indexmap! {
                    "check" => silverpelt::types::CommandExtendedData::kittycat_or_admin("commands", "check"),
                    "enable" => silverpelt::types::CommandExtendedData::kittycat_or_admin("commands", "enable"),
                    "disable" => silverpelt::types::CommandExtendedData::kittycat_or_admin("commands", "disable"),
                    "modperms" => silverpelt::types::CommandExtendedData::kittycat_or_admin("commands", "modperms"),
                },
            ),
            (
                guildroles::guildroles(),
                indexmap! {
                    "list" => silverpelt::types::CommandExtendedData::kittycat_simple("guildroles", "list"),
                    "add" => silverpelt::types::CommandExtendedData {
                        default_perms: PermissionChecks::Simple {
                            checks: vec![
                                PermissionCheck {
                                    kittycat_perms: vec!["guildroles.add".to_string()],
                                    native_perms: vec![],
                                    inner_and: false,
                                    outer_and: true,
                                },
                                PermissionCheck {
                                    kittycat_perms: vec![],
                                    native_perms: vec![serenity::model::permissions::Permissions::MANAGE_ROLES],
                                    inner_and: false,
                                    outer_and: false,
                                }
                            ],
                        },
                        ..Default::default()
                    },
                    "edit" => silverpelt::types::CommandExtendedData {
                        default_perms: PermissionChecks::Simple {
                            checks: vec![
                                PermissionCheck {
                                    kittycat_perms: vec!["guildroles.edit".to_string()],
                                    native_perms: vec![],
                                    inner_and: false,
                                    outer_and: true,
                                },
                                PermissionCheck {
                                    kittycat_perms: vec![],
                                    native_perms: vec![serenity::model::permissions::Permissions::MANAGE_ROLES],
                                    inner_and: false,
                                    outer_and: false,
                                }
                            ],
                        },
                        ..Default::default()
                    },
                    "remove" => silverpelt::types::CommandExtendedData::kittycat_simple("guildroles", "remove"),
                },
            ),
        ],
        on_startup: vec![Box::new(move |data| am_toggles::setup(data).boxed())],
        config_options: vec![(*settings::GUILD_ROLES).clone()],
        ..Default::default()
    }
}

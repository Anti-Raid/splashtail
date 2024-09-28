use indexmap::indexmap;
use permissions::types::{PermissionCheck, PermissionChecks};
use silverpelt::types::CommandExtendedData;

mod am_toggles;
mod commands;
mod guildmembers;
mod guildroles;
mod modules;

#[allow(clippy::module_inception)]
mod settings;

pub struct Module;

#[async_trait::async_trait]
impl silverpelt::module::Module for Module {
    fn id(&self) -> &'static str {
        "settings"
    }

    fn name(&self) -> &'static str {
        "Settings"
    }

    fn description(&self) -> &'static str {
        "Configure the bot to your liking"
    }

    fn toggleable(&self) -> bool {
        false
    }

    fn commands_toggleable(&self) -> bool {
        true
    }

    fn is_default_enabled(&self) -> bool {
        true
    }

    fn raw_commands(&self) -> Vec<silverpelt::module::CommandObj> {
        vec![
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
            (
                guildmembers::guildmembers(),
                indexmap! {
                    "list" => silverpelt::types::CommandExtendedData::kittycat_simple("guildmembers", "list"),
                    "add" => silverpelt::types::CommandExtendedData {
                        default_perms: PermissionChecks::Simple {
                            checks: vec![
                                PermissionCheck {
                                    kittycat_perms: vec!["guildmembers.add".to_string()],
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
                                    kittycat_perms: vec!["guildmembers.edit".to_string()],
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
                    "remove" => silverpelt::types::CommandExtendedData::kittycat_simple("guildmembers", "remove"),
                },
            ),
        ]
    }

    fn config_options(&self) -> Vec<module_settings::types::ConfigOption> {
        vec![
            (*settings::GUILD_ROLES).clone(),
            (*settings::GUILD_MEMBERS).clone(),
        ]
    }

    fn event_listeners(&self) -> Option<Box<dyn silverpelt::module::ModuleEventListeners>> {
        Some(Box::new(EventHandler))
    }
}

struct EventHandler;

#[async_trait::async_trait]
impl silverpelt::module::ModuleEventListeners for EventHandler {
    async fn on_startup(&self, data: &silverpelt::data::Data) -> Result<(), silverpelt::Error> {
        am_toggles::setup(data).await
    }
}

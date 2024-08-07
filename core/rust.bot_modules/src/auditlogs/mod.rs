mod am_toggles;
mod checks;
mod cmds;
pub mod events;
mod settings;

use futures_util::future::FutureExt;
use indexmap::indexmap;
use permissions::types::{PermissionCheck, PermissionChecks};
use silverpelt::types::CommandExtendedData;

pub fn module() -> silverpelt::Module {
    silverpelt::Module {
        id: "auditlogs",
        name: "Audit Logs",
        description:
            "Customizable and comprehensive audit logging module supporting 72+ discord events",
        toggleable: true,
        commands_toggleable: true,
        virtual_module: false,
        web_hidden: false,
        is_default_enabled: false,
        commands: vec![(
            cmds::auditlogs(),
            indexmap! {
                "list_sinks" => CommandExtendedData {
                    default_perms: PermissionChecks::Simple {
                        checks: vec![
                            PermissionCheck {
                                kittycat_perms: vec!["auditlogs.list_sinks".to_string(), "auditlogs.list".to_string()],
                                native_perms: vec![],
                                inner_and: false,
                                outer_and: false,
                            },
                            PermissionCheck {
                                kittycat_perms: vec![],
                                native_perms: vec![serenity::model::permissions::Permissions::VIEW_AUDIT_LOG, serenity::model::permissions::Permissions::MANAGE_GUILD],
                                inner_and: true,
                                outer_and: false,
                            }
                        ],
                    },
                    ..Default::default()
                },
                "add_channel" => CommandExtendedData {
                    default_perms: PermissionChecks::Simple {
                        checks: vec![
                            PermissionCheck {
                                kittycat_perms: vec!["auditlogs.add_sink".to_string()],
                                native_perms: vec![],
                                inner_and: false,
                                outer_and: false,
                            },
                            PermissionCheck {
                                kittycat_perms: vec![],
                                native_perms: vec![serenity::model::permissions::Permissions::VIEW_AUDIT_LOG, serenity::model::permissions::Permissions::MANAGE_GUILD],
                                inner_and: true,
                                outer_and: false,
                            }
                        ],
                    },
                    ..Default::default()
                },
                "add_sink" => CommandExtendedData {
                    default_perms: PermissionChecks::Simple {
                        checks: vec![
                            PermissionCheck {
                                kittycat_perms: vec!["auditlogs.add_sink".to_string()],
                                native_perms: vec![],
                                inner_and: false,
                                outer_and: false,
                            },
                            PermissionCheck {
                                kittycat_perms: vec![],
                                native_perms: vec![serenity::model::permissions::Permissions::VIEW_AUDIT_LOG, serenity::model::permissions::Permissions::MANAGE_GUILD],
                                inner_and: true,
                                outer_and: false,
                            }
                        ],
                    },
                    ..Default::default()
                },
                "add_discordhook" => CommandExtendedData {
                    default_perms: PermissionChecks::Simple {
                        checks: vec![
                            PermissionCheck {
                                kittycat_perms: vec!["auditlogs.add_sink".to_string()],
                                native_perms: vec![],
                                inner_and: false,
                                outer_and: false,
                            },
                            PermissionCheck {
                                kittycat_perms: vec![],
                                native_perms: vec![serenity::model::permissions::Permissions::VIEW_AUDIT_LOG, serenity::model::permissions::Permissions::MANAGE_GUILD],
                                inner_and: true,
                                outer_and: false,
                            }
                        ],
                    },
                    ..Default::default()
                },
                "edit_sink" => CommandExtendedData {
                    default_perms: PermissionChecks::Simple {
                        checks: vec![
                            PermissionCheck {
                                kittycat_perms: vec!["auditlogs.edit_sink".to_string()],
                                native_perms: vec![],
                                inner_and: false,
                                outer_and: false,
                            },
                            PermissionCheck {
                                kittycat_perms: vec![],
                                native_perms: vec![serenity::model::permissions::Permissions::VIEW_AUDIT_LOG, serenity::model::permissions::Permissions::MANAGE_GUILD],
                                inner_and: true,
                                outer_and: false,
                            }
                        ],
                    },
                    ..Default::default()
                },
                "remove_sink" => CommandExtendedData {
                    default_perms: PermissionChecks::Simple {
                        checks: vec![
                            PermissionCheck {
                                kittycat_perms: vec!["auditlogs.remove_sink".to_string()],
                                native_perms: vec![],
                                inner_and: false,
                                outer_and: false,
                            },
                            PermissionCheck {
                                kittycat_perms: vec![],
                                native_perms: vec![serenity::model::permissions::Permissions::VIEW_AUDIT_LOG, serenity::model::permissions::Permissions::MANAGE_GUILD],
                                inner_and: true,
                                outer_and: false,
                            }
                        ],
                    },
                    ..Default::default()
                },
            },
        )],
        on_startup: vec![Box::new(move |data| am_toggles::setup(data).boxed())],
        event_handlers: vec![Box::new(move |ectx| events::event_listener(ectx).boxed())],
        config_options: vec![(*settings::SINK).clone()],
        ..Default::default()
    }
}

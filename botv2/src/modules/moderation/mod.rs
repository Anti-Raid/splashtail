mod cmd;
mod core;
mod temp_punishment_task;

use indexmap::indexmap;
use futures_util::FutureExt;

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "moderation",
        name: "Moderation",
        description: "Basic customizable moderation plugin for your server.",
        toggleable: true,
        commands_configurable: true,
        virtual_module: false,
        web_hidden: false,
        is_default_enabled: true,
        commands: vec![
            (
                cmd::prune_user(),
                indexmap! {
                    "" => crate::silverpelt::CommandExtendedData {
                        default_perms: crate::silverpelt::PermissionChecks {
                            checks: vec![
                                crate::silverpelt::PermissionCheck {
                                    kittycat_perms: vec!["moderation.prune_user".to_string(), "moderation.prune_messages".to_string()],
                                    native_perms: vec![],
                                    inner_and: false,
                                    outer_and: false,
                                },
                                crate::silverpelt::PermissionCheck {
                                    kittycat_perms: vec![],
                                    native_perms: vec![serenity::model::permissions::Permissions::MANAGE_MESSAGES, serenity::model::permissions::Permissions::MANAGE_GUILD],
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
            (
                cmd::kick(),
                indexmap! {
                    "" => crate::silverpelt::CommandExtendedData {
                        default_perms: crate::silverpelt::PermissionChecks {
                            checks: vec![
                                crate::silverpelt::PermissionCheck {
                                    kittycat_perms: vec!["moderation.kick".to_string()],
                                    native_perms: vec![],
                                    inner_and: false,
                                    outer_and: false,
                                },
                                crate::silverpelt::PermissionCheck {
                                    kittycat_perms: vec![],
                                    native_perms: vec![serenity::model::permissions::Permissions::KICK_MEMBERS],
                                    inner_and: false,
                                    outer_and: false,
                                }
                            ],
                            checks_needed: 1,
                        },
                        ..Default::default()
                    },
                },
            ),
            (
                cmd::ban(),
                indexmap! {
                    "" => crate::silverpelt::CommandExtendedData {
                        default_perms: crate::silverpelt::PermissionChecks {
                            checks: vec![
                                crate::silverpelt::PermissionCheck {
                                    kittycat_perms: vec!["moderation.ban".to_string()],
                                    native_perms: vec![],
                                    inner_and: false,
                                    outer_and: false,
                                },
                                crate::silverpelt::PermissionCheck {
                                    kittycat_perms: vec![],
                                    native_perms: vec![serenity::model::permissions::Permissions::BAN_MEMBERS],
                                    inner_and: false,
                                    outer_and: false,
                                }
                            ],
                            checks_needed: 1,
                        },
                        ..Default::default()
                    },
                },
            ),
            (
                cmd::tempban(),
                indexmap! {
                    "" => crate::silverpelt::CommandExtendedData {
                        default_perms: crate::silverpelt::PermissionChecks {
                            checks: vec![
                                crate::silverpelt::PermissionCheck {
                                    kittycat_perms: vec!["moderation.tempban".to_string()],
                                    native_perms: vec![],
                                    inner_and: false,
                                    outer_and: false,
                                },
                                crate::silverpelt::PermissionCheck {
                                    kittycat_perms: vec![],
                                    native_perms: vec![serenity::model::permissions::Permissions::BAN_MEMBERS],
                                    inner_and: false,
                                    outer_and: false,
                                }
                            ],
                            checks_needed: 1,
                        },
                        ..Default::default()
                    },
                },
            ),
            (
                cmd::unban(),
                indexmap! {
                    "" => crate::silverpelt::CommandExtendedData {
                        default_perms: crate::silverpelt::PermissionChecks {
                            checks: vec![
                                crate::silverpelt::PermissionCheck {
                                    kittycat_perms: vec!["moderation.unban".to_string()],
                                    native_perms: vec![],
                                    inner_and: false,
                                    outer_and: false,
                                },
                                crate::silverpelt::PermissionCheck {
                                    kittycat_perms: vec![],
                                    native_perms: vec![serenity::model::permissions::Permissions::BAN_MEMBERS],
                                    inner_and: false,
                                    outer_and: false,
                                }
                            ],
                            checks_needed: 1,
                        },
                        ..Default::default()
                    },
                },
            ),
            (
                cmd::timeout(),
                indexmap! {
                    "" => crate::silverpelt::CommandExtendedData {
                        default_perms: crate::silverpelt::PermissionChecks {
                            checks: vec![
                                crate::silverpelt::PermissionCheck {
                                    kittycat_perms: vec!["moderation.timeout".to_string()],
                                    native_perms: vec![],
                                    inner_and: false,
                                    outer_and: false,
                                },
                                crate::silverpelt::PermissionCheck {
                                    kittycat_perms: vec![],
                                    native_perms: vec![serenity::model::permissions::Permissions::MODERATE_MEMBERS],
                                    inner_and: false,
                                    outer_and: false,
                                }
                            ],
                            checks_needed: 1,
                        },
                        ..Default::default()
                    },
                },
            )
        ],
        background_tasks: vec![
            botox::taskman::Task {
                name: "Temporary Punishment Task",
                description: "Handle expired punishments",
                duration: std::time::Duration::from_secs(1),
                enabled: true,
                run: Box::new(move |ctx| {
                    temp_punishment_task::temp_punishment(ctx).boxed()
                }),
            }
        ],
        ..Default::default()
    }
}

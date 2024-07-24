mod cmd;
mod core;

use futures_util::FutureExt;
use indexmap::indexmap;

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "moderation",
        name: "Moderation",
        description: "Basic customizable moderation plugin for your server.",
        toggleable: true,
        commands_toggleable: true,
        virtual_module: false,
        web_hidden: false,
        is_default_enabled: true,
        commands: vec![
            (
                cmd::prune_user(),
                indexmap! {
                    "" => crate::silverpelt::CommandExtendedData {
                        default_perms: crate::silverpelt::PermissionChecks::Simple {
                            checks: vec![
                                crate::silverpelt::PermissionCheck {
                                    kittycat_perms: vec!["moderation.prune_user".to_string()],
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
                        },
                        ..Default::default()
                    },
                },
            ),
            (
                cmd::kick(),
                indexmap! {
                    "" => crate::silverpelt::CommandExtendedData {
                        default_perms: crate::silverpelt::PermissionChecks::Simple {
                            checks: vec![
                                crate::silverpelt::PermissionCheck {
                                    kittycat_perms: vec!["moderation.kick".to_string()],
                                    native_perms: vec![serenity::model::permissions::Permissions::KICK_MEMBERS],
                                    inner_and: false,
                                    outer_and: false,
                                },
                            ],
                        },
                        ..Default::default()
                    },
                },
            ),
            (
                cmd::ban(),
                indexmap! {
                    "" => crate::silverpelt::CommandExtendedData {
                        default_perms: crate::silverpelt::PermissionChecks::Simple {
                            checks: vec![
                                crate::silverpelt::PermissionCheck {
                                    kittycat_perms: vec!["moderation.ban".to_string()],
                                    native_perms: vec![serenity::model::permissions::Permissions::BAN_MEMBERS],
                                    inner_and: false,
                                    outer_and: false,
                                },
                            ],
                        },
                        ..Default::default()
                    },
                },
            ),
            (
                cmd::tempban(),
                indexmap! {
                    "" => crate::silverpelt::CommandExtendedData {
                        default_perms: crate::silverpelt::PermissionChecks::Simple {
                            checks: vec![
                                crate::silverpelt::PermissionCheck {
                                    kittycat_perms: vec!["moderation.tempban".to_string()],
                                    native_perms: vec![serenity::model::permissions::Permissions::BAN_MEMBERS],
                                    inner_and: false,
                                    outer_and: false,
                                },
                            ],
                        },
                        ..Default::default()
                    },
                },
            ),
            (
                cmd::unban(),
                indexmap! {
                    "" => crate::silverpelt::CommandExtendedData {
                        default_perms: crate::silverpelt::PermissionChecks::Simple {
                            checks: vec![
                                crate::silverpelt::PermissionCheck {
                                    kittycat_perms: vec!["moderation.unban".to_string()],
                                    native_perms: vec![serenity::model::permissions::Permissions::BAN_MEMBERS],
                                    inner_and: false,
                                    outer_and: false,
                                },
                            ],
                        },
                        ..Default::default()
                    },
                },
            ),
            (
                cmd::timeout(),
                indexmap! {
                    "" => crate::silverpelt::CommandExtendedData {
                        default_perms: crate::silverpelt::PermissionChecks::Simple {
                            checks: vec![
                                crate::silverpelt::PermissionCheck {
                                    kittycat_perms: vec!["moderation.timeout".to_string()],
                                    native_perms: vec![serenity::model::permissions::Permissions::MODERATE_MEMBERS],
                                    inner_and: false,
                                    outer_and: false,
                                },
                            ],
                        },
                        ..Default::default()
                    },
                },
            ),
        ],
        on_startup: vec![
            Box::new(move |data| core::register_punishment_sting_source(data).boxed()),
            Box::new(move |data| core::register_temporary_punishment_source(data).boxed()),
        ],
        ..Default::default()
    }
}

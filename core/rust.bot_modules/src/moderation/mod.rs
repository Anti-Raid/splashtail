mod cmd;
mod core;

use futures_util::future::FutureExt;
use indexmap::indexmap;
use permissions::types::{PermissionCheck, PermissionChecks};
use silverpelt::types::CommandExtendedData;

pub fn module() -> silverpelt::Module {
    silverpelt::Module {
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
                    "" => CommandExtendedData {
                        default_perms: PermissionChecks::Simple {
                            checks: vec![
                                PermissionCheck {
                                    kittycat_perms: vec!["moderation.prune_user".to_string()],
                                    native_perms: vec![],
                                    inner_and: false,
                                    outer_and: false,
                                },
                                PermissionCheck {
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
                    "" => CommandExtendedData {
                        default_perms: PermissionChecks::Simple {
                            checks: vec![
                                PermissionCheck {
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
                    "" => CommandExtendedData {
                        default_perms: PermissionChecks::Simple {
                            checks: vec![
                                PermissionCheck {
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
                    "" => CommandExtendedData {
                        default_perms: PermissionChecks::Simple {
                            checks: vec![
                                PermissionCheck {
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
                    "" => CommandExtendedData {
                        default_perms: PermissionChecks::Simple {
                            checks: vec![
                                PermissionCheck {
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
                    "" => CommandExtendedData {
                        default_perms: PermissionChecks::Simple {
                            checks: vec![
                                PermissionCheck {
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

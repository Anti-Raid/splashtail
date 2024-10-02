mod cmd;
mod core;

use indexmap::indexmap;
use permissions::types::{PermissionCheck, PermissionChecks};
use silverpelt::types::CommandExtendedData;

pub struct Module;

impl silverpelt::module::Module for Module {
    fn id(&self) -> &'static str {
        "moderation"
    }

    fn name(&self) -> &'static str {
        "Moderation"
    }

    fn description(&self) -> &'static str {
        "Simple yet customizable moderation plugin for your server."
    }

    fn is_default_enabled(&self) -> bool {
        true
    }

    fn raw_commands(&self) -> Vec<silverpelt::module::CommandObj> {
        vec![
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
        ]
    }

    fn punishment_actions(
        &self,
    ) -> Vec<std::sync::Arc<dyn silverpelt::punishments::CreatePunishmentAction>> {
        vec![
            std::sync::Arc::new(core::punishment_actions::CreateTimeoutAction),
            std::sync::Arc::new(core::punishment_actions::CreateKickAction),
            std::sync::Arc::new(core::punishment_actions::CreateBanAction),
            std::sync::Arc::new(core::punishment_actions::CreateRemoveAllRolesAction),
        ]
    }
}

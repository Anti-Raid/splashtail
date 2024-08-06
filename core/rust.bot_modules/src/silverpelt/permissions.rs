use super::{
    CommandExtendedData, GuildCommandConfiguration, GuildModuleConfiguration, PermissionCheck,
    PermissionChecks,
};
use splashcore_rs::permissions::check_perms_single;
use splashcore_rs::types::silverpelt::PermissionResult;

/// Executes `PermissionChecks::Simple` against the member's native permissions and kittycat permissions
fn simple_permission_checks(
    checks: &[PermissionCheck],
    member_native_perms: serenity::all::Permissions,
    member_kittycat_perms: Vec<kittycat::perms::Permission>,
) -> PermissionResult {
    let mut remaining_checks = std::collections::VecDeque::with_capacity(checks.len());

    for check in checks {
        remaining_checks.push_back(check);
    }

    while let Some(check) = remaining_checks.pop_front() {
        // Run the check
        let res = check_perms_single(check, member_native_perms, &member_kittycat_perms);

        if check.outer_and {
            let next = match remaining_checks.pop_front() {
                Some(next) => next,
                None => return res,
            };

            let res_next = check_perms_single(next, member_native_perms, &member_kittycat_perms);

            if !res.is_ok() || !res_next.is_ok() {
                return PermissionResult::NoChecksSucceeded {
                    checks: PermissionChecks::Simple {
                        checks: vec![check.clone(), next.clone()],
                    },
                };
            }
        } else {
            if res.is_ok() {
                return res;
            }

            let next = match remaining_checks.pop_front() {
                Some(next) => next,
                None => return res,
            };

            let res_next = check_perms_single(next, member_native_perms, &member_kittycat_perms);

            if res_next.is_ok() {
                return res_next;
            }
        }
    }

    PermissionResult::Ok {}
}

#[derive(Default, Clone, Eq, PartialEq, Debug)]
pub struct PermissionChecksContext {
    pub user_id: serenity::all::UserId,
    pub guild_id: serenity::all::GuildId,
    pub guild_owner_id: serenity::all::UserId,
    pub channel_id: Option<serenity::all::ChannelId>,
}

async fn template_permission_checks(
    template: &str,
    member_native_perms: serenity::all::Permissions,
    member_kittycat_perms: Vec<kittycat::perms::Permission>,
    ctx: PermissionChecksContext,
) -> PermissionResult {
    templating::render_permissions_template(
        ctx.guild_id,
        template,
        templating::core::PermissionTemplateContext {
            member_native_permissions: member_native_perms,
            member_kittycat_permissions: member_kittycat_perms,
            user_id: ctx.user_id,
            guild_id: ctx.guild_id,
            guild_owner_id: ctx.guild_owner_id,
            channel_id: ctx.channel_id,
        },
        templating::CompileTemplateOptions {
            cache_result: true,
            ignore_cache: false,
        },
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn can_run_command(
    cmd_data: &CommandExtendedData,
    command_config: &GuildCommandConfiguration,
    module_config: &GuildModuleConfiguration,
    cmd_qualified_name: &str,
    member_native_perms: serenity::all::Permissions,
    member_kittycat_perms: Vec<kittycat::perms::Permission>,
    module_is_default_enabled: bool,
    perms_ctx: PermissionChecksContext,
) -> PermissionResult {
    log::debug!(
        "Command config: {:?} [{}]",
        command_config,
        cmd_qualified_name
    );

    if command_config
        .disabled
        .unwrap_or(!cmd_data.is_default_enabled)
    {
        return PermissionResult::CommandDisabled {
            command_config: command_config.clone(),
        };
    }

    if module_config.disabled.unwrap_or(!module_is_default_enabled) {
        return PermissionResult::ModuleDisabled {
            module_config: module_config.clone(),
        };
    }

    // Check:
    // - command_config.perms
    // - module_config.default_perms
    // - cmd_data.default_perms

    let perms = {
        if let Some(perms) = &command_config.perms {
            perms
        } else if let Some(perms) = &module_config.default_perms {
            perms
        } else {
            &cmd_data.default_perms
        }
    };

    match perms {
        PermissionChecks::Simple { checks } => {
            if checks.is_empty() {
                return PermissionResult::Ok {};
            }

            simple_permission_checks(checks, member_native_perms, member_kittycat_perms)
        }
        PermissionChecks::Template { template } => {
            if template.is_empty() {
                return PermissionResult::Ok {};
            }

            template_permission_checks(
                template,
                member_native_perms,
                member_kittycat_perms,
                perms_ctx,
            )
            .await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::silverpelt::*;

    /// Generates a module configuration with the given name
    fn gen_module_config(name: &str) -> GuildModuleConfiguration {
        GuildModuleConfiguration {
            id: "".to_string(),
            guild_id: "testing".into(),
            module: name.into(),
            disabled: None,
            default_perms: None,
        }
    }

    fn err_with_code(e: PermissionResult, code: &str) -> bool {
        let code_got = e.code();
        println!("test_check_perms_single: {} == {}", code_got, code);
        code == code_got
    }

    #[test]
    fn test_check_perms_single() {
        // Basic tests
        assert!(err_with_code(
            check_perms_single(
                &PermissionCheck {
                    kittycat_perms: vec![],
                    native_perms: vec![serenity::all::Permissions::ADMINISTRATOR],
                    outer_and: false,
                    inner_and: false,
                },
                serenity::all::Permissions::empty(),
                &["abc.test".into()],
            ),
            "missing_any_perms"
        ));

        assert!(check_perms_single(
            &PermissionCheck {
                kittycat_perms: vec![],
                native_perms: vec![],
                outer_and: false,
                inner_and: false,
            },
            serenity::all::Permissions::empty(),
            &["abc.test".into()],
        )
        .is_ok());

        // With inner and
        assert!(err_with_code(
            check_perms_single(
                &PermissionCheck {
                    kittycat_perms: vec![],
                    native_perms: vec![
                        serenity::all::Permissions::ADMINISTRATOR,
                        serenity::all::Permissions::BAN_MEMBERS
                    ],
                    outer_and: false,
                    inner_and: true,
                },
                serenity::all::Permissions::BAN_MEMBERS,
                &["abc.test".into()],
            ),
            "missing_native_perms"
        ));

        // Admin overrides other native perms
        assert!(check_perms_single(
            &PermissionCheck {
                kittycat_perms: vec![],
                native_perms: vec![serenity::all::Permissions::BAN_MEMBERS],
                outer_and: false,
                inner_and: false,
            },
            serenity::all::Permissions::ADMINISTRATOR,
            &["abc.test".into()],
        )
        .is_ok());

        // Kittycat
        assert!(err_with_code(
            check_perms_single(
                &PermissionCheck {
                    kittycat_perms: vec!["backups.create".to_string()],
                    native_perms: vec![],
                    outer_and: false,
                    inner_and: false,
                },
                serenity::all::Permissions::ADMINISTRATOR,
                &[],
            ),
            "missing_any_perms"
        ));
    }

    #[tokio::test]
    async fn test_can_run_command() {
        // Basic test
        assert!(can_run_command(
            &CommandExtendedData::none_map().get("").unwrap().clone(),
            &GuildCommandConfiguration {
                id: "test".into(),
                guild_id: "test".into(),
                command: "test".into(),
                perms: None,
                disabled: None,
            },
            &gen_module_config("core"),
            "test",
            serenity::all::Permissions::empty(),
            vec!["abc.test".into()],
            true,
            PermissionChecksContext::default()
        )
        .await
        .is_ok());

        // With a native permission
        assert!(err_with_code(
            can_run_command(
                &CommandExtendedData::none_map().get("").unwrap().clone(),
                &GuildCommandConfiguration {
                    id: "test".into(),
                    guild_id: "test".into(),
                    command: "test".into(),
                    perms: Some(PermissionChecks::Simple {
                        checks: vec![PermissionCheck {
                            kittycat_perms: vec![],
                            native_perms: vec![serenity::all::Permissions::ADMINISTRATOR],
                            outer_and: false,
                            inner_and: false,
                        }],
                    }),
                    disabled: None,
                },
                &gen_module_config("core"),
                "test",
                serenity::all::Permissions::empty(),
                vec!["abc.test".into()],
                true,
                PermissionChecksContext::default()
            )
            .await,
            "no_checks_succeeded"
        ));

        assert!(err_with_code(
            can_run_command(
                &CommandExtendedData::none_map().get("").unwrap().clone(),
                &GuildCommandConfiguration {
                    id: "test".into(),
                    guild_id: "test".into(),
                    command: "test".into(),
                    perms: Some(PermissionChecks::Simple {
                        checks: vec![PermissionCheck {
                            kittycat_perms: vec![],
                            native_perms: vec![serenity::all::Permissions::ADMINISTRATOR],
                            outer_and: false,
                            inner_and: false,
                        }],
                    }),
                    disabled: None,
                },
                &gen_module_config("core"),
                "test",
                serenity::all::Permissions::empty(),
                vec!["abc.test".into()],
                true,
                PermissionChecksContext::default()
            )
            .await,
            "no_checks_succeeded"
        ));

        assert!(err_with_code(
            can_run_command(
                &CommandExtendedData::none_map().get("").unwrap().clone(),
                &GuildCommandConfiguration {
                    id: "test".into(),
                    guild_id: "test".into(),
                    command: "test".into(),
                    perms: Some(PermissionChecks::Simple {
                        checks: vec![
                            PermissionCheck {
                                kittycat_perms: vec![],
                                native_perms: vec![serenity::all::Permissions::BAN_MEMBERS],
                                outer_and: false,
                                inner_and: true,
                            },
                            PermissionCheck {
                                kittycat_perms: vec![],
                                native_perms: vec![serenity::all::Permissions::KICK_MEMBERS],
                                outer_and: false,
                                inner_and: false,
                            },
                        ],
                    }),
                    disabled: None,
                },
                &gen_module_config("core"),
                "test",
                serenity::all::Permissions::BAN_MEMBERS,
                vec!["abc.test".into()],
                true,
                PermissionChecksContext::default()
            )
            .await,
            "no_checks_succeeded"
        ));

        // Real-life example
        assert!(err_with_code(
            can_run_command(
                &CommandExtendedData::kittycat_simple("backups", "create"),
                &GuildCommandConfiguration {
                    id: "test".into(),
                    guild_id: "test".into(),
                    command: "test".into(),
                    perms: None,
                    disabled: None,
                },
                &gen_module_config("core"),
                "backups create",
                serenity::all::Permissions::ADMINISTRATOR,
                vec![],
                true,
                PermissionChecksContext::default()
            )
            .await,
            "no_checks_succeeded"
        ));

        // Real-life example
        assert!(can_run_command(
            &CommandExtendedData::kittycat_or_admin("backups", "create"),
            &GuildCommandConfiguration {
                id: "test".into(),
                guild_id: "test".into(),
                command: "test".into(),
                perms: None,
                disabled: None,
            },
            &gen_module_config("core"),
            "backups create",
            serenity::all::Permissions::ADMINISTRATOR,
            vec![],
            true,
            PermissionChecksContext::default()
        )
        .await
        .is_ok());

        assert!(can_run_command(
            &CommandExtendedData::none_map().get("").unwrap().clone(),
            &GuildCommandConfiguration {
                id: "test".into(),
                guild_id: "test".into(),
                command: "test".into(),
                perms: Some(PermissionChecks::Simple {
                    checks: vec![
                        PermissionCheck {
                            kittycat_perms: vec![],
                            native_perms: vec![serenity::all::Permissions::BAN_MEMBERS],
                            outer_and: false,
                            inner_and: false,
                        },
                        PermissionCheck {
                            kittycat_perms: vec![],
                            native_perms: vec![serenity::all::Permissions::KICK_MEMBERS],
                            outer_and: false,
                            inner_and: false,
                        },
                    ],
                }),
                disabled: None,
            },
            &gen_module_config("core"),
            "test",
            serenity::all::Permissions::BAN_MEMBERS,
            vec!["abc.test".into()],
            true,
            PermissionChecksContext::default()
        )
        .await
        .is_ok());

        // Check: module default_perms
        // Real-life example
        assert!({
            let r = can_run_command(
                &CommandExtendedData::kittycat_or_admin("test", "abc"),
                &GuildCommandConfiguration {
                    id: "test".into(),
                    guild_id: "test".into(),
                    command: "test".into(),
                    perms: None,
                    disabled: None,
                },
                &GuildModuleConfiguration {
                    id: "".to_string(),
                    guild_id: "testing".into(),
                    module: "auditlogs".to_string(),
                    disabled: Some(false),
                    default_perms: Some(PermissionChecks::Simple {
                        checks: vec![PermissionCheck {
                            kittycat_perms: vec![],
                            native_perms: vec![serenity::all::Permissions::VIEW_AUDIT_LOG],
                            outer_and: false,
                            inner_and: false,
                        }],
                    }),
                },
                "test abc",
                serenity::all::Permissions::VIEW_AUDIT_LOG,
                vec![],
                true,
                PermissionChecksContext::default(),
            )
            .await;

            println!("{}", r.code());

            r
        }
        .is_ok());
    }
}

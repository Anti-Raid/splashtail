use splashcore_rs::types::silverpelt::{PermissionCheck, PermissionChecks, PermissionResult};

/// This function runs a single permission check on a command without taking any branching decisions
///
/// This may be useful when mocking or visualizing a permission check
pub fn check_perms_single(
    check: &PermissionCheck,
    member_native_perms: serenity::all::Permissions,
    member_kittycat_perms: &[kittycat::perms::Permission],
) -> PermissionResult {
    if check.kittycat_perms.is_empty() && check.native_perms.is_empty() {
        return PermissionResult::Ok {}; // Short-circuit if we don't have any permissions to check
    }

    // Check if we have ADMINISTRATOR
    let is_discord_admin = member_native_perms.contains(serenity::all::Permissions::ADMINISTRATOR);

    // Kittycat
    if check.inner_and {
        // inner AND, short-circuit if we don't have the permission
        for perm in &check.kittycat_perms {
            if !kittycat::perms::has_perm(
                member_kittycat_perms,
                &kittycat::perms::Permission::from_string(perm),
            ) {
                return PermissionResult::MissingKittycatPerms {
                    check: check.clone(),
                };
            }
        }

        if !is_discord_admin {
            for perm in &check.native_perms {
                if !member_native_perms.contains(*perm) {
                    return PermissionResult::MissingNativePerms {
                        check: check.clone(),
                    };
                }
            }
        }
    } else {
        // inner OR, short-circuit if we have the permission
        let has_any_np = check
            .native_perms
            .iter()
            .any(|perm| is_discord_admin || member_native_perms.contains(*perm));

        if !has_any_np {
            let has_any_kc = {
                let mut has_kc = false;
                for perm in check.kittycat_perms.iter() {
                    let kc = kittycat::perms::Permission::from_string(perm);

                    if kittycat::perms::has_perm(member_kittycat_perms, &kc) {
                        has_kc = true;
                        break;
                    }
                }

                has_kc
            };

            if !has_any_kc {
                return PermissionResult::MissingAnyPerms {
                    check: check.clone(),
                };
            }
        }
    }

    PermissionResult::Ok {}
}

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
    member_native_perms: serenity::all::Permissions,
    member_kittycat_perms: Vec<kittycat::perms::Permission>,
    perms: &PermissionChecks,
    perms_ctx: PermissionChecksContext,
) -> PermissionResult {
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
            serenity::all::Permissions::empty(),
            vec!["abc.test".into()],
            &PermissionChecks::Simple {
                checks: vec![PermissionCheck::default()],
            },
            PermissionChecksContext::default()
        )
        .await
        .is_ok());

        // With a native permission
        assert!(err_with_code(
            can_run_command(
                serenity::all::Permissions::empty(),
                vec!["abc.test".into()],
                &PermissionChecks::Simple {
                    checks: vec![PermissionCheck {
                        kittycat_perms: vec![],
                        native_perms: vec![serenity::all::Permissions::ADMINISTRATOR],
                        outer_and: false,
                        inner_and: false,
                    }],
                },
                PermissionChecksContext::default()
            )
            .await,
            "missing_any_perms"
        ));

        assert!(err_with_code(
            can_run_command(
                serenity::all::Permissions::empty(),
                vec!["abc.test".into()],
                &PermissionChecks::Simple {
                    checks: vec![PermissionCheck {
                        kittycat_perms: vec![],
                        native_perms: vec![serenity::all::Permissions::ADMINISTRATOR],
                        outer_and: false,
                        inner_and: false,
                    }],
                },
                PermissionChecksContext::default()
            )
            .await,
            "missing_any_perms"
        ));

        assert!(err_with_code(
            can_run_command(
                serenity::all::Permissions::BAN_MEMBERS,
                vec!["abc.test".into()],
                &PermissionChecks::Simple {
                    checks: vec![
                        PermissionCheck {
                            kittycat_perms: vec![],
                            native_perms: vec![serenity::all::Permissions::BAN_MEMBERS],
                            outer_and: true,
                            inner_and: false,
                        },
                        PermissionCheck {
                            kittycat_perms: vec![],
                            native_perms: vec![serenity::all::Permissions::KICK_MEMBERS],
                            outer_and: false,
                            inner_and: false,
                        },
                    ],
                },
                PermissionChecksContext::default()
            )
            .await,
            "no_checks_succeeded"
        ));

        // Real-life example
        assert!(err_with_code(
            can_run_command(
                serenity::all::Permissions::ADMINISTRATOR,
                vec![],
                &PermissionChecks::Simple {
                    checks: vec![PermissionCheck {
                        kittycat_perms: vec!["backups.create".to_string()],
                        native_perms: vec![],
                        outer_and: false,
                        inner_and: false,
                    },],
                },
                PermissionChecksContext::default()
            )
            .await,
            "missing_any_perms"
        ));

        // Real-life example
        assert!(can_run_command(
            serenity::all::Permissions::ADMINISTRATOR,
            vec!["backups.create".into()],
            &PermissionChecks::Simple {
                checks: vec![PermissionCheck {
                    kittycat_perms: vec!["backups.create".to_string()],
                    native_perms: vec![serenity::all::Permissions::ADMINISTRATOR],
                    outer_and: false,
                    inner_and: true,
                },],
            },
            PermissionChecksContext::default()
        )
        .await
        .is_ok());

        assert!(can_run_command(
            serenity::all::Permissions::BAN_MEMBERS,
            vec!["abc.test".into()],
            &PermissionChecks::Simple {
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
            },
            PermissionChecksContext::default()
        )
        .await
        .is_ok());
    }
}

pub mod types;

use types::{PermissionCheck, PermissionChecks, PermissionResult};

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

/// Executes a set of PermissionCheck against the member's native permissions and kittycat permissions
pub fn eval_checks(
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
    async fn test_eval_checks() {
        // Basic test
        assert!(eval_checks(
            &[PermissionCheck::default()],
            serenity::all::Permissions::empty(),
            vec!["abc.test".into()],
        )
        .is_ok());

        // With a native permission
        assert!(err_with_code(
            eval_checks(
                &[PermissionCheck {
                    kittycat_perms: vec![],
                    native_perms: vec![serenity::all::Permissions::ADMINISTRATOR],
                    outer_and: false,
                    inner_and: false,
                }],
                serenity::all::Permissions::empty(),
                vec!["abc.test".into()],
            ),
            "missing_any_perms"
        ));

        assert!(err_with_code(
            eval_checks(
                &[PermissionCheck {
                    kittycat_perms: vec![],
                    native_perms: vec![serenity::all::Permissions::ADMINISTRATOR],
                    outer_and: false,
                    inner_and: false,
                }],
                serenity::all::Permissions::empty(),
                vec!["abc.test".into()],
            ),
            "missing_any_perms"
        ));

        assert!(err_with_code(
            eval_checks(
                &[
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
                serenity::all::Permissions::BAN_MEMBERS,
                vec!["abc.test".into()],
            ),
            "no_checks_succeeded"
        ));

        // Real-life example
        assert!(err_with_code(
            eval_checks(
                &[PermissionCheck {
                    kittycat_perms: vec!["backups.create".to_string()],
                    native_perms: vec![],
                    outer_and: false,
                    inner_and: false,
                }],
                serenity::all::Permissions::ADMINISTRATOR,
                vec![],
            ),
            "missing_any_perms"
        ));

        // Real-life example
        assert!(eval_checks(
            &[PermissionCheck {
                kittycat_perms: vec!["backups.create".to_string()],
                native_perms: vec![serenity::all::Permissions::ADMINISTRATOR],
                outer_and: false,
                inner_and: true,
            }],
            serenity::all::Permissions::ADMINISTRATOR,
            vec!["backups.create".into()],
        )
        .is_ok());

        assert!(eval_checks(
            &[
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
            serenity::all::Permissions::BAN_MEMBERS,
            vec!["abc.test".into()],
        )
        .is_ok());
    }
}

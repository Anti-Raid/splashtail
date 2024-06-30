use super::{
    silverpelt_cache::SILVERPELT_CACHE, CommandExtendedData, GuildCommandConfiguration,
    GuildModuleConfiguration, PermissionCheck, PermissionChecks,
};
use splashcore_rs::types::silverpelt::PermissionResult;

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

/// Executes `PermissionChecks` against the member's native permissions and kittycat permissions
pub fn run_permission_checks(
    perms: &PermissionChecks,
    member_native_perms: serenity::all::Permissions,
    member_kittycat_perms: &[kittycat::perms::Permission],
) -> PermissionResult {
    // This stores whether or not we need to check the next permission AND the current one or OR the current one
    let mut outer_and = false;
    let mut success: usize = 0;

    for check in &perms.checks {
        // Run the check
        let res = check_perms_single(check, member_native_perms, member_kittycat_perms);

        if outer_and {
            // Question mark needs cloning which may harm performance
            if !res.is_ok() {
                return res;
            }

            // AND yet check_perms_single returned an error, so we can short-circuit and checks_needed
            if success >= perms.checks_needed {
                return res;
            }
        } else {
            // OR, so we can short-circuit if we have the permission and checks_needed
            if res.is_ok() && success >= perms.checks_needed {
                return res;
            }
        }

        if res.is_ok() {
            success += 1;
        }

        // Set the outer AND to the new outer AND
        outer_and = check.outer_and;
    }

    // If we have no successful checks, return the error
    if success == 0 {
        return PermissionResult::NoChecksSucceeded {
            checks: perms.clone(),
        };
    }

    if success < perms.checks_needed {
        return PermissionResult::MissingMinChecks {
            checks: perms.clone(),
        };
    }

    PermissionResult::Ok {}
}

pub fn can_run_command(
    cmd_data: &CommandExtendedData,
    command_config: &GuildCommandConfiguration,
    module_config: &GuildModuleConfiguration,
    cmd_qualified_name: &str,
    member_native_perms: serenity::all::Permissions,
    member_kittycat_perms: &[kittycat::perms::Permission],
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

    {
        let Some(module) = SILVERPELT_CACHE.module_cache.get(&module_config.module) else {
            return PermissionResult::UnknownModule {
                module_config: module_config.clone(),
            };
        };

        if module_config.disabled.unwrap_or(!module.is_default_enabled) {
            return PermissionResult::ModuleDisabled {
                module_config: module_config.clone(),
            };
        }
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

    if perms.checks.is_empty() {
        return PermissionResult::Ok {};
    }

    run_permission_checks(perms, member_native_perms, member_kittycat_perms)
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

    #[test]
    fn test_can_run_command() {
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
            &["abc.test".into()],
        )
        .is_ok());

        // With a native permission
        assert!(err_with_code(
            can_run_command(
                &CommandExtendedData::none_map().get("").unwrap().clone(),
                &GuildCommandConfiguration {
                    id: "test".into(),
                    guild_id: "test".into(),
                    command: "test".into(),
                    perms: Some(PermissionChecks {
                        checks: vec![PermissionCheck {
                            kittycat_perms: vec![],
                            native_perms: vec![serenity::all::Permissions::ADMINISTRATOR],
                            outer_and: false,
                            inner_and: false,
                        }],
                        checks_needed: 0,
                    }),
                    disabled: None,
                },
                &gen_module_config("core"),
                "test",
                serenity::all::Permissions::empty(),
                &["abc.test".into()],
            ),
            "no_checks_succeeded"
        ));

        assert!(err_with_code(
            can_run_command(
                &CommandExtendedData::none_map().get("").unwrap().clone(),
                &GuildCommandConfiguration {
                    id: "test".into(),
                    guild_id: "test".into(),
                    command: "test".into(),
                    perms: Some(PermissionChecks {
                        checks: vec![PermissionCheck {
                            kittycat_perms: vec![],
                            native_perms: vec![serenity::all::Permissions::ADMINISTRATOR],
                            outer_and: false,
                            inner_and: false,
                        }],
                        checks_needed: 1,
                    }),
                    disabled: None,
                },
                &gen_module_config("core"),
                "test",
                serenity::all::Permissions::empty(),
                &["abc.test".into()],
            ),
            "no_checks_succeeded"
        ));

        assert!(err_with_code(
            can_run_command(
                &CommandExtendedData::none_map().get("").unwrap().clone(),
                &GuildCommandConfiguration {
                    id: "test".into(),
                    guild_id: "test".into(),
                    command: "test".into(),
                    perms: Some(PermissionChecks {
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
                        checks_needed: 2,
                    }),
                    disabled: None,
                },
                &gen_module_config("core"),
                "test",
                serenity::all::Permissions::BAN_MEMBERS,
                &["abc.test".into()],
            ),
            "missing_min_checks"
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
                &[],
            ),
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
            &[],
        )
        .is_ok());

        assert!(can_run_command(
            &CommandExtendedData::none_map().get("").unwrap().clone(),
            &GuildCommandConfiguration {
                id: "test".into(),
                guild_id: "test".into(),
                command: "test".into(),
                perms: Some(PermissionChecks {
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
                    checks_needed: 1,
                }),
                disabled: None,
            },
            &gen_module_config("core"),
            "test",
            serenity::all::Permissions::BAN_MEMBERS,
            &["abc.test".into()],
        )
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
                    default_perms: Some(PermissionChecks {
                        checks: vec![PermissionCheck {
                            kittycat_perms: vec![],
                            native_perms: vec![serenity::all::Permissions::VIEW_AUDIT_LOG],
                            outer_and: false,
                            inner_and: false,
                        }],
                        checks_needed: 1,
                    }),
                },
                "test abc",
                serenity::all::Permissions::VIEW_AUDIT_LOG,
                &[],
            );

            println!("{}", r.code());

            r
        }
        .is_ok());
    }
}

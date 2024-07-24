use std::sync::{Arc, RwLock};

use super::{
    CommandExtendedData, GuildCommandConfiguration, GuildModuleConfiguration, PermissionCheck,
    PermissionChecks,
};
use splashcore_rs::types::silverpelt::PermissionResult;

/// This function runs a single permission check on a command without taking any branching decisions
///
/// This may be useful when mocking or visualizing a permission check
fn check_perms_single(
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

struct InternalTemplateExecuteState {
    /// The current native permissions of the member
    member_native_perms: serenity::all::Permissions,
    /// The current kittycat permissions of the member
    member_kittycat_perms: Vec<kittycat::perms::Permission>,
    /// The current permission result
    result: RwLock<Option<PermissionResult>>,
}

// Run permission check function
struct RunPermissionCheckFunction {
    state: Arc<InternalTemplateExecuteState>,
}

// run_permission_check(kittycat_perms = string[], native_permissions = Permissions, and = BOOLEAN)
impl templating::engine::Function for RunPermissionCheckFunction {
    fn call(
        &self,
        args: &std::collections::HashMap<String, templating::engine::Value>,
    ) -> templating::engine::Result<templating::engine::Value> {
        let kittycat_perms = args
            .get("kittycat_perms")
            .ok_or("missing kittycat_perms")?
            .as_array()
            .ok_or("kittycat_perms is not an array")?
            .iter()
            .map(|v| {
                v.as_str()
                    .ok_or("kittycat_perm is not a string")
                    .map(|s| s.to_string())
            })
            .collect::<Result<Vec<String>, _>>()?;

        let native_perms = match args.get("native_perms").ok_or("missing native_perms")? {
            templating::engine::Value::String(s) => {
                vec![serenity::all::Permissions::from_bits_truncate(
                    s.parse::<u64>()
                        .map_err(|_| "native_perms is not a number")?,
                )]
            }
            templating::engine::Value::Number(n) => {
                vec![serenity::all::Permissions::from_bits_truncate(
                    n.as_u64().ok_or("native_perms is not a number")?,
                )]
            }
            templating::engine::Value::Array(a) => a
                .iter()
                .map(|v| match v {
                    templating::engine::Value::String(s) => {
                        Ok(serenity::all::Permissions::from_bits_truncate(
                            s.parse::<u64>()
                                .map_err(|_| "native_perms is not a number")?,
                        ))
                    }
                    templating::engine::Value::Number(n) => {
                        Ok(serenity::all::Permissions::from_bits_truncate(
                            n.as_u64().ok_or("native_perms is not a number")?,
                        ))
                    }
                    _ => Err("native_perms is not a number"),
                })
                .collect::<Result<Vec<serenity::all::Permissions>, _>>()?,
            _ => return Err("native_perms is not a number".into()),
        };

        let and = args
            .get("and")
            .ok_or("missing and")?
            .as_bool()
            .ok_or("and is not a boolean")?;

        let res = check_perms_single(
            &PermissionCheck {
                kittycat_perms,
                native_perms,
                inner_and: and,
                outer_and: false,
            },
            self.state.member_native_perms,
            &self.state.member_kittycat_perms,
        );

        let mut writer = self
            .state
            .result
            .write()
            .map_err(|_| "failed to lock result")?;

        let ret = serde_json::json!({
            "code": res.code(),
            "ok": res.is_ok(),
        });

        *writer = Some(res);

        Ok(ret)
    }
}

// Run permission check function
struct PermissionResultFilter {
    state: Arc<InternalTemplateExecuteState>,
}

// permission_result (result=PermissionResult)
impl templating::engine::Filter for PermissionResultFilter {
    fn filter(
        &self,
        value: &templating::engine::Value,
        _args: &std::collections::HashMap<String, templating::engine::Value>,
    ) -> templating::engine::Result<templating::engine::Value> {
        let value = match value {
            templating::engine::Value::String(s) => serde_json::from_str::<PermissionResult>(s),
            templating::engine::Value::Object(m) => serde_json::from_value::<PermissionResult>(
                templating::engine::Value::Object(m.clone()),
            ),
            _ => return Err("value is not a string".into()),
        }
        .map_err(|e| format!("failed to parse PermissionResult: {}", e))?;

        let mut writer = self
            .state
            .result
            .write()
            .map_err(|_| "failed to lock result")?;

        *writer = Some(value);

        Ok(templating::engine::Value::Null)
    }
}

async fn template_permission_checks(
    template: &str,
    member_native_perms: serenity::all::Permissions,
    member_kittycat_perms: Vec<kittycat::perms::Permission>,
    ctx: TemplatePermissionChecksContext,
) -> PermissionResult {
    let mut template = match templating::compile_template(
        template,
        templating::CompileTemplateOptions {
            cache_result: true,
            ignore_cache: false,
        },
    )
    .await
    {
        Ok(t) => t,
        Err(e) => {
            return PermissionResult::GenericError {
                error: format!("failed to compile template: {}", e),
            }
        }
    };

    let mut context = templating::engine::Context::new();

    if let Err(e) = context.insert("user_id", &ctx.user_id) {
        return PermissionResult::GenericError {
            error: format!("failed to insert user_id into context: {}", e),
        };
    }

    if let Err(e) = context.insert("guild_id", &ctx.guild_id) {
        return PermissionResult::GenericError {
            error: format!("failed to insert guild_id into context: {}", e),
        };
    }

    if let Err(e) = context.insert("guild_owner_id", &ctx.guild_owner_id) {
        return PermissionResult::GenericError {
            error: format!("failed to insert guild_owner_id into context: {}", e),
        };
    }

    if let Err(e) = context.insert("channel_id", &ctx.channel_id) {
        return PermissionResult::GenericError {
            error: format!("failed to insert channel_id into context: {}", e),
        };
    }

    if let Err(e) = context.insert(
        "native_permissions",
        &member_native_perms.bits().to_string(),
    ) {
        return PermissionResult::GenericError {
            error: format!("failed to insert native_permissions into context: {}", e),
        };
    }

    if let Err(e) = context.insert("kittycat_permissions", &member_kittycat_perms) {
        return PermissionResult::GenericError {
            error: format!("failed to insert kittycat_permissions into context: {}", e),
        };
    }

    let state = Arc::new(InternalTemplateExecuteState {
        member_native_perms,
        member_kittycat_perms,
        result: RwLock::new(None),
    });

    template.register_function(
        "run_permission_check",
        RunPermissionCheckFunction {
            state: state.clone(),
        },
    );

    template.register_filter(
        "permission_result",
        PermissionResultFilter {
            state: state.clone(),
        },
    );

    // Execute the template
    match templating::execute_template(&mut template, &context).await {
        Ok(r) => r,
        Err(e) => {
            return PermissionResult::GenericError {
                error: format!("failed to execute template: {}", e),
            }
        }
    };

    let mut writer = match state
        .result
        .write()
        .map_err(|e| format!("failed to lock result: {:?}", e))
    {
        Ok(r) => r,
        Err(e) => return PermissionResult::GenericError { error: e },
    };

    (*writer).take().unwrap_or(PermissionResult::Ok {})
}

#[derive(Default, Clone, Eq, PartialEq, Debug)]
pub struct TemplatePermissionChecksContext {
    pub user_id: serenity::all::UserId,
    pub guild_id: serenity::all::GuildId,
    pub guild_owner_id: serenity::all::UserId,
    pub channel_id: Option<serenity::all::ChannelId>,
}

#[allow(clippy::too_many_arguments)]
pub async fn can_run_command(
    cmd_data: &CommandExtendedData,
    command_config: &GuildCommandConfiguration,
    module_config: &GuildModuleConfiguration,
    cmd_qualified_name: &str,
    member_native_perms: serenity::all::Permissions,
    member_kittycat_perms: Vec<kittycat::perms::Permission>,
    is_default_enabled: bool,
    template_ctx: TemplatePermissionChecksContext,
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
        if module_config.disabled.unwrap_or(!is_default_enabled) {
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
                template_ctx,
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
            TemplatePermissionChecksContext::default()
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
                TemplatePermissionChecksContext::default()
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
                TemplatePermissionChecksContext::default()
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
                TemplatePermissionChecksContext::default()
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
                TemplatePermissionChecksContext::default()
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
            TemplatePermissionChecksContext::default()
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
            TemplatePermissionChecksContext::default()
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
                TemplatePermissionChecksContext::default(),
            )
            .await;

            println!("{}", r.code());

            r
        }
        .is_ok());
    }
}

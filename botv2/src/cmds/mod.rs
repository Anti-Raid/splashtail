pub mod core;
pub mod limits;
pub mod backups;

use once_cell::sync::Lazy;
use indexmap::{indexmap, IndexMap};

pub type Command = poise::Command<crate::Data, crate::Error>;
pub type CommandExtendedDataMap = IndexMap<&'static str, CommandExtendedData>;
pub type CommandAndPermissions = (Command, CommandExtendedDataMap);

/// List of enabled commands
/// 
/// Add to this list to enable a command
pub fn enabled_commands() -> IndexMap<&'static str, Vec<CommandAndPermissions>> {
    indexmap! {
        "core" => core::commands(),
        "limits" => limits::commands(),
        "backups" => backups::commands(),
    }
}

#[derive(Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PermissionCheck {
    /// The kittycat permissions needed to run the command
    pub kittycat_perms: Vec<String>,
    /// The native permissions needed to run the command
    pub native_perms: Vec<serenity::all::Permissions>,
    /// Whether the next permission check should be ANDed (all needed) or OR'd (at least one) to the current
    pub outer_and: bool,
    /// Whether or not the perms are ANDed (all needed) or OR'd (at least one)
    pub inner_and: bool,
}

#[derive(Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PermissionChecks {
    /// The list of permission checks
    pub checks: Vec<PermissionCheck>,

    /// Number of checks that need to be true
    pub checks_needed: usize,
}

#[derive(Clone, PartialEq)]
pub struct CommandExtendedData {
    /// The default permissions needed to run this command
    pub default_perms: PermissionChecks,
}

impl Default for CommandExtendedData {
    fn default() -> Self {
        Self {
            default_perms: PermissionChecks {
                checks: vec![],
                checks_needed: 0,
            },
        }
    }
}

/// Command extra data (permissions)
pub static COMMAND_EXTRA_DATA: Lazy<indexmap::IndexMap<String, CommandExtendedDataMap>> = Lazy::new(|| {
    let mut map = indexmap::IndexMap::new();
    
    for (_, commands) in enabled_commands() {
        for (command, extended_data) in commands {
            map.insert(command.name.clone(), extended_data);
        }
    }

    map
});

/// Guild command configuration data
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize, Default)]
pub struct GuildCommandConfiguration {
    /// Thd ID
    pub id: String,
    /// The guild id (from db)
    pub guild_id: String,
    /// The command name
    pub command: String,
    /// The permission method (kittycat)
    pub perms: Option<PermissionChecks>,
    /// Whether or not the command is disabled
    pub disabled: bool,
}

/// From name_split, construct a list of all permutations of the command name to check
///
/// E.g: If subcommand is `limits hit`, then `limits` and `limits hit` will be constructed
///     as the list of commands to check
/// E.g 2: If subcommand is `limits hit add`, then `limits`, `limits hit` and `limits hit add`
///     will be constructed as the list of commands to check
pub fn permute_command_names(name: &str) -> Vec<String> {
    // Check if subcommand by splitting the command name
    let name_split = name.split(' ').collect::<Vec<&str>>();

    let mut commands_to_check = Vec::new();

    for i in 0..name_split.len() {
        let mut command = String::new();

        for (j, cmd) in name_split.iter().enumerate().take(i + 1) {
            command += cmd;

            if j != i {
                command += " ";
            }
        }

        commands_to_check.push(command);
    }        

    commands_to_check
}

/// Returns the configuration of a command
pub async fn get_command_configuration(pool: &sqlx::PgPool, guild_id: &str, name: &str) -> Result<(CommandExtendedData, Option<GuildCommandConfiguration>), crate::Error> {
    let permutations = permute_command_names(name);
    let root_cmd = permutations.first().unwrap();

    let root_cmd_data = COMMAND_EXTRA_DATA.get(root_cmd);

    let Some(root_cmd_data) = root_cmd_data else {
        return Err(
            format!("The command ``{}`` does not exist [no root configuration found?]", name).into()
        );
    };

    let mut cmd_data = root_cmd_data.get("").unwrap_or(&CommandExtendedData::default()).clone();
    for command in permutations.iter() {
        let cmd_replaced = command.replace(&root_cmd.to_string(), "");
        if let Some(data) = root_cmd_data.get(&cmd_replaced.as_str()) {
            cmd_data = data.clone();
        }
    }

    let mut command_configuration = None;

    for permutation in permutations.iter() {
        let rec = sqlx::query!(
            "SELECT id, guild_id, command, perms, disabled FROM guild_command_configurations WHERE guild_id = $1 AND command = $2",
            guild_id,
            permutation,
        )
        .fetch_optional(pool)
        .await?;

        // We are deeper in the tree, so we can overwrite the command configuration
        if let Some(rec) = rec {
            command_configuration = Some(GuildCommandConfiguration {
                id: rec.id.hyphenated().to_string(),
                guild_id: rec.guild_id,
                command: rec.command,
                perms: {
                    if let Some(perms) = rec.perms {
                        serde_json::from_value(perms).unwrap()
                    } else {
                        None
                    }
                },
                disabled: rec.disabled,
            });
        }
    }

    Ok((cmd_data, command_configuration))
}

/// This function runs a single permission check on a command without taking any branching decisions
/// 
/// This may be useful when mocking or visualizing a permission check
pub fn check_perms_single(cmd_qualified_name: &str, cmd_real_name: &str, check: &PermissionCheck, member_native_perms: serenity::all::Permissions, member_kittycat_perms: &[String]) -> Result<(), (String, crate::Error)> {    
    if check.kittycat_perms.is_empty() && check.native_perms.is_empty() {
        return Ok(()); // Short-circuit if we don't have any permissions to check
    }
    
    // Check if we have ADMINISTRATOR
    let is_discord_admin = member_native_perms.contains(serenity::all::Permissions::ADMINISTRATOR);

    // Kittycat
    if check.inner_and {
        // inner AND, short-circuit if we don't have the permission
        for perm in &check.kittycat_perms {
            if !kittycat::perms::has_perm(member_kittycat_perms, perm) {
                return Err(
                    (
                        "missing_kittycat_perms".into(),
                        format!("You do not have the required permissions to run this command (``{}``) implied from ``{}``. You need ``{}`` permissions to execute this command.", cmd_qualified_name, cmd_real_name, perm).into()
                    )
                );
            }
        }

        if !is_discord_admin {
            for perm in &check.native_perms {
                if !member_native_perms.contains(*perm) {
                    return Err(
                        (
                            "missing_native_perms".into(),
                            format!("You do not have the required permissions to run this command (``{}``) implied from ``{}``. You need ``{}`` permissions to execute this command.", cmd_qualified_name, cmd_real_name, perm).into()
                        )
                    );
                }
            }
        }
    } else {
        // inner OR, short-circuit if we have the permission
        let has_any_np = check.native_perms.iter().any(|perm| is_discord_admin || member_native_perms.contains(*perm));
        
        if !has_any_np {
            let has_any_kc = check.kittycat_perms.iter().any(|perm| kittycat::perms::has_perm(member_kittycat_perms, perm));

            if !has_any_kc {
                let np = check.native_perms.iter().map(|p| format!("{}", p)).collect::<Vec<String>>().join(" | ");

                let perms = format!("*Discord*: ``{}`` OR *Custom Permissions*: ``{}``", check.kittycat_perms.join(" | "), np);

                return Err(
                    (
                        "missing_any_perms".into(),
                        format!("You do not have the required permissions to run this command (``{}``) implied from ``{}``. You need at least one of the following permissions to execute this command:\n\n{}", cmd_qualified_name, cmd_real_name, perms).into()
                    )
                );
            }
        }
    }    

    Ok(())
}

pub fn can_run_command(
    cmd_data: &CommandExtendedData, 
    command_config: &GuildCommandConfiguration, 
    cmd_qualified_name: &str, 
    member_native_perms: serenity::all::Permissions,
    member_kittycat_perms: &[String],
) -> Result<(), (String, crate::Error)> {
    if command_config.disabled {
        return Err(
            (
                "command_disabled".into(),
                format!("The command ``{}`` (inherited from ``{}``) is disabled on this server", cmd_qualified_name, command_config.command).into()
            )
        );
    }

    let perms = command_config.perms.as_ref().unwrap_or(&cmd_data.default_perms);

    if perms.checks.is_empty() {
        return Ok(());
    }

    // This stores whether or not we need to check the next permission AND the current one or OR the current one 
    let mut outer_and = false;
    let mut success: usize = 0;
    
    for check in &perms.checks {
        // Run the check
        let res = check_perms_single(cmd_qualified_name, &command_config.command, check, member_native_perms, member_kittycat_perms);

        if outer_and {
            #[allow(clippy::question_mark)] // Question mark needs cloning which may harm performance
            if res.is_err() {
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

    // Check the OR now
    if perms.checks_needed == 0 {
        if success == 0 {
            let mut np = Vec::new();
            let mut kc = Vec::new();

            for check in &perms.checks {
                np.extend(check.native_perms.iter().map(|p| p.to_string()));
                kc.extend(check.kittycat_perms.iter().map(|p| p.to_string()));
            }

            let perms = format!("*Discord*: ``{}`` OR *Custom Permissions*: ``{}``", np.join(" | "), kc.join(" | "));
            
            return Err(
                (
                    "missing_any_perms".into(),
                    format!("You do not have the required permissions to run this command (``{}``) implied from ``{}``. You need at least one of the following permissions to execute this command:\n``{}``", cmd_qualified_name, command_config.command, perms).into()
                )
            );
        } else {
            return Ok(());
        }
    } else if success < perms.checks_needed {
        let mut np = Vec::new();
        let mut kc = Vec::new();

        for check in &perms.checks {
            np.extend(check.native_perms.iter().map(|p| p.to_string()));
            kc.extend(check.kittycat_perms.iter().map(|p| p.to_string()));
        }

        let ps = format!("*Discord*: ``{}`` OR *Custom Permissions*: ``{}``", np.join(" | "), kc.join(" | "));

        // TODO: Improve this and group the permissions in error
        return Err(
            (
                "missing_min_checks".into(),
                format!("You do not have the required permissions to run this command (``{}``) implied from ``{}``. You need at least {} of the following permissions to execute this command:\n\n{}", cmd_qualified_name, command_config.command, perms.checks_needed, ps).into()
            )
        );
    }

    Ok(())
}

impl CommandExtendedData {
    pub fn none() -> CommandExtendedDataMap {
        indexmap! {
            "" => CommandExtendedData {
                default_perms: PermissionChecks {
                    checks: vec![],
                    checks_needed: 0,
                },
            },
        }
    }

    pub fn kittycat_simple(namespace: &str, permission: &str) -> CommandExtendedData {
        CommandExtendedData {
            default_perms: PermissionChecks {
                checks: vec![PermissionCheck {
                    kittycat_perms: vec![format!("{}.{}", namespace, permission)],
                    native_perms: vec![],
                    outer_and: false,
                    inner_and: false,
                }],
                checks_needed: 1,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn err_with_code(e: Result<(), (String, crate::Error)>, code: &str) -> bool {
        if let Err((e_code, _)) = e {
            println!("test_check_perms_single: {} == {}", e_code, code);
            e_code == code
        } else {
            false
        }
    }

    #[test]
    fn test_names_to_check() {
        println!("{:?}", permute_command_names("limits hit view"));
        assert_eq!(permute_command_names("limits"), vec!["limits"]);
        assert_eq!(permute_command_names("limits hit"), vec!["limits", "limits hit"]);
        assert_eq!(permute_command_names("limits hit add"), vec!["limits", "limits hit", "limits hit add"]);
    }

    #[test]
    fn test_check_perms_single() {
        // Basic tests
        assert!(
            err_with_code(
                check_perms_single(
                "test",
                "test",
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
            )
        );

        assert!(
            check_perms_single(
                "test",
                "test",
                &PermissionCheck {
                    kittycat_perms: vec![],
                    native_perms: vec![],
                    outer_and: false,
                    inner_and: false,
                },
                serenity::all::Permissions::empty(),
                &["abc.test".into()],
            ).is_ok()
        );

        // With inner and
        assert!(
            err_with_code(
                check_perms_single(
                "test",
                "test",
                &PermissionCheck {
                    kittycat_perms: vec![],
                    native_perms: vec![serenity::all::Permissions::ADMINISTRATOR, serenity::all::Permissions::BAN_MEMBERS],
                    outer_and: false,
                    inner_and: true,
                },
                serenity::all::Permissions::BAN_MEMBERS,
                &["abc.test".into()],
            ),
            "missing_native_perms"
            )
        );

        // Admin overrides other native perms
        assert!(
            check_perms_single(
                "test",
                "test",
                &PermissionCheck {
                    kittycat_perms: vec![],
                    native_perms: vec![serenity::all::Permissions::BAN_MEMBERS],
                    outer_and: false,
                    inner_and: false,
                },
                serenity::all::Permissions::ADMINISTRATOR,
                &["abc.test".into()],
            ).is_ok()
        );

        // Kittycat
        assert!(
            err_with_code(
                check_perms_single(
                    "test",
                    "test",
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
            )
        );
    }

    #[test]
    fn test_can_run_command() {
        // Basic test
        assert!(can_run_command(
            &CommandExtendedData::none().get("").unwrap().clone(),
            &GuildCommandConfiguration {
                id: "test".into(),
                guild_id: "test".into(),
                command: "test".into(),
                perms: None,
                disabled: false,
            },
            "test",
            serenity::all::Permissions::empty(),
            &["abc.test".into()],
        ).is_ok());

        // With a native permission
        assert!(
            err_with_code(
                can_run_command(
                    &CommandExtendedData::none().get("").unwrap().clone(),
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
                        disabled: false,
                    },
                    "test",
                    serenity::all::Permissions::empty(),
                    &["abc.test".into()],
                ),
                "missing_any_perms"
            )
        );

        assert!(
            err_with_code(
                can_run_command(
                    &CommandExtendedData::none().get("").unwrap().clone(),
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
                        disabled: false,
                    },
                    "test",
                    serenity::all::Permissions::empty(),
                    &["abc.test".into()],
                ),
                "missing_min_checks"
            )
        );

        assert!(
            err_with_code(
                can_run_command(
                    &CommandExtendedData::none().get("").unwrap().clone(),
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
                        disabled: false,
                    },
                    "test",
                    serenity::all::Permissions::BAN_MEMBERS,
                    &["abc.test".into()],
                ),
                "missing_min_checks"
            )
        );

        // Real-life example
        assert!(
            err_with_code(
                can_run_command(
                    &CommandExtendedData::kittycat_simple("backups", "create"),
                    &GuildCommandConfiguration {
                        id: "test".into(),
                        guild_id: "test".into(),
                        command: "test".into(),
                        perms: None,
                        disabled: false,
                    },
                    "backups create",
                    serenity::all::Permissions::ADMINISTRATOR,
                    &[],
                ),
                "missing_min_checks"
            )
        );

        assert!(
            can_run_command(
                &CommandExtendedData::none().get("").unwrap().clone(),
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
                    disabled: false,
                },
                "test",
                serenity::all::Permissions::BAN_MEMBERS,
                &["abc.test".into()],
            ).is_ok()
        );
    }
}
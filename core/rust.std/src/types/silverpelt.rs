use indexmap::{indexmap, IndexMap};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

pub type CommandExtendedDataMap = IndexMap<&'static str, CommandExtendedData>;

#[derive(Default, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Debug)]
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

impl Display for PermissionCheck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.native_perms.is_empty() {
            write!(f, "\nDiscord: ")?;

            for (j, perm) in self.native_perms.iter().enumerate() {
                if j != 0 {
                    write!(f, " ")?;
                }

                write!(f, "{}", perm)?;

                if j < self.native_perms.len() - 1 {
                    if self.inner_and {
                        write!(f, " AND")?;
                    } else {
                        write!(f, " OR")?;
                    }
                }
            }
        }

        if !self.kittycat_perms.is_empty() {
            write!(f, "\nCustom Permissions (kittycat): ")?;

            for (j, perm) in self.kittycat_perms.iter().enumerate() {
                if j != 0 {
                    write!(f, " ")?;
                }

                write!(f, "{}", perm)?;

                if j < self.kittycat_perms.len() - 1 {
                    if self.inner_and {
                        write!(f, " AND")?;
                    } else {
                        write!(f, " OR")?;
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Default, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct PermissionChecks {
    /// The list of permission checks
    pub checks: Vec<PermissionCheck>,

    /// Number of checks that need to be true
    pub checks_needed: usize,
}

impl Display for PermissionChecks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, check) in self.checks.iter().enumerate() {
            if i != 0 {
                write!(f, " ")?;
            }

            write!(f, "\n{}. {}", i, check)?; // The Display trait on PermissionCheck automatically formats individual permissions the correct way

            let empty = check.kittycat_perms.is_empty() && check.native_perms.is_empty();

            if i < self.checks.len() - 1 {
                if check.outer_and && !empty {
                    write!(f, " AND ")?;
                } else {
                    write!(f, " OR ")?;
                }
            }
        }

        write!(f, "\nChecks needed: {}", self.checks_needed)?;

        Ok(())
    }
}

// @ci go=CommandExtendedData
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize, Debug)]
pub struct CommandExtendedData {
    /// The default permissions needed to run this command
    pub default_perms: PermissionChecks,
    /// Whether the command is enabled by default or not
    pub is_default_enabled: bool,
    /// Whether the command should be hidden on the website or not
    pub web_hidden: bool,
    /// Whether the command is a virtual command or not (virtual commands are not loaded into the bot, but can be used for permission checks etc)
    pub virtual_command: bool,
}

impl Default for CommandExtendedData {
    fn default() -> Self {
        Self {
            default_perms: PermissionChecks {
                checks: vec![],
                checks_needed: 0,
            },
            is_default_enabled: true,
            web_hidden: false,
            virtual_command: false,
        }
    }
}

impl CommandExtendedData {
    pub fn none() -> Self {
        CommandExtendedData {
            default_perms: PermissionChecks {
                checks: vec![],
                checks_needed: 0,
            },
            is_default_enabled: true,
            web_hidden: false,
            virtual_command: false,
        }
    }

    pub fn none_map() -> CommandExtendedDataMap {
        indexmap! {
            "" => CommandExtendedData {
                default_perms: PermissionChecks {
                    checks: vec![],
                    checks_needed: 0,
                },
                is_default_enabled: true,
                web_hidden: false,
                virtual_command: false,
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
            is_default_enabled: true,
            web_hidden: false,
            virtual_command: false,
        }
    }

    pub fn kittycat_or_admin(namespace: &str, permission: &str) -> CommandExtendedData {
        CommandExtendedData {
            default_perms: PermissionChecks {
                checks: vec![PermissionCheck {
                    kittycat_perms: vec![format!("{}.{}", namespace, permission)],
                    native_perms: vec![serenity::all::Permissions::ADMINISTRATOR],
                    outer_and: false,
                    inner_and: false,
                }],
                checks_needed: 1,
            },
            is_default_enabled: true,
            web_hidden: false,
            virtual_command: false,
        }
    }
}

/// Guild command configuration data
#[derive(Clone, Hash, Eq, PartialEq, serde::Serialize, serde::Deserialize, Debug)]
pub struct GuildCommandConfiguration {
    /// The ID
    pub id: String,
    /// The guild id (from db)
    pub guild_id: String,
    /// The command name
    pub command: String,
    /// The permission checks on the command, if unset, will revert to either the modules default_perms and if that is unset, the default perms set on the command itself
    pub perms: Option<PermissionChecks>,
    /// Whether or not the command is disabled. None means to use the default command configuration
    pub disabled: Option<bool>,
}

/// Guild module configuration data
#[derive(Clone, Hash, Eq, PartialEq, serde::Serialize, serde::Deserialize, Debug)]
pub struct GuildModuleConfiguration {
    /// The ID
    pub id: String,
    /// The guild id (from db)
    pub guild_id: String,
    /// The module id
    pub module: String,
    /// Whether ot not the module is disabled or not. None means to use the default module configuration
    pub disabled: Option<bool>,
    /// The default permission checks of the module, can be overrided by the command configuration
    pub default_perms: Option<PermissionChecks>,
}

// @ci go=PermissionResult
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "var")]
pub enum PermissionResult {
    Ok {},
    OkWithMessage {
        message: String,
    },
    MissingKittycatPerms {
        check: PermissionCheck,
    },
    MissingNativePerms {
        check: PermissionCheck,
    },
    MissingAnyPerms {
        check: PermissionCheck,
    },
    CommandDisabled {
        command_config: GuildCommandConfiguration,
    },
    UnknownModule {
        module_config: GuildModuleConfiguration,
    },
    ModuleNotFound {},
    ModuleDisabled {
        module_config: GuildModuleConfiguration,
    },
    NoChecksSucceeded {
        checks: PermissionChecks,
    },
    MissingMinChecks {
        checks: PermissionChecks,
    },
    DiscordError {
        error: String,
    },
    SudoNotGranted {},
    GenericError {
        error: String,
    },
}

impl<T: core::fmt::Display> From<T> for PermissionResult {
    fn from(e: T) -> Self {
        PermissionResult::GenericError {
            error: e.to_string(),
        }
    }
}

impl PermissionResult {
    pub fn code(&self) -> &'static str {
        match self {
            PermissionResult::Ok { .. } => "ok",
            PermissionResult::OkWithMessage { .. } => "ok_with_message",
            PermissionResult::MissingKittycatPerms { .. } => "missing_kittycat_perms",
            PermissionResult::MissingNativePerms { .. } => "missing_native_perms",
            PermissionResult::MissingAnyPerms { .. } => "missing_any_perms",
            PermissionResult::CommandDisabled { .. } => "command_disabled",
            PermissionResult::UnknownModule { .. } => "unknown_module",
            PermissionResult::ModuleNotFound { .. } => "module_not_found",
            PermissionResult::ModuleDisabled { .. } => "module_disabled",
            PermissionResult::NoChecksSucceeded { .. } => "no_checks_succeeded",
            PermissionResult::MissingMinChecks { .. } => "missing_min_checks",
            PermissionResult::DiscordError { .. } => "discord_error",
            PermissionResult::SudoNotGranted { .. } => "sudo_not_granted",
            PermissionResult::GenericError { .. } => "generic_error",
        }
    }

    pub fn is_ok(&self) -> bool {
        matches!(
            self,
            PermissionResult::Ok { .. } | PermissionResult::OkWithMessage { .. }
        )
    }

    pub fn to_markdown(&self) -> String {
        match self {
            PermissionResult::Ok { .. } => "No message/context available".to_string(),
            PermissionResult::OkWithMessage { message } => message.clone(),
            PermissionResult::MissingKittycatPerms { check } => {
                format!(
                    "You do not have the required permissions to perform this action. Try checking that you have the below permissions: {}",
                    check
                )
            }
            PermissionResult::MissingNativePerms { check } => {
                format!(
                    "You do not have the required permissions to perform this action. Try checking that you have the below permissions: {}",
                    check
                )
            }
            PermissionResult::MissingAnyPerms { check } => {
                format!(
                    "You do not have the required permissions to perform this action. Try checking that you have the below permissions: {}",
                    check
                )
            }
            PermissionResult::CommandDisabled { command_config } => {
                format!(
                    "You cannot perform this action because the command ``{}`` is disabled on this server",
                    command_config.command
                )
            }
            PermissionResult::UnknownModule { module_config } => {
                format!("The module ``{}`` does not exist", module_config.module)
            }
            PermissionResult::ModuleNotFound {} => {
                "The module corresponding to this command could not be determined".to_string()
            }
            PermissionResult::ModuleDisabled { module_config } => {
                format!(
                    "The module ``{}`` is disabled on this server",
                    module_config.module
                )
            }
            PermissionResult::NoChecksSucceeded { checks } => {
                format!(
                    "You do not have the required permissions to perform this action. You need at least one of the following permissions to perform this action:\n\n**Required Permissions**: {}",
                    checks
                )
            }
            PermissionResult::MissingMinChecks { checks } => {
                format!(
                    "You do not have the required permissions to perform this action. You need at least {} of the following permissions to perform this action:\n\n**Required Permissions**: {}",
                    checks.checks_needed, checks
                )
            }
            PermissionResult::DiscordError { error } => {
                format!("A Discord-related error seems to have occurred: {}.\n\nPlease try again later, it might work!", error)
            }
            PermissionResult::SudoNotGranted {} => {
                "This module is only available for root (staff) and/or developers of the bot"
                    .to_string()
            }
            PermissionResult::GenericError { error } => error.clone(),
        }
    }
}

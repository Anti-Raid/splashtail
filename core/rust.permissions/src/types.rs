use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::hash::Hash;

#[derive(Default, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct PermissionCheck {
    /// The kittycat permissions needed to run the command
    pub kittycat_perms: Vec<String>,
    /// The native permissions needed to run the command
    pub native_perms: Vec<serenity::all::Permissions>,
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

// @ci go=PermissionResult
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "var")]
pub enum PermissionResult {
    Ok {},
    OkWithMessage { message: String },
    MissingKittycatPerms { check: PermissionCheck },
    MissingNativePerms { check: PermissionCheck },
    MissingAnyPerms { check: PermissionCheck },
    CommandDisabled { command: String },
    UnknownModule { module: String },
    ModuleNotFound {},
    ModuleDisabled { module: String },
    DiscordError { error: String },
    SudoNotGranted {},
    GenericError { error: String },
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
            PermissionResult::CommandDisabled { command } => {
                format!(
                    "You cannot perform this action because the command ``{}`` is disabled on this server",
                    command
                )
            }
            PermissionResult::UnknownModule { module } => {
                format!("The module ``{}`` does not exist", module)
            }
            PermissionResult::ModuleNotFound {} => {
                "The module corresponding to this command could not be determined".to_string()
            }
            PermissionResult::ModuleDisabled { module } => {
                format!("The module ``{}`` is disabled on this server", module)
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

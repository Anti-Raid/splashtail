use permissions::types::PermissionResult;
use serde::{Deserialize, Serialize};
use serenity::all::{GuildChannel, Permissions, Role, RoleId};
use silverpelt::types::{GuildCommandConfiguration, GuildModuleConfiguration};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GuildChannelWithPermissions {
    pub user: Permissions,
    pub bot: Permissions,
    pub channel: GuildChannel,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BaseGuildUserInfo {
    pub owner_id: String,
    pub name: String,
    pub icon: Option<String>,
    /// List of all roles in the server
    pub roles: Vec<Role>,
    /// List of roles the user has
    pub user_roles: Vec<RoleId>,
    /// List of roles the bot has
    pub bot_roles: Vec<RoleId>,
    /// List of all channels in the server
    pub channels: Vec<GuildChannelWithPermissions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckCommandPermission {
    pub perm_res: PermissionResult,
    pub is_ok: bool,
}

#[derive(Debug, Serialize, Deserialize)]
/// Given a guild id, a user id and a command name, check if the user has permission to run the command
pub struct CheckCommandPermissionRequest {
    pub command: String,
    pub opts: RpcCheckCommandOptions,
}

/// Extra options for checking a command
///
/// This is seperate from the actual internal stuff to both avoid exposing
/// internals and to optimize data flow
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct RpcCheckCommandOptions {
    /// Flags of type RpcCheckCommandOptionsFlags
    #[serde(default)]
    pub flags: u8,

    /// What custom resolved permissions to use for the user.
    ///
    /// API needs this for limiting the permissions of a user, allows setting custom resolved perms
    #[serde(default)]
    pub custom_resolved_kittycat_perms: Option<Vec<String>>,

    /// Custom permission checks to use
    #[serde(default)]
    pub custom_command_configuration: Option<Box<GuildCommandConfiguration>>,

    /// Custom permission checks to use
    #[serde(default)]
    pub custom_module_configuration: Option<Box<GuildModuleConfiguration>>,

    /// The current channel id
    #[serde(default)]
    pub channel_id: Option<serenity::all::ChannelId>,
}

bitflags::bitflags! {
    pub struct RpcCheckCommandOptionsFlags: u8 {
        /// Whether or not to ignore the fact that the module is disabled in the guild
        const IGNORE_MODULE_DISABLED = 1 << 2;
        /// Whether or not to ignore the fact that the command is disabled in the guild
        const IGNORE_COMMAND_DISABLED = 1 << 3;
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClearModulesEnabledCacheRequest {
    pub guild_id: Option<serenity::all::GuildId>,
    pub module: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearModulesEnabledCacheResponse {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CanonicalSettingsResult {
    Ok {
        fields: Vec<indexmap::IndexMap<String, splashcore_rs::value::Value>>,
    },
    Err {
        error: ar_settings::types::SettingsError,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsOperationRequest {
    pub fields: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    pub op: ar_settings::types::OperationType,
    pub setting: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteTemplateRequest {
    pub args: serde_json::Value,
    pub template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecuteTemplateResponse {
    Ok { result: Option<serde_json::Value> },
    ExecErr { error: String },
    PermissionError { res: PermissionResult },
}

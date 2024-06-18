pub mod canonical_module;
pub mod cmd;
pub mod ext_generate;
pub mod member_permission_calc;
pub mod module_config;
pub mod permissions;
pub mod proxysupport;
pub mod settings;
pub mod silverpelt_cache;
pub mod utils;
pub mod value;

use futures::future::BoxFuture;
use indexmap::IndexMap;
use std::fmt::Display;
use std::sync::Arc;

pub type Command = poise::Command<crate::Data, crate::Error>;
pub type CommandExtendedDataMap = IndexMap<&'static str, CommandExtendedData>;

pub struct EventHandlerContext {
    pub guild_id: serenity::all::GuildId,
    pub full_event: serenity::all::FullEvent,
    pub data: Arc<crate::Data>,
    pub serenity_context: serenity::all::Context,
}

pub type ModuleEventHandler = Box<
    dyn Send
        + Sync
        + for<'a> Fn(&'a EventHandlerContext) -> BoxFuture<'a, Result<(), crate::Error>>,
>;

pub type OnStartupFunction =
    Box<dyn Send + Sync + for<'a> Fn(&'a crate::Data) -> BoxFuture<'a, Result<(), crate::Error>>>;

pub type OnReadyFunction = Box<
    dyn Send
        + Sync
        + for<'a> Fn(
            serenity::all::Context,
            &'a crate::Data,
        ) -> BoxFuture<'a, Result<(), crate::Error>>,
>;

/// This structure defines a basic module
#[derive(Default)]
pub struct Module {
    /// The ID of the module
    pub id: &'static str,

    /// The name of the module
    pub name: &'static str,

    /// The description of the module
    pub description: &'static str,

    /// Whether or not the module should be visible on the websites command lists
    pub web_hidden: bool,

    /// Whether or the module can be enabled and/or disabled
    pub toggleable: bool,

    /// Whether or not individual commands in the module can be configured
    pub commands_configurable: bool,

    /// Virtual module. These modules allow controlling functionality of the bot without having its commands loaded into the bot
    pub virtual_module: bool,

    /// Whether the module is enabled or disabled by default
    pub is_default_enabled: bool,

    /// The commands in the module
    pub commands: Vec<(Command, CommandExtendedDataMap)>,

    /// Event handlers (if any)
    pub event_handlers: Vec<ModuleEventHandler>,

    /// Background tasks (if any)
    pub background_tasks: Vec<botox::taskman::Task>,

    /// Function to be run on startup
    ///
    /// To run code involving serenity context, consider ``on_ready`` instead
    pub on_startup: Vec<OnStartupFunction>,

    /// Function to be run on ready
    ///
    /// This function will only be called once, when the shard is first ready
    pub on_first_ready: Vec<OnReadyFunction>,

    /// Modules may store files on seaweed, in order to allow for usage tracking,
    /// s3_paths should be set to the paths of the files on seaweed
    pub s3_paths: Vec<String>,

    /// Config options for this module
    pub config_options: Vec<settings::config_opts::ConfigOption>,
}

#[derive(Default, Clone, Hash, Eq, PartialEq, serde::Serialize, serde::Deserialize, Debug)]
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

#[derive(Default, Clone, Hash, Eq, PartialEq, serde::Serialize, serde::Deserialize, Debug)]
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
    /// Whether the command is enabeld by default or not
    pub is_default_enabled: bool,
}

impl Default for CommandExtendedData {
    fn default() -> Self {
        Self {
            default_perms: PermissionChecks {
                checks: vec![],
                checks_needed: 0,
            },
            is_default_enabled: true,
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
    /// The permission method (kittycat)
    pub perms: Option<PermissionChecks>,
    /// Whether or not the command is disabled. None means to use the default command configuration
    pub disabled: Option<bool>,
}

/// Guild module configuration data
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GuildModuleConfiguration {
    /// The ID
    pub id: String,
    /// The guild id (from db)
    pub guild_id: String,
    /// The module id
    pub module: String,
    /// Whether ot not the module is disabled or not. None means to use the default module configuration
    pub disabled: Option<bool>,
}

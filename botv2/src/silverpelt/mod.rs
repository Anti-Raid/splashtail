pub mod canonical_repr;
pub mod permissions;

use futures::future::BoxFuture;
use indexmap::IndexMap;
use moka::future::Cache;
use once_cell::sync::Lazy;
use serenity::all::{GuildId, UserId};

/// The silverpelt cache is a structure that contains the core state for the bot
pub struct SilverpeltCache {
    /// Cache of whether a (GuildId, UserId) pair has the permission to run a command
    pub command_permission_cache: Cache<(GuildId, UserId), IndexMap<String, CachedPermResult>>,

    /// Cache of the extended data given a command (the extended data map stores the default base permissions and other data per command)
    pub command_extra_data_map: dashmap::DashMap<String, CommandExtendedDataMap>,

    /// A commonly needed operation is mapping a module id to its respective module
    ///
    /// Module_id_cache is a cache of module id to module
    pub module_id_cache: dashmap::DashMap<String, Module>,

    /// Command ID to module map
    ///
    /// This uses an indexmap for now to avoid sending values over await point
    pub command_id_module_map: indexmap::IndexMap<String, String>,

    /// Cache of the canonical forms of all modules
    pub canonical_module_cache: dashmap::DashMap<String, canonical_repr::modules::CanonicalModule>,

    /// Cache of all event listeners for a given module
    pub module_event_listeners_cache: indexmap::IndexMap<String, Vec<ModuleEventHandler>>,
}

impl SilverpeltCache {
    pub fn new() -> Self {
        Self {
            command_permission_cache: Cache::builder()
                .time_to_live(std::time::Duration::from_secs(60))
                .build(),
            command_extra_data_map: {
                let map = dashmap::DashMap::new();

                for module in crate::modules::modules() {
                    for (command, extended_data) in module.commands {
                        map.insert(command.name.clone(), extended_data);
                    }
                }

                map
            },
            module_id_cache: {
                let map = dashmap::DashMap::new();

                for module in crate::modules::modules() {
                    map.insert(module.id.to_string(), module);
                }

                map
            },
            command_id_module_map: {
                let mut map = indexmap::IndexMap::new();

                for module in crate::modules::modules() {
                    for command in module.commands.iter() {
                        map.insert(command.0.name.to_string(), module.id.to_string());

                        for sub in command.0.subcommands.iter() {
                            map.insert(sub.name.to_string(), module.id.to_string());
                        }
                    }
                }

                map
            },
            canonical_module_cache: {
                let map = dashmap::DashMap::new();

                for module in crate::modules::modules() {
                    map.insert(
                        module.id.to_string(),
                        canonical_repr::modules::CanonicalModule::from(module),
                    );
                }

                map
            },
            module_event_listeners_cache: {
                let mut map = indexmap::IndexMap::new();

                for module in crate::modules::modules() {
                    map.insert(module.id.to_string(), module.event_handlers);
                }

                map
            },
        }
    }
}

pub static SILVERPELT_CACHE: Lazy<SilverpeltCache> = Lazy::new(SilverpeltCache::new);

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum CachedPermResult {
    Ok,
    Err(String),
}

pub type Command = poise::Command<crate::Data, crate::Error>;
pub type CommandExtendedDataMap = IndexMap<&'static str, CommandExtendedData>;
pub type CommandAndPermissions = (Command, CommandExtendedDataMap);

pub type ModuleEventHandler = Box<
    dyn Send
        + Sync
        + for<'a> Fn(
            &'a serenity::all::Context,
            &'a serenity::all::FullEvent,
        ) -> BoxFuture<'a, Result<(), crate::Error>>,
>;

/// This structure defines a basic module
pub struct Module {
    /// The ID of the module
    pub id: &'static str,

    /// The name of the module
    pub name: &'static str,

    /// The description of the module
    pub description: &'static str,

    /// Whether or not the module should be visible on the websites command lists
    pub web_hidden: bool,

    /// Whether or the module is configurable
    pub configurable: bool,

    /// Whether or not individual commands in the module can be configured
    pub commands_configurable: bool,

    /// Whether the module is enabled or disabled by default
    pub is_default_enabled: bool,

    /// The commands in the module
    pub commands: Vec<CommandAndPermissions>,

    /// Event handlers (if any)
    pub event_handlers: Vec<ModuleEventHandler>,
}

#[derive(Default, Clone, PartialEq, serde::Serialize, serde::Deserialize, Debug)]
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

#[derive(Default, Clone, PartialEq, serde::Serialize, serde::Deserialize, Debug)]
pub struct PermissionChecks {
    /// The list of permission checks
    pub checks: Vec<PermissionCheck>,

    /// Number of checks that need to be true
    pub checks_needed: usize,
}

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize, Debug)]
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

/// Guild command configuration data
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize, Debug)]
pub struct GuildCommandConfiguration {
    /// The ID
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
pub async fn get_command_configuration(
    pool: &sqlx::PgPool,
    guild_id: &str,
    name: &str,
) -> Result<
    (
        CommandExtendedData,
        Option<GuildCommandConfiguration>,
        Option<GuildModuleConfiguration>,
    ),
    crate::Error,
> {
    let permutations = permute_command_names(name);
    let root_cmd = permutations.first().unwrap();

    let root_cmd_data = SILVERPELT_CACHE.command_extra_data_map.get(root_cmd);

    let Some(root_cmd_data) = root_cmd_data else {
        return Err(format!(
            "The command ``{}`` does not exist [no root configuration found?]",
            name
        )
        .into());
    };

    // Check if theres any module configuration
    let module_configuration = sqlx::query!(
        "SELECT id, guild_id, module, disabled FROM guild_module_configurations WHERE guild_id = $1 AND module = $2",
        guild_id,
        SILVERPELT_CACHE.command_id_module_map.get(root_cmd).ok_or::<crate::Error>("Unknown error determining module of command".into())?,
    )
    .fetch_optional(pool)
    .await?
    .map(|rec| GuildModuleConfiguration {
        id: rec.id.hyphenated().to_string(),
        guild_id: rec.guild_id,
        module: rec.module,
        disabled: rec.disabled,
    });

    let mut cmd_data = root_cmd_data
        .get("")
        .unwrap_or(&CommandExtendedData::default())
        .clone();
    for command in permutations.iter() {
        let cmd_replaced = command
            .replace(&root_cmd.to_string(), "")
            .trim()
            .to_string();
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

    Ok((cmd_data, command_configuration, module_configuration))
}

pub mod canonical_module;
pub mod cmd;
pub mod ext_generate;
pub mod member_permission_calc;
pub mod module_config;
pub mod permissions;
pub mod permodule_toggle;
pub mod settings_poise;
pub mod silverpelt_cache;
pub mod utils;
pub mod validators;

// Load the common core types
pub use splashcore_rs::types::silverpelt::{
    CommandExtendedData, CommandExtendedDataMap, GuildCommandConfiguration,
    GuildModuleConfiguration, PermissionCheck, PermissionChecks, PermissionResult,
};

use futures::future::BoxFuture;
use std::sync::Arc;

pub type Command = poise::Command<crate::Data, crate::Error>;

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

    /// Whether or not individual commands in the module can be toggled
    pub commands_toggleable: bool,

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
    pub config_options: Vec<module_settings::types::ConfigOption>,

    pub __parsed: bool,
}

impl Module {
    /// Parses a module, while this doesnt really do anything right now, it may be used in the future
    pub fn parse(self) -> Module {
        #[poise::command(prefix_command, slash_command, rename = "")]
        pub async fn base_cmd(_ctx: crate::Context<'_>) -> Result<(), crate::Error> {
            Ok(())
        }

        let mut parsed = self;

        // If virtual module, all commands must also be virtual, if root command is virtual, all subcommands must be virtual
        for command in &mut parsed.commands {
            let root_is_virtual = {
                match command.1.get("") {
                    Some(root) => root.virtual_command,
                    None => false,
                }
            };
            for (_, extended_data) in command.1.iter_mut() {
                if parsed.virtual_module || root_is_virtual {
                    extended_data.virtual_command = true;
                }
            }
        }

        // acl__{module}_defaultperms_check is a special command that is added to all modules
        let mut acl_module_defaultperms_check = base_cmd();
        acl_module_defaultperms_check.name = format!("acl__{}_defaultperms_check", parsed.id);
        acl_module_defaultperms_check.qualified_name =
            format!("acl__{}_defaultperms_check", parsed.id);
        parsed.commands.push((
            acl_module_defaultperms_check,
            CommandExtendedData::none_map(),
        ));

        parsed.__parsed = true;
        parsed
    }

    pub fn is_parsed(&self) -> bool {
        self.__parsed
    }
}

pub mod cache;
pub mod canonical_module;
pub mod cmd;
pub mod data;
pub mod jobserver;
pub mod member_permission_calc;
pub mod module_config;
pub mod settings_poise;
pub mod types;
pub mod utils;
pub mod validators;

use crate::types::{CommandExtendedData, CommandExtendedDataMap};

use futures_util::future::BoxFuture;
use std::sync::Arc;

pub type Error = Box<dyn std::error::Error + Send + Sync>; // This is constant and should be copy pasted
pub type Command = poise::Command<data::Data, Error>;
pub type Context<'a> = poise::Context<'a, data::Data, Error>;

pub struct EventHandlerContext {
    pub guild_id: serenity::all::GuildId,
    pub full_event: serenity::all::FullEvent,
    pub data: Arc<data::Data>,
    pub serenity_context: serenity::all::Context,
}

pub type ModuleEventHandler =
    Box<dyn Send + Sync + for<'a> Fn(&'a EventHandlerContext) -> BoxFuture<'a, Result<(), Error>>>;

pub type OnStartupFunction =
    Box<dyn Send + Sync + for<'a> Fn(&'a data::Data) -> BoxFuture<'a, Result<(), Error>>>;

pub type OnReadyFunction = Box<
    dyn Send
        + Sync
        + for<'a> Fn(serenity::all::Context, &'a data::Data) -> BoxFuture<'a, Result<(), Error>>,
>;

pub type BackgroundTask = (
    botox::taskman::Task,
    fn(&serenity::all::Context) -> (bool, String),
);

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

    /// Background tasks (if any), first argument is the task
    ///
    /// Second is a function that returns whether the task should be added
    pub background_tasks: Vec<BackgroundTask>,

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
    /// Parses a module and performs basic checks before starting the bot to ensure a proper module setup
    pub fn parse(self) -> Module {
        #[poise::command(prefix_command, slash_command, rename = "")]
        pub async fn base_cmd(_ctx: crate::Context<'_>) -> Result<(), Error> {
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
            indexmap::indexmap! {
                "" => CommandExtendedData {
                    virtual_command: true,
                    ..Default::default()
                },
            },
        ));

        // Check: Ensure all command extended data's have valid subcommands listed
        for (command, extended_data) in &parsed.commands {
            let mut listed_subcommands = Vec::new();
            let mut actual_subcommands = Vec::new();

            for (subcommand, _) in extended_data.iter() {
                listed_subcommands.push(subcommand.to_string());
            }

            for subcommand in &command.subcommands {
                actual_subcommands.push(subcommand.name.clone());
            }

            // We don't care about omission of "" (rootcmd) here
            if !listed_subcommands.contains(&"".to_string()) {
                listed_subcommands.insert(0, "".to_string());
            }

            if !actual_subcommands.contains(&"".to_string()) {
                actual_subcommands.insert(0, "".to_string());
            }

            if listed_subcommands != actual_subcommands {
                panic!(
                    "Module {} has a command {} with subcommands that do not match the actual subcommands [{} != {}]",
                    parsed.id,
                    command.name,
                    listed_subcommands.join(", "),
                    actual_subcommands.join(", ")
                );
            }
        }

        // Check that all config_opts have unique ids
        let mut config_ids = Vec::new();

        for config_opt in &parsed.config_options {
            if config_ids.contains(&config_opt.id) {
                panic!(
                    "Module {} has a duplicate config option id: {}",
                    parsed.id, config_opt.id
                );
            }

            config_ids.push(config_opt.id);
        }

        parsed.__parsed = true;
        parsed
    }

    pub fn is_parsed(&self) -> bool {
        self.__parsed
    }
}

pub mod canonical_module;
pub mod cmd;
pub mod ext_generate;
pub mod member_permission_calc;
pub mod module_config;
pub mod settings_poise;
pub mod silverpelt_cache;
pub mod utils;
pub mod validators;

// Load the common core types
pub use splashcore_rs::types::silverpelt::{
    CommandExtendedData, CommandExtendedDataMap, GuildCommandConfiguration,
    GuildModuleConfiguration, PermissionCheck, PermissionChecks, PermissionResult,
};

use futures_util::future::BoxFuture;
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

/// This test ensures that all modules can be parsed
#[cfg(test)]
pub mod test_module_parse {
    #[test]
    fn test_module_parse() {
        let _ = crate::modules::modules();
    }

    #[tokio::test]
    async fn check_modules_test() {
        // Check for env var CHECK_MODULES_TEST_ENABLED
        if std::env::var("CHECK_MODULES_TEST_ENABLED").unwrap_or_default() == "true" {
            return;
        }

        // Set current directory to ../../
        let current_dir = std::env::current_dir().unwrap();

        if current_dir.ends_with("core/rust.bot_modules") {
            std::env::set_current_dir("../../").unwrap();
        }

        let pg_pool = sqlx::postgres::PgPoolOptions::new()
            .connect(&config::CONFIG.meta.postgres_url)
            .await
            .expect("Could not initialize connection");

        for module in crate::modules::modules() {
            assert!(module.is_parsed());

            // Ensure that all settings have all columns
            for config_opt in module.config_options {
                let mut missing_columns = Vec::new();

                for column in config_opt.columns.iter() {
                    missing_columns.push(column.id.to_string());
                }

                let cache = serenity::all::Cache::new();
                let http = serenity::all::Http::new("DUMMY");
                let cache_http = botox::cache::CacheHttpImpl {
                    cache: cache.into(),
                    http: http.into(),
                };
                let reqwest_client = reqwest::Client::new();

                let mut data_store = config_opt
                    .data_store
                    .create(
                        &config_opt,
                        &cache_http,
                        &reqwest_client,
                        &pg_pool,
                        serenity::all::GuildId::new(1),
                        serenity::all::UserId::new(1),
                        &base_data::permodule::DummyPermoduleFunctionExecutor {},
                        indexmap::IndexMap::new(),
                    )
                    .await
                    .unwrap();

                let columns = data_store.columns().await.unwrap();

                println!(
                    "Module: {}, Config Opt: {}, Columns: {:?}",
                    module.id, config_opt.id, columns
                );

                for column in columns {
                    if let Some(index) = missing_columns.iter().position(|x| x == &column) {
                        missing_columns.remove(index);
                    }
                }

                if !missing_columns.is_empty() {
                    panic!(
                        "Module {} has a config option {} with missing columns: {}",
                        module.id,
                        config_opt.id,
                        missing_columns.join(", ")
                    );
                }
            }
        }
    }
}

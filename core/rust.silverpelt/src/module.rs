use std::sync::Arc;

pub type CommandObj = (crate::Command, crate::CommandExtendedDataMap);

/// The `Module` trait can be used to create/define modules that run on Anti-Raid
///
/// A trait is used here to avoid a ton of complicated BoxFuture's, make Default handling more explicit and customizable and makes creating new Modules easier
pub trait Module: Send + Sync {
    /// The ID of the module
    fn id(&self) -> &'static str;

    /// The name of the module
    fn name(&self) -> &'static str;

    /// The description of the module
    fn description(&self) -> &'static str;

    /// Whether or not the module should be visible on the websites command lists
    fn web_hidden(&self) -> bool {
        false
    }

    /// Whether or the module can be enabled and/or disabled
    fn toggleable(&self) -> bool {
        true
    }

    /// Whether or not individual commands in the module can be toggled
    fn commands_toggleable(&self) -> bool {
        true
    }

    /// Virtual module. These modules allow controlling functionality of the bot without having its commands loaded into the bot
    ///
    /// Note that commands on a virtual module must also be virtual as well
    fn virtual_module(&self) -> bool {
        false
    }

    /// Whether the module is enabled or disabled by default
    fn is_default_enabled(&self) -> bool {
        false // Don't enable new modules by default unless modules explicitly opt in to this behavior
    }

    /// The commands in the module
    fn raw_commands(&self) -> Vec<CommandObj> {
        Vec::new()
    }

    /// The full command list of the module
    ///
    /// Note that modules should not need to override this function (normally)
    ///
    /// When in doubt, just implement `raw_commands` instead
    fn full_command_list(&self) -> Vec<CommandObj> {
        create_full_command_list(self.id(), self.raw_commands(), self.config_options())
    }

    /// Event listeners for the module
    fn event_listeners(&self) -> Option<Box<dyn ModuleEventListeners>> {
        None
    }

    /// Background tasks (if any), first argument is the task
    ///
    /// Second is a function that returns whether the task should be added
    fn background_tasks(&self) -> Vec<crate::BackgroundTask> {
        Vec::new()
    }

    /// Modules may store files on seaweed, in order to allow for usage tracking,
    /// s3_paths should be set to the paths of the files on seaweed
    fn s3_paths(&self) -> Vec<String> {
        Vec::new()
    }

    /// Config options for this module
    fn config_options(&self) -> Vec<module_settings::types::ConfigOption> {
        Vec::new()
    }

    /// Whether the module is a 'root'/sudo module. These modules will only be accessible
    /// to a whitelist-defined set of users
    fn root_module(&self) -> bool {
        false
    }

    /// What punishment actions this module provides
    fn punishment_actions(&self) -> Vec<Arc<dyn crate::punishments::CreatePunishmentAction>> {
        Vec::new()
    }

    /// Performs any sanity/validation checks on the module
    ///
    /// Should not be overrided by modules unless absolutely necessary
    fn validate(&self) -> Result<(), crate::Error> {
        validate_module(self)
    }
}

#[async_trait::async_trait]
pub trait ModuleEventListeners: Send + Sync {
    /// Event handler for the module
    ///
    /// Modules requiring multiple event_handlers will have to handle that themselves
    async fn event_handler(
        &self,
        _ctx: &crate::ar_event::EventHandlerContext,
    ) -> Result<(), crate::Error> {
        Ok(())
    }

    /// Filter the inbound events to a module
    ///
    /// This function is called before the event_handler function
    ///
    /// Returning false here will prevent a new tokio task from being spawned hence
    /// making things more efficient
    fn event_handler_filter(&self, _event: &crate::ar_event::AntiraidEvent) -> bool;
}

/// Validates a module to ensure it is set up correctly
pub fn validate_module<T: Module + ?Sized>(module: &T) -> Result<(), crate::Error> {
    let commands = module.raw_commands();

    // If virtual module, all commands must also be virtual, if root command is virtual, all subcommands must be virtual
    for command in commands.iter() {
        let root_is_virtual = {
            match command.1.get("") {
                Some(root) => root.virtual_command,
                None => false,
            }
        };
        for (sub_name, extended_data) in command.1.iter() {
            if module.virtual_module() && !extended_data.virtual_command {
                return Err(format!(
                    "Module {} is a virtual module, but has a non-virtual command {}",
                    module.id(),
                    command.0.name
                )
                .into());
            }

            if root_is_virtual && !extended_data.virtual_command {
                return Err(format!(
                    "Module {} has a virtual root command, but a non-virtual subcommand {} {}",
                    module.id(),
                    command.0.name,
                    sub_name
                )
                .into());
            }
        }
    }

    // Check: Ensure all command extended data's have valid subcommands listed
    for (command, extended_data) in commands.iter() {
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
            return Err(
                format!(
                    "Module {} has a command {} with subcommands that do not match the actual subcommands [{} != {}]",
                    module.id(),
                    command.name,
                    listed_subcommands.join(", "),
                    actual_subcommands.join(", ")
                ).into()
            );
        }
    }

    // Check that all config_opts have unique ids
    let mut config_ids = Vec::new();

    for config_opt in &module.config_options() {
        if config_ids.contains(&config_opt.id) {
            panic!(
                "Module {} has a duplicate config option id: {}",
                module.id(),
                config_opt.id
            );
        }

        config_ids.push(config_opt.id);
    }

    Ok(())
}

fn create_full_command_list(
    module_id: &str,
    commands: Vec<CommandObj>,
    config_options: Vec<module_settings::types::ConfigOption>,
) -> Vec<CommandObj> {
    #[poise::command(prefix_command, slash_command, rename = "")]
    pub async fn base_cmd(_ctx: crate::Context<'_>) -> Result<(), crate::Error> {
        Ok(())
    }

    let mut commands = commands;

    // acl__{module}_defaultperms_check is a special command that is added to all modules
    let mut acl_module_defaultperms_check = base_cmd();
    acl_module_defaultperms_check.name = format!("acl__{}_defaultperms_check", module_id);
    acl_module_defaultperms_check.qualified_name = format!("acl__{}_defaultperms_check", module_id);
    commands.push((
        acl_module_defaultperms_check,
        indexmap::indexmap! {
            "" => crate::CommandExtendedData {
                virtual_command: true,
                ..Default::default()
            },
        },
    ));

    // Add in the settings related commands
    for config_opt in config_options {
        let created_cmd =
            crate::settings_autogen::create_poise_commands_from_setting(module_id, &config_opt);

        let mut extended_data = indexmap::IndexMap::new();

        for (op, operation) in config_opt.operations.iter() {
            extended_data.insert(
                operation.corresponding_command,
                crate::CommandExtendedData::kittycat_or_admin(
                    module_id,
                    &format!("{}{}", op.to_string().to_lowercase(), &config_opt.id),
                ),
            );
        }

        commands.push((created_cmd, extended_data));
    }

    commands
}

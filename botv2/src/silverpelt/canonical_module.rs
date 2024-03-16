use indexmap::IndexMap;

/// Canonical representation of a module for external use
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct CanonicalModule {
    /// The ID of the module
    pub id: String,

    /// The name of the module
    pub name: String,

    /// The description of the module
    pub description: String,

    /// Whether or not the module should be visible on the websites command lists
    pub web_hidden: bool,

    /// Whether or the module can be enabled and/or disabled
    pub toggleable: bool,

    /// Whether or not individual commands in the module can be configured
    pub commands_configurable: bool,

    /// Virtual module. These modules allow controlling certain functionality of the bot without being loaded into the actual bot
    pub virtual_module: bool,

    /// Whether the module is enabled or disabled by default
    pub is_default_enabled: bool,

    /// The commands in the module
    pub commands: Vec<CanonicalCommand>,
}

/// Canonical representation of a command (data section) for external use
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct CanonicalCommand {
    pub command: CanonicalCommandData,
    pub extended_data: IndexMap<String, crate::silverpelt::CommandExtendedData>,
}

/// Canonical representation of a command argument for external use
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct CanonicalCommandArgument {
    /// The name of the argument
    pub name: String,

    /// The description of the argument
    pub description: Option<String>,

    /// Whether or not the argument is required
    pub required: bool,

    /// The choices available for the argument
    pub choices: Vec<String>,
}

/// Canonical representation of a command (data section) for external use
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct CanonicalCommandData {
    /// The name of the command
    pub name: String,

    /// The qualified name of the command
    pub qualified_name: String,

    /// The description of the command
    pub description: Option<String>,

    /// NSFW status
    pub nsfw: bool,

    /// The subcommands of the command
    pub subcommands: Vec<CanonicalCommandData>,

    /// Whether or not a subcommand is required or not
    pub subcommand_required: bool,

    /// The arguments of the command
    pub arguments: Vec<CanonicalCommandArgument>,
}

/// Given a command, return its canonical representation
impl CanonicalCommand {
    pub fn from_repr(
        cmd: &crate::silverpelt::Command,
        extended_data: IndexMap<&'static str, crate::silverpelt::CommandExtendedData>,
    ) -> Self {
        CanonicalCommand {
            command: cmd.into(),
            extended_data: extended_data
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        }
    }
}

/// Given command data, return its canonical representation
impl From<&crate::silverpelt::Command> for CanonicalCommandData {
    fn from(cmd: &crate::silverpelt::Command) -> Self {
        CanonicalCommandData {
            name: cmd.name.clone(),
            qualified_name: cmd.qualified_name.clone(),
            description: cmd.description.clone(),
            nsfw: cmd.nsfw_only,
            subcommands: cmd
                .subcommands
                .iter()
                .map(CanonicalCommandData::from)
                .collect(),
            subcommand_required: cmd.subcommand_required,
            arguments: cmd
                .parameters
                .iter()
                .map(|arg| CanonicalCommandArgument {
                    name: arg.name.clone(),
                    description: arg.description.clone(),
                    required: arg.required,
                    choices: arg
                        .choices
                        .iter()
                        .map(|choice| choice.name.to_string())
                        .collect(),
                })
                .collect(),
        }
    }
}

/// Given a module, return its canonical representation
impl From<crate::silverpelt::Module> for CanonicalModule {
    fn from(module: crate::silverpelt::Module) -> Self {
        CanonicalModule {
            id: module.id.to_string(),
            name: module.name.to_string(),
            description: module.description.to_string(),
            toggleable: module.toggleable,
            commands_configurable: module.commands_configurable,
            virtual_module: module.virtual_module,
            web_hidden: module.web_hidden,
            is_default_enabled: module.is_default_enabled,
            commands: module
                .commands
                .into_iter()
                .map(|(cmd, perms)| CanonicalCommand::from_repr(&cmd, perms))
                .collect(),
        }
    }
}

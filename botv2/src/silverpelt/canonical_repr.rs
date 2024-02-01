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

    /// Whether or the module is configurable
    pub configurable: bool,

    /// Whether or not individual commands in the module can be configured
    pub commands_configurable: bool,

    /// Whether the module is enabled or disabled by default
    pub is_default_enabled: bool,

    /// The commands in the module
    pub commands: Vec<CanonicalCommand>,
}

/// Canonical representation of a extended command data for external use
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct CanonicalCommandExtendedData {
    pub id: String,

    #[serde(flatten)]
    pub data: super::CommandExtendedData
}

/// Canonical representation of a command (data section) for external use
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct CanonicalCommand {
    pub command: CanonicalCommandData,
    pub extended_data: Vec<CanonicalCommandExtendedData>,
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
    pub fn from_repr(cmd: &super::Command, extended_data: super::CommandExtendedDataMap) -> Self {
        CanonicalCommand {
            command: cmd.into(),
            extended_data: {
                let mut v = Vec::new();

                for (id, data) in extended_data {
                    v.push(CanonicalCommandExtendedData {
                        id: id.to_string(),
                        data,
                    });
                }

                v
            },
        }
    }
}

/// Given command data, return its canonical representation
impl From<&super::Command> for CanonicalCommandData {
    fn from(cmd: &super::Command) -> Self {
        CanonicalCommandData {
            name: cmd.name.clone(),
            qualified_name: cmd.qualified_name.clone(),
            description: cmd.description.clone(),
            nsfw: cmd.nsfw_only,
            subcommands: cmd.subcommands.iter().map(|cmd| {
                CanonicalCommandData::from(cmd)
            }).collect(),
            subcommand_required: cmd.subcommand_required,
            arguments: cmd.parameters.iter().map(|arg| {
                CanonicalCommandArgument {
                    name: arg.name.clone(),
                    description: arg.description.clone(),
                    required: arg.required,
                    choices: arg.choices.iter().map(|choice| {
                        choice.name.to_string()
                    }).collect(),
                }
            }).collect(),
        }
    }
}

/// Given a module, return its canonical representation
impl From<super::Module> for CanonicalModule {
    fn from(module: super::Module) -> Self {
        CanonicalModule {
            id: module.id.to_string(),
            name: module.name.to_string(),
            description: module.description.to_string(),
            configurable: module.configurable,
            commands_configurable: module.commands_configurable,
            web_hidden: module.web_hidden,
            is_default_enabled: module.is_default_enabled,
            commands: module.commands.into_iter().map(|(cmd, perms)| {
                CanonicalCommand::from_repr(&cmd, perms)
            }).collect(),
        }
    }
}
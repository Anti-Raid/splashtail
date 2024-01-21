/// Canonical representation of a module for external use
#[derive(serde::Serialize, serde::Deserialize)]
pub struct CanonicalModule {
    /// The ID of the module
    pub id: &'static str,

    /// The name of the module
    pub name: &'static str,    

    /// The commands in the module
    pub commands: Vec<CanonicalCommand>,
}

pub type CanonicalCommandExtendedDataMap = indexmap::IndexMap<String, super::CommandExtendedData>;

/// Canonical representation of a command (data section) for external use
#[derive(serde::Serialize, serde::Deserialize)]
pub struct CanonicalCommand {
    pub command: CanonicalCommandData,
    pub extended_data: CanonicalCommandExtendedDataMap,
}

impl CanonicalCommand {
    pub fn from_repr(cmd: &super::Command, extended_data: super::CommandExtendedDataMap) -> Self {
        CanonicalCommand {
            command: cmd.into(),
            extended_data: extended_data.into_iter().map(|(k, v)| {
                (k.to_string(), v)
            }).collect(),
        }
    }
}

/// Canonical representation of a command argument for external use
#[derive(serde::Serialize, serde::Deserialize)]
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
#[derive(serde::Serialize, serde::Deserialize)]
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
            id: module.id,
            name: module.name,
            commands: module.commands.into_iter().map(|(cmd, perms)| {
                CanonicalCommand::from_repr(&cmd, perms)
            }).collect(),
        }
    }
}
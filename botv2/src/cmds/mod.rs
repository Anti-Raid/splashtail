pub mod core;
pub mod limits;

use once_cell::sync::Lazy;

/// List of enabled commands
/// 
/// Add to this list to enable a command
pub fn enabled_commands() -> Vec<Vec<CommandAndPermissions>> {
    vec![
        core::commands(),
        limits::commands(),
    ]
}

#[derive(Default, Clone, PartialEq)]
pub struct NativePermissions {
    /// The permission level needed to run this command (discord)
    pub perms: Vec<serenity::all::Permissions>,
    /// Whether or not the perms are ANDed (all needed) or OD'd (at least one)
    pub and: bool,
}

#[derive(Default, Clone, PartialEq)]
pub struct KittycatPermissions {
    /// The permission level needed to run this command (kittycat)
    pub perms: Vec<String>,
    /// Whether or not the perms are ANDed (all needed) or OD'd (at least one)
    pub and: bool,
}

#[derive(Clone, PartialEq, Default)]
pub struct CommandExtendedData {
    /// The permission level needed to run this command (kittycat)
    pub kittycat_perms: Option<KittycatPermissions>,
    /// The corresponding native permission(s)
    pub native_perms: Option<NativePermissions>,
}

pub type Command = poise::Command<crate::Data, crate::Error>;
pub type CommandAndPermissions = (Command, CommandExtendedData);

/// Command extra data (permissions)
pub static COMMAND_EXTRA_DATA: Lazy<indexmap::IndexMap<String, CommandExtendedData>> = Lazy::new(|| {
    let mut map = indexmap::IndexMap::new();
    
    for commands in enabled_commands() {
        for (command, extended_data) in commands {
            map.insert(command.name.clone(), extended_data);
        }
    }

    map
});
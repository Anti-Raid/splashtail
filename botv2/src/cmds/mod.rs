pub mod core;
pub mod limits;

use std::str::FromStr;
use once_cell::sync::Lazy;

pub type Command = poise::Command<crate::Data, crate::Error>;
pub type CommandAndPermissions = (Command, CommandExtendedData);

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

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize, strum_macros::EnumString, strum_macros::EnumVariantNames)]
pub enum GuildCommandConfigurationPermissionMethod {
    /// The permission method is native
    Native,
    /// The permission method is kittycat
    Kittycat,
    /// Unknown permission method
    Unknown,
}

/// Guild command configuration data
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GuildCommandConfiguration {
    /// Thd ID
    pub id: String,
    /// The guild id (from db)
    pub guild_id: String,
    /// The command name
    pub command: String,
    /// The permission method (kittycat)
    pub permission_method: GuildCommandConfigurationPermissionMethod,
    /// Whether or not the command is disabled
    pub disabled: bool,
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
pub async fn get_command_configuration(pool: &sqlx::PgPool, guild_id: &str, name: &str) -> Result<Option<GuildCommandConfiguration>, crate::Error> {
    let permutations = permute_command_names(name);

    let mut command_configuration = None;

    for permutation in permutations {
        let rec = sqlx::query!(
            "SELECT id, guild_id, command, perm_method, disabled FROM guild_command_configurations WHERE guild_id = $1 AND command = $2",
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
                permission_method: GuildCommandConfigurationPermissionMethod::from_str(&rec.perm_method).unwrap_or(GuildCommandConfigurationPermissionMethod::Unknown),
                disabled: rec.disabled,
            });
        }
    }

    Ok(command_configuration)
}

impl CommandExtendedData {
    pub async fn can_run_command(&self, cache_http: &crate::impls::cache::CacheHttpImpl, pool: &sqlx::PgPool, cmd: &Command, member: &serenity::all::Member) -> Result<(), crate::Error> {
        let Some(command_config) = get_command_configuration(pool, &member.guild_id.to_string(), &cmd.qualified_name).await? else {
            return Ok(()); // No command configuration, so we can run the command
        };

        if command_config.disabled {
            return Err(
                format!("The command ``{}`` (inherited from ``{}`` is disabled on this server", cmd.qualified_name, command_config.command).into()
            );
        }

        if self.kittycat_perms.is_none() && self.native_perms.is_none() {
            return Ok(()); // Optimisation: Early return if no perms are set
        }

        match command_config.permission_method {
            GuildCommandConfigurationPermissionMethod::Native => {
                if let Some(native_perms) = &self.native_perms {
                    let mut has_perms = false;

                    let member_perms = member.permissions(&cache_http.cache)
                    .map_err(|e| format!("Failed to get member permissions: {}", e))?;
                    
                    if native_perms.and {
                        for perm in &native_perms.perms {
                            if !member_perms.contains(*perm) {
                                return Err(
                                    format!("You do not have the required permissions to run this command (``{}``) implied from ``{}``. You need ``{}`` permissions to execute this command.", cmd.qualified_name, command_config.command, perm).into()
                                );
                            }
                        }
                    } else {
                        for perm in &native_perms.perms {
                            if member_perms.contains(*perm) {
                                has_perms = true;
                                break;
                            }
                        }

                        if !has_perms {
                            let mut perms = Vec::new();
                            for perm in &native_perms.perms {
                                perms.push(perm.get_permission_names().iter().map(|s| s.to_string()).collect::<Vec<String>>().join(",").to_string());
                            }

                            let perms: String = perms.join(" | ");

                            return Err(
                                format!("You do not have the required permissions to run this command (``{}``) implied from ``{}``. You need at least one of the following permissions to execute this command: ``{}``", cmd.qualified_name, command_config.command, perms).into()
                            );
                        }
                    }
                } else {
                    return Err(
                        format!("The command ``{}`` (inherited from ``{}`` does *not* support native permissions yet! Please ask a higher-up to edit server settings!", cmd.qualified_name, command_config.command).into()
                    );
                }
            },
            GuildCommandConfigurationPermissionMethod::Kittycat => {
                if let Some(kittycat_perms) = &self.kittycat_perms {
                    let mut has_perms = false;

                    let member_perms: Vec<String> = Vec::new(); // TODO: Implement support for fetching member perms

                    if kittycat_perms.and {
                        for perm in &kittycat_perms.perms {
                            if !kittycat::perms::has_perm(&member_perms, perm) {
                                return Err(
                                    format!("You do not have the required permissions to run this command (``{}``) implied from ``{}``. You need ``{}`` permissions to execute this command.", cmd.qualified_name, command_config.command, perm).into()
                                );
                            }
                        }
                    } else {
                        for perm in &kittycat_perms.perms {
                            if member_perms.contains(perm) {
                                has_perms = true;
                                break;
                            }
                        }

                        if !has_perms {
                            let perms = kittycat_perms.perms.join(" | ");

                            return Err(
                                format!("You do not have the required permissions to run this command (``{}``) implied from ``{}``. You need at least one of the following permissions to execute this command: ``{}``", cmd.qualified_name, command_config.command, perms).into()
                            );
                        }
                    }
                }
            },
            GuildCommandConfigurationPermissionMethod::Unknown => {
                return Err(
                    format!("The command ``{}`` (inherited from ``{}`` is disabled on this server", cmd.qualified_name, command_config.command).into()
                );
            },
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_names_to_check() {
        println!("{:?}", permute_command_names("limits hit view"));
        assert_eq!(permute_command_names("limits"), vec!["limits"]);
        assert_eq!(permute_command_names("limits hit"), vec!["limits", "limits hit"]);
        assert_eq!(permute_command_names("limits hit add"), vec!["limits", "limits hit", "limits hit add"]);
    }
}
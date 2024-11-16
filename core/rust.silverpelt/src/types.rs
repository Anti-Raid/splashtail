use indexmap::{indexmap, IndexMap};
use permissions::types::PermissionCheck;

pub type CommandExtendedDataMap = IndexMap<&'static str, CommandExtendedData>;

// @ci go=CommandExtendedData
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize, Debug)]
pub struct CommandExtendedData {
    /// The default permissions needed to run this command
    pub default_perms: PermissionCheck,
    /// Whether the command is enabled by default or not
    pub is_default_enabled: bool,
    /// Whether the command should be hidden on the website or not
    pub web_hidden: bool,
    /// Whether the command is a virtual command or not (virtual commands are not loaded into the bot, but can be used for permission checks etc)
    pub virtual_command: bool,
}

impl Default for CommandExtendedData {
    fn default() -> Self {
        Self {
            default_perms: PermissionCheck {
                kittycat_perms: vec![],
                native_perms: vec![],
                inner_and: false,
            },
            is_default_enabled: true,
            web_hidden: false,
            virtual_command: false,
        }
    }
}

impl CommandExtendedData {
    pub fn none() -> Self {
        CommandExtendedData {
            default_perms: PermissionCheck {
                kittycat_perms: vec![],
                native_perms: vec![],
                inner_and: false,
            },
            is_default_enabled: true,
            web_hidden: false,
            virtual_command: false,
        }
    }

    pub fn none_map() -> CommandExtendedDataMap {
        indexmap! {
            "" => CommandExtendedData {
                default_perms: PermissionCheck {
                    kittycat_perms: vec![],
                    native_perms: vec![],
                    inner_and: false,
                },
                is_default_enabled: true,
                web_hidden: false,
                virtual_command: false,
            },
        }
    }

    pub fn kittycat_simple(namespace: &str, permission: &str) -> CommandExtendedData {
        CommandExtendedData {
            default_perms: PermissionCheck {
                kittycat_perms: vec![format!("{}.{}", namespace, permission)],
                native_perms: vec![],
                inner_and: false,
            },
            is_default_enabled: true,
            web_hidden: false,
            virtual_command: false,
        }
    }

    pub fn kittycat_or_admin(namespace: &str, permission: &str) -> CommandExtendedData {
        CommandExtendedData {
            default_perms: PermissionCheck {
                kittycat_perms: vec![format!("{}.{}", namespace, permission)],
                native_perms: vec![serenity::all::Permissions::ADMINISTRATOR],
                inner_and: false,
            },
            is_default_enabled: true,
            web_hidden: false,
            virtual_command: false,
        }
    }
}

/// Guild command configuration data
#[derive(Clone, Hash, Eq, PartialEq, serde::Serialize, serde::Deserialize, Debug)]
pub struct GuildCommandConfiguration {
    /// The ID
    pub id: String,
    /// The guild id (from db)
    pub guild_id: String,
    /// The command name
    pub command: String,
    /// The permission checks on the command, if unset, will revert to the default perms set on the command itself
    pub perms: Option<PermissionCheck>,
    /// Whether or not the command is disabled. None means to use the default command configuration
    pub disabled: Option<bool>,
}

impl GuildCommandConfiguration {
    pub async fn to_full_guild_command_configuration(
        self,
        pool: &sqlx::PgPool,
    ) -> Result<FullGuildCommandConfiguration, crate::Error> {
        let id = self.id.parse::<sqlx::types::uuid::Uuid>()?;
        let audit_info = sqlx::query!(
            r#"
            SELECT created_at, created_by, last_updated_at, last_updated_by
            FROM guild_command_configurations
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(pool)
        .await?;

        Ok(FullGuildCommandConfiguration {
            id: self.id,
            guild_id: self.guild_id,
            command: self.command,
            perms: self.perms,
            disabled: self.disabled,
            created_at: audit_info.created_at,
            created_by: audit_info.created_by,
            last_updated_at: audit_info.last_updated_at,
            last_updated_by: audit_info.last_updated_by,
        })
    }
}

/// Full guild command configuration data including audit info etc.
#[derive(Clone, Hash, Eq, PartialEq, serde::Serialize, serde::Deserialize, Debug)]
pub struct FullGuildCommandConfiguration {
    /// The ID
    pub id: String,
    /// The guild id (from db)
    pub guild_id: String,
    /// The command name
    pub command: String,
    /// The permission checks on the command, if unset, will revert to the default perms set on the command itself
    pub perms: Option<PermissionCheck>,
    /// Whether or not the command is disabled. None means to use the default command configuration
    pub disabled: Option<bool>,
    /// The time the command configuration was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// The user who created the command configuration
    pub created_by: String,
    /// The time the command configuration was last updated
    pub last_updated_at: chrono::DateTime<chrono::Utc>,
    /// The user who last updated the command configuration
    pub last_updated_by: String,
}

impl FullGuildCommandConfiguration {
    fn to_guild_command_configuration(&self) -> GuildCommandConfiguration {
        GuildCommandConfiguration {
            id: self.id.clone(),
            guild_id: self.guild_id.clone(),
            command: self.command.clone(),
            perms: self.perms.clone(),
            disabled: self.disabled,
        }
    }
}

impl From<FullGuildCommandConfiguration> for GuildCommandConfiguration {
    fn from(f: FullGuildCommandConfiguration) -> Self {
        f.to_guild_command_configuration()
    }
}

/// Guild module configuration data
#[derive(Clone, Hash, Eq, PartialEq, serde::Serialize, serde::Deserialize, Debug)]
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

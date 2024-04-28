use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CanonicalColumnType {
    #[serde(rename = "String")]
    #[default]
    String,
    #[serde(rename = "Integer")]
    Integer,
    #[serde(rename = "Boolean")]
    Boolean,
    #[serde(rename = "User")]
    User,
    #[serde(rename = "Channel")]
    Channel,
    #[serde(rename = "Role")]
    Role,
    #[serde(rename = "Emoji")]
    Emoji,
    #[serde(rename = "Message")]
    Message,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalColumn {
    /// The ID of the column
    pub id: String,
    
    /// The friendly name of the column
    pub name: String,
    
    /// The type of the column
    pub column_type: CanonicalColumnType,
    
    /// Whether or not the column is nullable
    pub nullable: bool,
    
    /// Whether or not the column is unique
    pub unique: bool,
    
    /// Whether or not the column is an array
    pub array: bool,
    
    /// Internal: the column hint, may be used in autocomplete etc.
    pub hint: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalConfigOption {
    /// The ID of the option
    pub id: String,

    /// The name of the option
    pub name: String,
    
    /// The description of the option
    pub description: String,
    
    /// The table name for the config option
    pub table: String,
    
    /// The column name refering to the guild id of the config option    
    pub guild_id: String,
            
    /// The columns for this option
    pub columns: Vec<CanonicalColumn>,
    
    /// Whether or not the row must exist before hand
    pub row_must_exist: bool,
    
    /// Config option hint, used internally for stuff like guild channel configuration
    pub hint: Option<String>,
}

/// Given a module, return its canonical representation
impl From<crate::silverpelt::config_opt::ConfigOption> for CanonicalConfigOption {
    fn from(module: crate::silverpelt::config_opt::ConfigOption) -> Self {
        Self {
            id: module.id.to_string(),
            table: module.table.to_string(),
            guild_id: module.guild_id.to_string(),
            name: module.name.to_string(),
            description: module.description.to_string(),
            columns: module.columns.into_iter().map(|c| CanonicalColumn {
                id: c.id.to_string(),
                name: c.name.to_string(),
                column_type: match c.column_type {
                    crate::silverpelt::config_opt::ColumnType::String => CanonicalColumnType::String,
                    crate::silverpelt::config_opt::ColumnType::Integer => CanonicalColumnType::Integer,
                    crate::silverpelt::config_opt::ColumnType::Boolean => CanonicalColumnType::Boolean,
                    crate::silverpelt::config_opt::ColumnType::User => CanonicalColumnType::User,
                    crate::silverpelt::config_opt::ColumnType::Channel => CanonicalColumnType::Channel,
                    crate::silverpelt::config_opt::ColumnType::Role => CanonicalColumnType::Role,
                    crate::silverpelt::config_opt::ColumnType::Emoji => CanonicalColumnType::Emoji,
                    crate::silverpelt::config_opt::ColumnType::Message => CanonicalColumnType::Message,
                },
                nullable: c.nullable,
                unique: c.unique,
                array: c.array,
                hint: c.hint,
            }).collect(),
            row_must_exist: module.row_must_exist,
            hint: module.hint,
        }
    }
}

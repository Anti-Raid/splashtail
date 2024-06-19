use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum CanonicalColumnType {
    /// A single valued column (scalar)
    Scalar {
        /// The value type
        column_type: CanonicalInnerColumnType,
    },
    /// An array column
    Array {
        /// The inner type of the array
        inner: CanonicalInnerColumnType,
    },
}

impl From<super::config_opts::ColumnType> for CanonicalColumnType {
    fn from(column_type: super::config_opts::ColumnType) -> Self {
        match column_type {
            super::config_opts::ColumnType::Scalar { column_type } => CanonicalColumnType::Scalar {
                column_type: column_type.into(),
            },
            super::config_opts::ColumnType::Array { inner } => CanonicalColumnType::Array {
                inner: inner.into(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum CanonicalInnerColumnTypeStringKind {
    /// Normal string
    Normal,
    /// User
    User,
    /// Channel
    Channel,
    /// Role
    Role,
    /// Emoji
    Emoji,
    /// Message
    Message,
}

impl From<super::config_opts::InnerColumnTypeStringKind> for CanonicalInnerColumnTypeStringKind {
    fn from(kind: super::config_opts::InnerColumnTypeStringKind) -> Self {
        match kind {
            super::config_opts::InnerColumnTypeStringKind::Normal => {
                CanonicalInnerColumnTypeStringKind::Normal
            }
            super::config_opts::InnerColumnTypeStringKind::User => {
                CanonicalInnerColumnTypeStringKind::User
            }
            super::config_opts::InnerColumnTypeStringKind::Channel => {
                CanonicalInnerColumnTypeStringKind::Channel
            }
            super::config_opts::InnerColumnTypeStringKind::Role => {
                CanonicalInnerColumnTypeStringKind::Role
            }
            super::config_opts::InnerColumnTypeStringKind::Emoji => {
                CanonicalInnerColumnTypeStringKind::Emoji
            }
            super::config_opts::InnerColumnTypeStringKind::Message => {
                CanonicalInnerColumnTypeStringKind::Message
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum CanonicalInnerColumnType {
    Uuid {},
    String {
        min_length: Option<usize>,
        max_length: Option<usize>,
        allowed_values: Vec<String>,
        kind: CanonicalInnerColumnTypeStringKind,
    },
    Timestamp {},
    TimestampTz {},
    Integer {},
    Float {},
    BitFlag {
        /// The bit flag values
        values: indexmap::IndexMap<String, i64>,
    },
    Boolean {},
    Json {},
}

impl From<super::config_opts::InnerColumnType> for CanonicalInnerColumnType {
    fn from(column_type: super::config_opts::InnerColumnType) -> Self {
        match column_type {
            super::config_opts::InnerColumnType::Uuid {} => CanonicalInnerColumnType::Uuid {},
            super::config_opts::InnerColumnType::String {
                min_length,
                max_length,
                allowed_values,
                kind,
            } => CanonicalInnerColumnType::String {
                min_length,
                max_length,
                allowed_values: allowed_values.iter().map(|s| s.to_string()).collect(),
                kind: kind.into(),
            },
            super::config_opts::InnerColumnType::Timestamp {} => {
                CanonicalInnerColumnType::Timestamp {}
            }
            super::config_opts::InnerColumnType::TimestampTz {} => {
                CanonicalInnerColumnType::TimestampTz {}
            }
            super::config_opts::InnerColumnType::Integer {} => CanonicalInnerColumnType::Integer {},
            super::config_opts::InnerColumnType::Float {} => CanonicalInnerColumnType::Float {},
            super::config_opts::InnerColumnType::BitFlag { values } => {
                CanonicalInnerColumnType::BitFlag {
                    values: values
                        .into_iter()
                        .map(|(k, v)| (k.to_string(), v))
                        .collect::<indexmap::IndexMap<String, i64>>(),
                }
            }
            super::config_opts::InnerColumnType::Boolean {} => CanonicalInnerColumnType::Boolean {},
            super::config_opts::InnerColumnType::Json {} => CanonicalInnerColumnType::Json {},
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CanonicalColumnSuggestion {
    Static {
        suggestions: Vec<String>,
    },
    Dynamic {
        table_name: String,
        column_name: String,
    },
    None {},
}

impl From<super::config_opts::ColumnSuggestion> for CanonicalColumnSuggestion {
    fn from(column_suggestion: super::config_opts::ColumnSuggestion) -> Self {
        match column_suggestion {
            super::config_opts::ColumnSuggestion::Static { suggestions } => {
                CanonicalColumnSuggestion::Static {
                    suggestions: suggestions.iter().map(|s| s.to_string()).collect(),
                }
            }
            super::config_opts::ColumnSuggestion::Dynamic {
                table_name,
                column_name,
            } => CanonicalColumnSuggestion::Dynamic {
                table_name: table_name.to_string(),
                column_name: column_name.to_string(),
            },
            super::config_opts::ColumnSuggestion::None {} => CanonicalColumnSuggestion::None {},
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalColumn {
    /// The ID of the column
    pub id: String,

    /// The friendly name of the column
    pub name: String,

    /// The type of the column
    pub column_type: CanonicalColumnType,

    /// Whether or not the column is nullable
    pub nullable: bool,

    /// Suggestions to display
    pub suggestions: CanonicalColumnSuggestion,

    /// Whether or not the column is unique
    pub unique: bool,

    /// For which operations should the field be ignored for (essentially, read only)
    ///
    /// Note that checks for this column will still be applied (use an empty array in pre_checks to disable checks)
    ///
    /// Semantics:
    /// View => The column is removed from the list of columns sent to the consumer. The key is set to its current value when executing the actions
    /// Create => The column is not handled on the client however actions are still executed. The key itself is set to None when executing the actions
    /// Update => The column is not handled on the client however actions are still executed. The key itself is set to None when executing the actions
    /// Delete => The column is not handled on the client however actions are still executed. The key itself is set to None when executing the actions
    pub ignored_for: Vec<CanonicalOperationType>,
}

impl From<super::config_opts::Column> for CanonicalColumn {
    fn from(column: super::config_opts::Column) -> Self {
        Self {
            id: column.id.to_string(),
            name: column.name.to_string(),
            column_type: column.column_type.into(),
            nullable: column.nullable,
            suggestions: column.suggestions.into(),
            unique: column.unique,
            ignored_for: column.ignored_for.into_iter().map(|o| o.into()).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalOperationSpecific {
    /// The corresponding command for ACL purposes
    pub corresponding_command: String,

    /// Any columns to set. For example, a last_updated column should be set on update
    ///
    /// Variables:
    /// - {now} => the current timestamp
    ///
    /// Format: {column_name} => {value}
    ///
    /// Note: updating columns outside of the table itself
    ///
    /// In Create/Update, these columns are directly included in the create/update itself
    pub columns_to_set: indexmap::IndexMap<String, String>,
}

impl From<super::config_opts::OperationSpecific> for CanonicalOperationSpecific {
    fn from(operation_specific: super::config_opts::OperationSpecific) -> Self {
        Self {
            corresponding_command: operation_specific.corresponding_command.to_string(),
            columns_to_set: operation_specific
                .columns_to_set
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum CanonicalOperationType {
    #[serde(rename = "View")]
    View,
    #[serde(rename = "Create")]
    Create,
    #[serde(rename = "Update")]
    Update,
    #[serde(rename = "Delete")]
    Delete,
}

impl From<super::config_opts::OperationType> for CanonicalOperationType {
    fn from(operation_type: super::config_opts::OperationType) -> Self {
        match operation_type {
            super::config_opts::OperationType::View => CanonicalOperationType::View,
            super::config_opts::OperationType::Create => CanonicalOperationType::Create,
            super::config_opts::OperationType::Update => CanonicalOperationType::Update,
            super::config_opts::OperationType::Delete => CanonicalOperationType::Delete,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

    /// The primary key of the table
    pub primary_key: String,

    /// The columns for this option
    pub columns: Vec<CanonicalColumn>,

    /// Operation specific data
    pub operations: indexmap::IndexMap<CanonicalOperationType, CanonicalOperationSpecific>,
}

/// Given a module, return its canonical representation
impl From<super::config_opts::ConfigOption> for CanonicalConfigOption {
    fn from(module: super::config_opts::ConfigOption) -> Self {
        Self {
            id: module.id.to_string(),
            table: module.table.to_string(),
            guild_id: module.guild_id.to_string(),
            name: module.name.to_string(),
            description: module.description.to_string(),
            columns: module.columns.into_iter().map(|c| c.into()).collect(),
            primary_key: module.primary_key.to_string(),
            operations: module
                .operations
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        }
    }
}

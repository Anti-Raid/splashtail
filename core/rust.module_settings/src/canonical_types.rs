use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CanonicalSettingsError {
    /// Operation not supported
    OperationNotSupported {
        operation: CanonicalOperationType,
    },
    /// Generic error
    Generic {
        message: String,
        src: String,
        typ: String,
    },
    /// Schema type validation error
    SchemaTypeValidationError {
        column: String,
        expected_type: String,
        got_type: String,
    },
    /// Schema null value validation error
    SchemaNullValueValidationError {
        column: String,
    },
    /// Schema check validation error
    SchemaCheckValidationError {
        column: String,
        check: String,
        error: String,
        accepted_range: String,
    },
    /// Missing or invalid field
    MissingOrInvalidField {
        field: String,
        src: String,
    },
    RowExists {
        column_id: String,
        count: i64,
    },
    RowDoesNotExist {
        column_id: String,
    },
    MaximumCountReached {
        max: usize,
        current: usize,
    },
}

impl From<super::types::SettingsError> for CanonicalSettingsError {
    fn from(error: super::types::SettingsError) -> Self {
        match error {
            super::types::SettingsError::OperationNotSupported { operation } => {
                CanonicalSettingsError::OperationNotSupported {
                    operation: operation.into(),
                }
            }
            super::types::SettingsError::Generic { message, src, typ } => {
                CanonicalSettingsError::Generic {
                    message: message.to_string(),
                    src: src.to_string(),
                    typ: typ.to_string(),
                }
            }
            super::types::SettingsError::SchemaTypeValidationError {
                column,
                expected_type,
                got_type,
            } => CanonicalSettingsError::SchemaTypeValidationError {
                column: column.to_string(),
                expected_type: expected_type.to_string(),
                got_type: got_type.to_string(),
            },
            super::types::SettingsError::SchemaNullValueValidationError { column } => {
                CanonicalSettingsError::SchemaNullValueValidationError {
                    column: column.to_string(),
                }
            }
            super::types::SettingsError::SchemaCheckValidationError {
                column,
                check,
                error,
                accepted_range,
            } => CanonicalSettingsError::SchemaCheckValidationError {
                column: column.to_string(),
                check: check.to_string(),
                error: error.to_string(),
                accepted_range: accepted_range.to_string(),
            },
            super::types::SettingsError::MissingOrInvalidField { field, src } => {
                CanonicalSettingsError::MissingOrInvalidField {
                    field: field.to_string(),
                    src: src.to_string(),
                }
            }
            super::types::SettingsError::RowExists { column_id, count } => {
                CanonicalSettingsError::RowExists {
                    column_id: column_id.to_string(),
                    count,
                }
            }
            super::types::SettingsError::RowDoesNotExist { column_id } => {
                CanonicalSettingsError::RowDoesNotExist {
                    column_id: column_id.to_string(),
                }
            }
            super::types::SettingsError::MaximumCountReached { max, current } => {
                CanonicalSettingsError::MaximumCountReached { max, current }
            }
        }
    }
}

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

impl From<super::types::ColumnType> for CanonicalColumnType {
    fn from(column_type: super::types::ColumnType) -> Self {
        match column_type {
            super::types::ColumnType::Scalar { column_type } => CanonicalColumnType::Scalar {
                column_type: column_type.into(),
            },
            super::types::ColumnType::Array { inner } => CanonicalColumnType::Array {
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

impl From<super::types::InnerColumnTypeStringKind> for CanonicalInnerColumnTypeStringKind {
    fn from(kind: super::types::InnerColumnTypeStringKind) -> Self {
        match kind {
            super::types::InnerColumnTypeStringKind::Normal => {
                CanonicalInnerColumnTypeStringKind::Normal
            }
            super::types::InnerColumnTypeStringKind::User => {
                CanonicalInnerColumnTypeStringKind::User
            }
            super::types::InnerColumnTypeStringKind::Channel => {
                CanonicalInnerColumnTypeStringKind::Channel
            }
            super::types::InnerColumnTypeStringKind::Role => {
                CanonicalInnerColumnTypeStringKind::Role
            }
            super::types::InnerColumnTypeStringKind::Emoji => {
                CanonicalInnerColumnTypeStringKind::Emoji
            }
            super::types::InnerColumnTypeStringKind::Message => {
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

impl From<super::types::InnerColumnType> for CanonicalInnerColumnType {
    fn from(column_type: super::types::InnerColumnType) -> Self {
        match column_type {
            super::types::InnerColumnType::Uuid {} => CanonicalInnerColumnType::Uuid {},
            super::types::InnerColumnType::String {
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
            super::types::InnerColumnType::Timestamp {} => CanonicalInnerColumnType::Timestamp {},
            super::types::InnerColumnType::TimestampTz {} => {
                CanonicalInnerColumnType::TimestampTz {}
            }
            super::types::InnerColumnType::Integer {} => CanonicalInnerColumnType::Integer {},
            super::types::InnerColumnType::Float {} => CanonicalInnerColumnType::Float {},
            super::types::InnerColumnType::BitFlag { values } => {
                CanonicalInnerColumnType::BitFlag {
                    values: values
                        .into_iter()
                        .map(|(k, v)| (k.to_string(), v))
                        .collect::<indexmap::IndexMap<String, i64>>(),
                }
            }
            super::types::InnerColumnType::Boolean {} => CanonicalInnerColumnType::Boolean {},
            super::types::InnerColumnType::Json {} => CanonicalInnerColumnType::Json {},
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CanonicalColumnSuggestion {
    Static {
        suggestions: Vec<String>,
    },
    Dynamic {
        /// The table name to query
        table_name: String,
        /// The column name to query for the user-displayed value
        value_column: String,
        /// The column name to query for the id
        id_column: String,
        /// The column name to query for the guild id
        guild_id_column: String,
    },
    None {},
}

impl From<super::types::ColumnSuggestion> for CanonicalColumnSuggestion {
    fn from(column_suggestion: super::types::ColumnSuggestion) -> Self {
        match column_suggestion {
            super::types::ColumnSuggestion::Static { suggestions } => {
                CanonicalColumnSuggestion::Static {
                    suggestions: suggestions.iter().map(|s| s.to_string()).collect(),
                }
            }
            super::types::ColumnSuggestion::Dynamic {
                table_name,
                value_column,
                id_column,
                guild_id_column,
            } => CanonicalColumnSuggestion::Dynamic {
                table_name: table_name.to_string(),
                value_column: value_column.to_string(),
                id_column: id_column.to_string(),
                guild_id_column: guild_id_column.to_string(),
            },
            super::types::ColumnSuggestion::None {} => CanonicalColumnSuggestion::None {},
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

    /// Whether or not the column is a secret, if so, usize stores the length of the secret that should be generated in reset
    pub secret: Option<usize>,
}

impl From<&super::types::Column> for CanonicalColumn {
    fn from(column: &super::types::Column) -> Self {
        Self {
            id: column.id.to_string(),
            name: column.name.to_string(),
            column_type: column.column_type.clone().into(),
            nullable: column.nullable,
            suggestions: column.suggestions.clone().into(),
            unique: column.unique,
            ignored_for: column
                .ignored_for
                .iter()
                .map(|o| o.clone().into())
                .collect(),
            secret: column.secret,
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

impl From<super::types::OperationSpecific> for CanonicalOperationSpecific {
    fn from(operation_specific: super::types::OperationSpecific) -> Self {
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

impl From<super::types::OperationType> for CanonicalOperationType {
    fn from(operation_type: super::types::OperationType) -> Self {
        match operation_type {
            super::types::OperationType::View => CanonicalOperationType::View,
            super::types::OperationType::Create => CanonicalOperationType::Create,
            super::types::OperationType::Update => CanonicalOperationType::Update,
            super::types::OperationType::Delete => CanonicalOperationType::Delete,
        }
    }
}

impl From<CanonicalOperationType> for super::types::OperationType {
    fn from(operation_type: CanonicalOperationType) -> super::types::OperationType {
        match operation_type {
            CanonicalOperationType::View => super::types::OperationType::View,
            CanonicalOperationType::Create => super::types::OperationType::Create,
            CanonicalOperationType::Update => super::types::OperationType::Update,
            CanonicalOperationType::Delete => super::types::OperationType::Delete,
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

    /// Maximum number of entries a server may have
    pub max_entries: usize,

    /// Operation specific data
    pub operations: indexmap::IndexMap<CanonicalOperationType, CanonicalOperationSpecific>,
}

/// Given a module, return its canonical representation
impl From<super::types::ConfigOption> for CanonicalConfigOption {
    fn from(module: super::types::ConfigOption) -> Self {
        Self {
            id: module.id.to_string(),
            table: module.table.to_string(),
            guild_id: module.guild_id.to_string(),
            name: module.name.to_string(),
            description: module.description.to_string(),
            columns: module.columns.iter().map(|c| c.into()).collect(),
            primary_key: module.primary_key.to_string(),
            max_entries: module.max_entries,
            operations: module
                .operations
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        }
    }
}

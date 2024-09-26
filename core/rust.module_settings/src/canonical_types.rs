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
pub struct CanonicalColumnTypeDynamicClause {
    /// The field to check in state (lite templating [only variable substitution] is allowed)
    pub field: String,
    /// The value to check for
    pub value: serde_json::Value,
    /// The column type to set if the value matches
    pub column_type: CanonicalColumnType,
}

impl From<super::types::ColumnTypeDynamicClause> for CanonicalColumnTypeDynamicClause {
    fn from(clause: super::types::ColumnTypeDynamicClause) -> Self {
        Self {
            field: clause.field.to_string(),
            value: clause.value.to_json(),
            column_type: clause.column_type.into(),
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
    /// Dynamic type that changes based on the value of another field
    ///
    /// Dynamic types are the one case where the field order matters.
    Dynamic {
        /// The clauses to check for setting the actual kind
        clauses: Vec<CanonicalColumnTypeDynamicClause>,
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
            super::types::ColumnType::Dynamic { clauses } => CanonicalColumnType::Dynamic {
                clauses: clauses.into_iter().map(|v| (v.into())).collect(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum CanonicalInnerColumnTypeStringKindTemplateKind {
    /// Template for formatting messages
    Message {},
}

impl From<super::types::InnerColumnTypeStringKindTemplateKind>
    for CanonicalInnerColumnTypeStringKindTemplateKind
{
    fn from(kind: super::types::InnerColumnTypeStringKindTemplateKind) -> Self {
        match kind {
            super::types::InnerColumnTypeStringKindTemplateKind::Message {} => {
                CanonicalInnerColumnTypeStringKindTemplateKind::Message {}
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum CanonicalInnerColumnTypeStringKind {
    /// Normal string
    Normal {},
    /// A token that is autogenerated if not provided by the user
    Token {
        /// The default length of the secret if not provided by the user
        default_length: usize,
    },
    /// A textarea
    Textarea {},
    /// A template string
    Template {
        /// The kind of template
        kind: CanonicalInnerColumnTypeStringKindTemplateKind,
    },
    /// A kittycat permission
    KittycatPermission {},
    /// User
    User {},
    /// Channel
    Channel {
        allowed_types: Vec<serenity::all::ChannelType>,
        needed_bot_permissions: serenity::model::permissions::Permissions,
    },
    /// Role
    Role {},
    /// Emoji
    Emoji {},
    /// Message
    Message {},
}

impl From<super::types::InnerColumnTypeStringKind> for CanonicalInnerColumnTypeStringKind {
    fn from(kind: super::types::InnerColumnTypeStringKind) -> Self {
        match kind {
            super::types::InnerColumnTypeStringKind::Normal => {
                CanonicalInnerColumnTypeStringKind::Normal {}
            }
            super::types::InnerColumnTypeStringKind::Token { default_length } => {
                CanonicalInnerColumnTypeStringKind::Token { default_length }
            }
            super::types::InnerColumnTypeStringKind::Textarea => {
                CanonicalInnerColumnTypeStringKind::Textarea {}
            }
            super::types::InnerColumnTypeStringKind::Template { kind } => {
                CanonicalInnerColumnTypeStringKind::Template { kind: kind.into() }
            }
            super::types::InnerColumnTypeStringKind::KittycatPermission => {
                CanonicalInnerColumnTypeStringKind::KittycatPermission {}
            }
            super::types::InnerColumnTypeStringKind::User => {
                CanonicalInnerColumnTypeStringKind::User {}
            }
            super::types::InnerColumnTypeStringKind::Channel {
                allowed_types,
                needed_bot_permissions,
            } => CanonicalInnerColumnTypeStringKind::Channel {
                allowed_types,
                needed_bot_permissions,
            },
            super::types::InnerColumnTypeStringKind::Role => {
                CanonicalInnerColumnTypeStringKind::Role {}
            }
            super::types::InnerColumnTypeStringKind::Emoji => {
                CanonicalInnerColumnTypeStringKind::Emoji {}
            }
            super::types::InnerColumnTypeStringKind::Message => {
                CanonicalInnerColumnTypeStringKind::Message {}
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
    Interval {},
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
            super::types::InnerColumnType::Interval {} => CanonicalInnerColumnType::Interval {},
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
    /// A reference to another setting
    ///
    /// The primary key of the referred setting is used as the value
    SettingsReference {
        /// The module of the referenced setting
        module: String,
        /// The setting of the referenced setting
        setting: String,
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
            super::types::ColumnSuggestion::SettingsReference { module, setting } => {
                CanonicalColumnSuggestion::SettingsReference {
                    module: module.to_string(),
                    setting: setting.to_string(),
                }
            }
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

    /// The description of the column
    pub description: String,

    /// The type of the column
    pub column_type: CanonicalColumnType,

    /// Whether or not the column is nullable
    pub nullable: bool,

    /// The default value of the column
    pub default: Option<serde_json::Value>,

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

    /// Whether or not the column is a secret
    ///
    /// Note that secret columns are not present in view or update actions unless explicitly provided by the user. ignored_for rules continue to apply.
    ///
    /// The exact semantics of a secret column depend on column type (a String of kind token will lead to autogeneration of a token for example)
    pub secret: bool,
}

impl From<&super::types::Column> for CanonicalColumn {
    fn from(column: &super::types::Column) -> Self {
        Self {
            id: column.id.to_string(),
            name: column.name.to_string(),
            description: column.description.to_string(),
            column_type: column.column_type.clone().into(),
            nullable: column.nullable,
            default: column.default.clone().map(|v| v(true).to_json()),
            suggestions: column.suggestions.clone().into(),
            unique: column.unique,
            ignored_for: column.ignored_for.iter().map(|o| (*o).into()).collect(),
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

    /// The common filters to apply to all crud operations on this config options
    ///
    /// For example, this can be used for guild_id scoped config options or non-guild scoped config options
    ///
    /// Semantics:
    ///
    /// View/Update/Delete: Common filters are applied to the view operation as an extension of all other filters
    /// Create: Common filters are appended on to the entry itself
    pub common_filters:
        indexmap::IndexMap<CanonicalOperationType, indexmap::IndexMap<String, String>>,

    /// The default common filter
    pub default_common_filters: indexmap::IndexMap<String, String>,

    /// The primary key of the table
    pub primary_key: String,

    /// Title template, used for the title of the embed
    pub title_template: String,

    /// The columns for this option
    pub columns: Vec<CanonicalColumn>,

    /// Maximum number of entries to return
    ///
    /// Only applies to View operations
    pub max_return: i64,

    /// Maximum number of entries a server may have
    pub max_entries: Option<usize>,

    /// Operation specific data
    pub operations: indexmap::IndexMap<CanonicalOperationType, CanonicalOperationSpecific>,
}

/// Given a module, return its canonical representation
impl From<super::types::ConfigOption> for CanonicalConfigOption {
    fn from(module: super::types::ConfigOption) -> Self {
        Self {
            id: module.id.to_string(),
            table: module.table.to_string(),
            common_filters: module
                .common_filters
                .into_iter()
                .map(|(k, v)| {
                    (
                        k.into(),
                        v.into_iter()
                            .map(|(k, v)| (k.to_string(), v.to_string()))
                            .collect(),
                    )
                })
                .collect(),
            default_common_filters: module
                .default_common_filters
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            name: module.name.to_string(),
            description: module.description.to_string(),
            columns: module.columns.iter().map(|c| c.into()).collect(),
            primary_key: module.primary_key.to_string(),
            title_template: module.title_template.to_string(),
            max_return: module.max_return,
            max_entries: module.max_entries,
            operations: module
                .operations
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        }
    }
}

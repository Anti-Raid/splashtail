// Common state variables:
//
// - {__author} => the user id of the user running the operation
// - {__guild_id} => the guild id of the guild the operation is being run in
//
// {__now} always returns the current timestamp (TimestampTz), {__now_naive} returns the current timestamp in naive form (Timestamp)
//
// Note that these special variables do not need to live in state and may instead be special cased
//
// For display purposes, the special case variable {[__column_id]_displaytype} can be set to allow displaying in a different form

use futures::future::BoxFuture;
use splashcore_rs::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsError {
    /// Operation not supported
    OperationNotSupported {
        operation: OperationType,
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
        value: Value,
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

impl std::fmt::Display for SettingsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SettingsError::Generic { message, src, typ } => {
                write!(f, "{} from src `{}` of type `{}`", message, src, typ)
            }
            SettingsError::OperationNotSupported { operation } => {
                write!(f, "Operation `{}` is not supported", operation)
            }
            SettingsError::SchemaTypeValidationError {
                column,
                expected_type,
                got_type,
            } => write!(
                f,
                "Column `{}` expected type `{}`, got type `{}`",
                column, expected_type, got_type
            ),
            SettingsError::SchemaNullValueValidationError { column } => {
                write!(f, "Column `{}` is not nullable, yet value is null", column)
            }
            SettingsError::SchemaCheckValidationError {
                column,
                check,
                error,
                value,
                accepted_range,
            } => {
                write!(
                    f,
                    "Column `{}` failed check `{}` with value `{}`, accepted range: `{}`, error: `{}`",
                    column, check, value, accepted_range, error
                )
            }
            SettingsError::MissingOrInvalidField { field, src } => write!(f, "Missing (or invalid) field `{}` with src: {}", field, src),
            SettingsError::RowExists { column_id, count } => write!(
                f,
                "A row with the same column `{}` already exists. Count: {}",
                column_id, count
            ),
            SettingsError::RowDoesNotExist { column_id } => {
                write!(f, "A row with the same column `{}` does not exist", column_id)
            }
            SettingsError::MaximumCountReached { max, current } => write!(
                f,
                "The maximum number of entities this server may have ({}) has been reached. This server currently has {}.",
                max, current
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ColumnType {
    /// A single valued column (scalar)
    Scalar {
        /// The value type
        column_type: InnerColumnType,
    },
    /// An array column
    Array {
        /// The inner type of the array
        inner: InnerColumnType,
    },
}

impl ColumnType {
    /// Returns whether the column type is an array
    #[allow(dead_code)]
    pub fn is_array(&self) -> bool {
        matches!(self, ColumnType::Array { .. })
    }

    /// Returns whether the column type is a scalar
    #[allow(dead_code)]
    pub fn is_scalar(&self) -> bool {
        matches!(self, ColumnType::Scalar { .. })
    }

    pub fn new_scalar(inner: InnerColumnType) -> Self {
        ColumnType::Scalar { column_type: inner }
    }

    pub fn new_array(inner: InnerColumnType) -> Self {
        ColumnType::Array { inner }
    }
}

impl std::fmt::Display for ColumnType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColumnType::Scalar { column_type } => write!(f, "{}", column_type),
            ColumnType::Array { inner } => write!(f, "Array<{}>", inner),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum InnerColumnTypeStringKind {
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

impl std::fmt::Display for InnerColumnTypeStringKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InnerColumnTypeStringKind::Normal => write!(f, "Normal"),
            InnerColumnTypeStringKind::User => write!(f, "User"),
            InnerColumnTypeStringKind::Channel => write!(f, "Channel"),
            InnerColumnTypeStringKind::Role => write!(f, "Role"),
            InnerColumnTypeStringKind::Emoji => write!(f, "Emoji"),
            InnerColumnTypeStringKind::Message => write!(f, "Message"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum InnerColumnType {
    Uuid {},
    String {
        min_length: Option<usize>,
        max_length: Option<usize>,
        allowed_values: Vec<&'static str>, // If empty, all values are allowed
        kind: InnerColumnTypeStringKind,
    },
    Timestamp {},
    TimestampTz {},
    Integer {},
    Float {},
    BitFlag {
        /// The bit flag values
        values: indexmap::IndexMap<&'static str, i64>,
    },
    Boolean {},
    Json {},
}

impl std::fmt::Display for InnerColumnType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InnerColumnType::Uuid {} => write!(f, "Uuid"),
            InnerColumnType::String {
                min_length,
                max_length,
                allowed_values,
                kind,
            } => {
                write!(f, "String {}", kind)?;
                if let Some(min) = min_length {
                    write!(f, " (min length: {})", min)?;
                }
                if let Some(max) = max_length {
                    write!(f, " (max length: {})", max)?;
                }
                if !allowed_values.is_empty() {
                    write!(f, " (allowed values: {:?})", allowed_values)?;
                }
                Ok(())
            }
            InnerColumnType::Timestamp {} => write!(f, "Timestamp"),
            InnerColumnType::TimestampTz {} => write!(f, "TimestampTz"),
            InnerColumnType::Integer {} => write!(f, "Integer"),
            InnerColumnType::Float {} => write!(f, "Float"),
            InnerColumnType::BitFlag { values } => {
                write!(f, "BitFlag (values: ")?;
                for (i, (key, value)) in values.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", key, value)?;
                }
                write!(f, ")")
            }
            InnerColumnType::Boolean {} => write!(f, "Boolean"),
            InnerColumnType::Json {} => write!(f, "Json"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColumnSuggestion {
    Static {
        suggestions: Vec<&'static str>,
    },
    Dynamic {
        /// The table name to query
        table_name: &'static str,
        /// The column name to query for the user-displayed value
        value_column: &'static str,
        /// The column name to query for the id
        id_column: &'static str,
    },
    None {},
}

/// This is the context provided to all NativeAction's. Note that on_conditions have a slightly different structure
/// as they are synchronous functions and thus cannot use certain fields
#[allow(dead_code)]
pub struct NativeActionContext {
    pub author: serenity::all::UserId,
    pub guild_id: serenity::all::GuildId,
    pub pool: sqlx::PgPool,
}

pub type NativeActionFunc = Box<
    dyn Send
        + Sync
        + for<'a> Fn(
            NativeActionContext,
            &'a mut super::state::State,
        ) -> BoxFuture<'a, Result<(), SettingsError>>,
>;

#[allow(dead_code)]
pub struct ActionConditionContext {
    pub author: serenity::all::UserId,
    pub guild_id: serenity::all::GuildId,
}

pub type ActionCondition =
    fn(ActionConditionContext, &super::state::State) -> Result<bool, SettingsError>;

#[allow(dead_code)]
pub enum ColumnAction {
    /// Run a rust (native) action
    NativeAction {
        /// The action to run
        action: NativeActionFunc,
        /// Under what circumstance should the action be run
        on_condition: Option<ActionCondition>,
    },
    SetVariable {
        /// The key to set
        key: &'static str,

        /// The value to set
        value: serde_json::Value,

        /// Under what circumstance should the action be run
        on_condition: Option<ActionCondition>,
    },
    IpcPerModuleFunction {
        /// The module to use
        module: &'static str,

        /// The function to execute
        function: &'static str,

        /// The arguments to pass to the function
        ///
        /// In syntax: {key_on_function} -> {key_on_map}
        arguments: indexmap::IndexMap<&'static str, &'static str>,

        /// Under what circumstance should the action be run
        on_condition: Option<ActionCondition>,
    },
}

impl std::fmt::Debug for ColumnAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColumnAction::NativeAction { .. } => {
                write!(f, "NativeAction {{ action: <function> ")
            }
            ColumnAction::SetVariable {
                key,
                value,
                on_condition,
            } => {
                write!(
                    f,
                    "SetVariable {{ key: {}, value: {:?}, on_condition: {:?} }}",
                    key, value, on_condition
                )
            }
            ColumnAction::IpcPerModuleFunction {
                module,
                function,
                arguments,
                on_condition
            } => write!(
                f,
                "IpcPerModuleFunction {{ module: {}, function: {}, arguments: {:?}, on_condition: {:?} }}",
                module, function, arguments, on_condition
            ),
        }
    }
}

#[derive(Debug)]
pub struct Column {
    /// The ID of the column on the database
    pub id: &'static str,

    /// The friendly name of the column
    pub name: &'static str,

    /// The type of the column
    pub column_type: ColumnType,

    /// Whether or not the column is nullable
    ///
    /// Note that the point where nullability is checked may vary but will occur after pre_checks are executed
    pub nullable: bool,

    /// Suggestions to display
    pub suggestions: ColumnSuggestion,

    /// Whether or not the column is unique
    ///
    /// Note that the point where uniqueness is checked may vary but will occur after pre_checks are executed
    pub unique: bool,

    /// For which operations should the field be ignored for (essentially, read only)
    ///
    /// Note that checks for this column will still be applied (use an empty array in pre_checks to disable checks)
    ///
    /// Semantics:
    ///
    /// View => The column is removed from the list of columns sent to the consumer. The key is set to its current value when executing the actions
    ///
    /// Create => All column checks other than actions are ignored. The value itself will be set to None. The key itself is set to None in state
    ///
    /// Update => All column checks other than actions are ignored. The value itself will be set to None. The key itself is set to None in state
    ///
    /// Delete => All column checks other than actions are ignored. The value itself will be set to None. The key itself is set to None in state
    pub ignored_for: Vec<OperationType>,

    /// Pre-execute checks
    ///
    /// Note that these may run either during or after all fields are validated however the current (and all previous) columns
    /// are guaranteed to be set
    ///
    /// Note: pre_checks/default_pre_checks for a column will still execute if ignored_for is set for the operation however the value
    /// may be unset or Value::None
    pub pre_checks: indexmap::IndexMap<OperationType, Vec<ColumnAction>>,

    /// Default pre-execute checks to fallback to if the operation specific ones are not set
    ///
    /// Same rules as pre_checks apply
    pub default_pre_checks: Vec<ColumnAction>,
}

impl PartialEq for Column {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OperationSpecific {
    /// The corresponding command for ACL purposes
    pub corresponding_command: &'static str,

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
    pub columns_to_set: indexmap::IndexMap<&'static str, &'static str>,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
#[allow(dead_code)]
pub enum OperationType {
    View,
    Create,
    Update,
    Delete,
}

impl std::fmt::Display for OperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperationType::View => write!(f, "View"),
            OperationType::Create => write!(f, "Create"),
            OperationType::Update => write!(f, "Update"),
            OperationType::Delete => write!(f, "Delete"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ConfigOption {
    /// The ID of the option
    pub id: &'static str,

    /// The name of the option
    pub name: &'static str,

    /// The description of the option
    pub description: &'static str,

    /// The table name for the config option
    pub table: &'static str,

    /// The column name refering to the guild id of the config option    
    pub guild_id: &'static str,

    /// The primary key of the table
    pub primary_key: &'static str,

    /// The columns for this option
    pub columns: Vec<Column>,

    /// Maximum number of entries a server may have
    pub max_entries: usize,

    /// Operation specific data
    pub operations: indexmap::IndexMap<OperationType, OperationSpecific>,
}

use async_trait::async_trait;
use std::sync::Arc;

pub struct SettingsData {
    pub pool: sqlx::PgPool,
    pub reqwest: reqwest::Client,
    pub object_store: Arc<splashcore_rs::objectstore::ObjectStore>,
    pub cache_http: botox::cache::CacheHttpImpl,
    pub serenity_context: serenity::all::Context,
}

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
                    write!(f, "```{}```\n**Source:** `{}`\n**Error Type:** `{}`", message, src, typ)
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
                    accepted_range,
                } => {
                    write!(
                        f,
                        "Column `{}` failed check `{}`, accepted range: `{}`, error: `{}`",
                        column, check, accepted_range, error
                    )
                }
                SettingsError::MissingOrInvalidField { field, src } => write!(f, "Missing (or invalid) field `{}` with src: `{}`", field, src),
                SettingsError::RowExists { column_id, count } => write!(
                    f,
                    "A row with the same column `{}` already exists. Count: `{}`",
                    column_id, count
                ),
                SettingsError::RowDoesNotExist { column_id } => {
                    write!(f, "A row with the same column `{}` does not exist", column_id)
                }
                SettingsError::MaximumCountReached { max, current } => write!(
                    f,
                    "The maximum number of entities this server may have (`{}`) has been reached. This server currently has `{}`.",
                    max, current
                ),
            }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct ColumnTypeDynamicClause {
    /// The field to check in state (lite templating [only variable substitution] is allowed)
    pub field: &'static str,
    /// The value to check for
    pub value: splashcore_rs::value::Value,
    /// The column type to set if the value matches
    pub column_type: ColumnType,
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
    /// Dynamic type that changes based on the value of another field
    ///
    /// Dynamic types are the one case where the field order matters.
    Dynamic {
        /// The clauses to check for setting the actual kind
        clauses: Vec<ColumnTypeDynamicClause>,
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

    pub fn new_dynamic(clauses: Vec<ColumnTypeDynamicClause>) -> Self {
        ColumnType::Dynamic { clauses }
    }
}

impl std::fmt::Display for ColumnType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColumnType::Scalar { column_type } => write!(f, "{}", column_type),
            ColumnType::Array { inner } => write!(f, "Array<{}>", inner),
            ColumnType::Dynamic { clauses } => {
                write!(f, "Dynamic (possible clauses: ")?;
                for (i, clause) in clauses.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(
                        f,
                        "{}: {} -> {}",
                        clause.field, clause.value, clause.column_type
                    )?;
                }
                write!(f, ")")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum InnerColumnTypeStringKindTemplateKind {
    /// Template for formatting messages
    Message {},
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum InnerColumnTypeStringKind {
    /// Normal string
    Normal,
    /// A token that is autogenerated if not provided by the user
    Token {
        /// The default length of the secret if not provided by the user
        default_length: usize,
    },
    /// A textarea
    Textarea,
    /// A template string
    Template {
        kind: InnerColumnTypeStringKindTemplateKind,
    },
    /// A kittycat permission
    KittycatPermission,
    /// User
    User,
    /// Channel
    Channel {
        allowed_types: Vec<serenity::all::ChannelType>,
        needed_bot_permissions: serenity::model::permissions::Permissions,
    },
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
            InnerColumnTypeStringKind::Token { default_length } => {
                write!(f, "Token (default_length: {})", default_length)
            }
            InnerColumnTypeStringKind::Textarea => write!(f, "Textarea"),
            InnerColumnTypeStringKind::Template { kind } => write!(f, "Template {:?}", kind),
            InnerColumnTypeStringKind::KittycatPermission => write!(f, "KittycatPermission"),
            InnerColumnTypeStringKind::User => write!(f, "User"),
            InnerColumnTypeStringKind::Channel {
                allowed_types,
                needed_bot_permissions,
            } => {
                write!(
                    f,
                    "Channel: {:?}, with needed bot permissions: {:?}",
                    allowed_types, needed_bot_permissions
                )
            }
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
            InnerColumnType::Interval {} => write!(f, "Interval"),
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
    /// A reference to another setting
    ///
    /// The primary key of the referred setting is used as the value
    SettingsReference {
        /// The module of the referenced setting
        module: &'static str,
        /// The setting of the referenced setting
        setting: &'static str,
    },
    None {},
}

#[derive(Debug, Clone)]
pub struct Column {
    /// The ID of the column on the database
    pub id: &'static str,

    /// The friendly name of the column
    pub name: &'static str,

    /// The description of the column
    pub description: &'static str,

    /// The type of the column
    pub column_type: ColumnType,

    /// Whether or not the column is nullable
    ///
    /// Note that the point where nullability is checked may vary but will occur after pre_checks are executed
    pub nullable: bool,

    /// The default value of the column
    pub default: Option<fn(is_example: bool) -> splashcore_rs::value::Value>,

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
    /// View => The column is removed from the list of columns sent to the consumer. The value is set to its current value when executing the actions
    ///
    /// Create => All column checks other than actions are ignored. The value itself may or may not be set. The key itself is set to None in state
    ///
    /// Update => All column checks other than actions are ignored. The value itself will be set to its current (on-database) value [an unchanged field].
    ///
    /// Delete => No real effect. The column will still be set in state for Delete operations for actions to consume them.
    pub ignored_for: Vec<OperationType>,

    /// Whether or not the column is a secret
    ///
    /// Note that secret columns are not present in view or update actions unless explicitly provided by the user. ignored_for rules continue to apply.
    ///
    /// The exact semantics of a secret column depend on column type (a String of kind token will lead to autogeneration of a token for example)
    ///
    /// Due to secret still following ignore_for rules and internal implementation reasons, tokens etc. will not be autogenerated if the column has ignored_for set. In this case, a native action must be used
    pub secret: bool,
}

impl PartialEq for Column {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OperationSpecific {
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

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
#[allow(dead_code)]
pub enum OperationType {
    View,
    Create,
    Update,
    Delete,
}

impl OperationType {
    pub fn corresponding_command_suffix(&self) -> &'static str {
        match self {
            OperationType::View => "view",
            OperationType::Create => "create",
            OperationType::Update => "update",
            OperationType::Delete => "delete",
        }
    }
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

#[derive(Debug, Clone)]
pub struct ConfigOption {
    /// The ID of the option
    pub id: &'static str,

    /// The name of the option
    pub name: &'static str,

    /// The description of the option
    pub description: &'static str,

    /// The table name for the config option
    pub table: &'static str,

    /// The common filters to apply to all crud operations on this config options
    ///
    /// For example, this can be used for guild_id scoped config options or non-guild scoped config options
    ///
    /// Semantics:
    ///
    /// View/Update/Delete: Common filters are applied to the view operation as an extension of all other filters
    /// Create: Common filters are appended on to the entry itself
    pub common_filters:
        indexmap::IndexMap<OperationType, indexmap::IndexMap<&'static str, &'static str>>,

    /// The default common filter
    pub default_common_filters: indexmap::IndexMap<&'static str, &'static str>,

    /// The primary key of the table
    pub primary_key: &'static str,

    /// Title template, used for the title of the embed
    pub title_template: &'static str,

    /// The columns for this option
    pub columns: Arc<Vec<Column>>,

    /// Maximum number of entries to return
    ///
    /// Only applies to View operations
    pub max_return: i64,

    /// Maximum number of entries a server may have
    pub max_entries: Option<usize>,

    /// Operation specific data
    pub operations: indexmap::IndexMap<OperationType, OperationSpecific>,

    /// Any post-operation actions. These are guaranteed to run after the operation has been completed
    ///
    /// Note: this is pretty useless in View but may be useful in Create/Update/Delete
    ///
    /// If/when called, the state will be the state after the operation has been completed. In particular, the data itself may not be present in database anymore
    pub post_action: Arc<dyn PostAction>,

    /// What validator to use for this config option
    pub validator: Arc<dyn SettingDataValidator>,

    /// The underlying data store to use for fetch operations
    ///
    /// This can be useful in cases where postgres/etc. is not the main underlying storage (for example, seaweedfs etc.)
    pub data_store: Arc<dyn CreateDataStore>,
}

impl ConfigOption {
    pub fn get_corresponding_command(&self, op: OperationType) -> String {
        format!("{} {}", self.id, op.corresponding_command_suffix())
    }
}

impl PartialEq for ConfigOption {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

/// Wraps `v` in the currently used wrapper
///
/// Currently, this is an Arc for now
pub fn settings_wrap<T>(v: T) -> Arc<T> {
    Arc::new(v)
}

/// Trait to create a new data store
#[async_trait]
pub trait CreateDataStore: Send + Sync {
    /// Create a datastore performing any needed setup
    async fn create(
        &self,
        setting: &ConfigOption,
        guild_id: serenity::all::GuildId,
        author: serenity::all::UserId,
        data: &SettingsData,
        common_filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<Box<dyn DataStore>, SettingsError>;
}

impl std::fmt::Debug for dyn CreateDataStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CreateDataStore")
    }
}

/// How should data be fetched
#[async_trait]
pub trait DataStore: Send + Sync {
    /// Casts the DataStore to std::any::Any
    fn as_any(&mut self) -> &mut dyn std::any::Any;

    /// Start a transaction
    async fn start_transaction(&mut self) -> Result<(), SettingsError>;

    /// Commit the changes to the data store
    async fn commit(&mut self) -> Result<(), SettingsError>;

    /// Gets the list of all available columns on the database
    ///
    /// This can be useful for debugging purposes and validation/tests
    async fn columns(&mut self) -> Result<Vec<String>, SettingsError>;

    /// Fetches all requested fields of a setting for a given guild matching filters
    async fn fetch_all(
        &mut self,
        fields: &[String],
        filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<Vec<super::state::State>, SettingsError>;

    /// Fetch the count of all entries matching filters
    async fn matching_entry_count(
        &mut self,
        filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<usize, SettingsError>;

    /// Creates a new entry given a set of columns to set returning the newly created entry
    async fn create_entry(
        &mut self,
        entry: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<super::state::State, SettingsError>;

    /// Updates all matching entry given a set of columns to set and a set of filters
    async fn update_matching_entries(
        &mut self,
        filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
        entry: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<(), SettingsError>;

    /// Deletes entries given a set of filters
    ///
    /// NOTE: Data stores should return an error if no rows are deleted
    async fn delete_matching_entries(
        &mut self,
        filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<(), SettingsError>;
}

impl std::fmt::Debug for dyn DataStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DataStore")
    }
}

/// This is the context provided to all hooks (post-actions, validators, etc.)
#[allow(dead_code)]
pub struct HookContext<'a> {
    pub author: serenity::all::UserId,
    pub guild_id: serenity::all::GuildId,
    pub data_store: &'a mut dyn DataStore, // The current datastore
    pub data: &'a SettingsData,            // The data object
    pub operation_type: OperationType,
    pub unchanged_fields: Vec<String>, // The fields that have not changed in an operation
}

/// Settings can (optionally) have a validator to allow for custom data validation/processing prior to executing an operation
///
/// Note that validators are guaranteed to have all data of the column set in state when called
#[async_trait]
pub trait SettingDataValidator: Send + Sync {
    /// Validates the data
    async fn validate<'a>(
        &self,
        context: HookContext<'a>,
        state: &'a mut super::state::State,
    ) -> Result<(), SettingsError>;
}

impl std::fmt::Debug for dyn SettingDataValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SettingDataValidator")
    }
}

/// A simple NoOp validator
pub struct NoOpValidator;

#[async_trait]
impl SettingDataValidator for NoOpValidator {
    async fn validate<'a>(
        &self,
        _context: HookContext<'a>,
        _state: &'a mut super::state::State,
    ) -> Result<(), SettingsError> {
        Ok(())
    }
}

/// Settings can (optionally) have a post-action to allow for custom data validation/processing prior to executing an operation
///
/// Note that post-actions are guaranteed to have all data of the column set in state when called
#[async_trait]
pub trait PostAction: Send + Sync {
    /// Validates the data
    async fn post_action<'a>(
        &self,
        context: HookContext<'a>,
        state: &'a mut super::state::State,
    ) -> Result<(), SettingsError>;
}

impl std::fmt::Debug for dyn PostAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PostAction")
    }
}

/// A simple NoOp post-action
pub struct NoOpPostAction;

#[async_trait]
impl PostAction for NoOpPostAction {
    async fn post_action<'a>(
        &self,
        _context: HookContext<'a>,
        _state: &'a mut super::state::State,
    ) -> Result<(), SettingsError> {
        Ok(())
    }
}

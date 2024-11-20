use async_trait::async_trait;
use module_settings::{
    data_stores::{PostgresDataStore, PostgresDataStoreImpl},
    types::{
        settings_wrap, Column, ColumnSuggestion, ColumnType, CreateDataStore, DataStore, HookContext, InnerColumnType, InnerColumnTypeStringKind, NoOpPostAction, NoOpValidator, OperationSpecific, OperationType, Setting, SettingExecutor, SettingsData, SettingsError
    },
};
use splashcore_rs::value::Value;
use std::sync::LazyLock;

pub static LOCKDOWN_SETTINGS: LazyLock<Setting> = LazyLock::new(|| {
    Setting {
        id: "lockdown_guilds".to_string(),
        name: "Lockdown Settings".to_string(),
        description: "Setup standard lockdown settings for a server".to_string(),
        primary_key: "guild_id".to_string(),
        columns: settings_wrap(vec![
            module_settings::common_columns::guild_id(
                "guild_id",
                "Guild ID",
                "Guild ID of the server in question",
            ),
            Column {
                id: "member_roles".to_string(),
                name: "Member Roles".to_string(),
                description: "Which roles to use as member roles for the purpose of lockdown. These roles will be explicitly modified during lockdown".to_string(),
                column_type: ColumnType::new_array(InnerColumnType::String {
                    kind: InnerColumnTypeStringKind::Role,
                    min_length: None,
                    max_length: None,
                    allowed_values: vec![],
                }),
                nullable: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
            },
            Column {
                id: "require_correct_layout".to_string(),
                name: "Require Correct Layout".to_string(),
                description: "Whether or not a lockdown can proceed even without correct critical role permissions. May lead to partial lockdowns if disabled".to_string(),
                column_type: ColumnType::new_scalar(InnerColumnType::Boolean {}),
                nullable: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
            },
            module_settings::common_columns::created_at(),
            module_settings::common_columns::created_by(),
            module_settings::common_columns::last_updated_at(),
            module_settings::common_columns::last_updated_by(),
        ]),
        title_template: "Lockdown Settings".to_string(),
        supported_operations: vec![
            OperationType::View,
            OperationType::Save,
            OperationType::Delete,
        ],
    }
});

pub static LOCKDOWNS: LazyLock<Setting> = LazyLock::new(|| Setting {
    id: "lockdowns".to_string(),
    name: "Lockdowns".to_string(),
    description: "Lockdowns".to_string(),
    primary_key: "id".to_string(),
    columns: settings_wrap(vec![
        Column {
            id: "id".to_string(),
            name: "ID".to_string(),
            description: "The ID of the lockdown".to_string(),
            column_type: ColumnType::new_scalar(InnerColumnType::Uuid {}),
            nullable: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![OperationType::Create],
            secret: false,
        },
        module_settings::common_columns::guild_id(
            "guild_id",
            "Guild ID",
            "The Guild ID referring to this lockdown",
        ),
        Column {
            id: "type".to_string(),
            name: "Type".to_string(),
            description: "The type of the lockdown.".to_string(),
            column_type: ColumnType::new_scalar(InnerColumnType::String {
                kind: InnerColumnTypeStringKind::Normal,
                min_length: Some(1),
                max_length: Some(256),
                allowed_values: vec![],
            }),
            nullable: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
        },
        Column {
            id: "data".to_string(),
            name: "Data".to_string(),
            description: "The data stored of the lockdown.".to_string(),
            column_type: ColumnType::new_scalar(InnerColumnType::Json { max_bytes: None }),
            nullable: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![OperationType::Save],
            secret: false,
        },
        Column {
            id: "reason".to_string(),
            name: "Reason".to_string(),
            description: "The reason for starting the lockdown.".to_string(),
            column_type: ColumnType::new_scalar(InnerColumnType::String {
                kind: InnerColumnTypeStringKind::Normal,
                min_length: Some(1),
                max_length: Some(256),
                allowed_values: vec![],
            }),
            nullable: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
        },
        module_settings::common_columns::created_at(),
    ]),
    title_template: "Reason: {reason}".to_string(),
    supported_operations: vec![
        OperationType::View,
        OperationType::Save,
        OperationType::Delete,
    ],
});

pub struct LockdownExecutor;

#[async_trait]
impl SettingExecutor for LockdownExecutor {
    /// View the settings data
    ///
    /// __limit and __offset, if found, contains the limit/offset of the query
    ///
    /// All Executors should return an __count value containing the total count of the total number of entries
    async fn view<'a>(
        &self,
        context: HookContext<'a>,
        filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<Vec<indexmap::IndexMap<String, splashcore_rs::value::Value>>, silverpelt::Error> {
        Ok(vec![]) // TODO: Implement
    }

    /// Saves the setting
    async fn save<'a>(
        &self,
        context: HookContext<'a>,
        state: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<indexmap::IndexMap<String, splashcore_rs::value::Value>, silverpelt::Error> {
        Ok(indexmap::indexmap! {}) // TODO: Implement
    }

    /// Deletes the setting
    async fn delete<'a>(
        &self,
        context: HookContext<'a>,
        pkey: splashcore_rs::value::Value,
    ) -> Result<indexmap::IndexMap<String, splashcore_rs::value::Value>, silverpelt::Error> {
        Ok(indexmap::indexmap! {}) // TODO: Implement
    }
}

/// A custom data store is needed to handle the specific requirements of the lockdown module
pub struct LockdownDataStore {}

#[async_trait]
impl CreateDataStore for LockdownDataStore {
    async fn create(
        &self,
        setting: &Setting,
        guild_id: serenity::all::GuildId,
        author: serenity::all::UserId,
        data: &SettingsData,
        common_filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<Box<dyn DataStore>, SettingsError> {
        Ok(Box::new(LockdownDataStoreImpl {
            inner: (PostgresDataStore {})
                .create_impl(setting, guild_id, author, data, common_filters)
                .await?,
            cache: silverpelt::data::Data::silverpelt_cache(data),
            lockdown_data: lockdowns::LockdownData {
                cache_http: data.cache_http.clone(),
                pool: data.pool.clone(),
                reqwest: data.reqwest.clone(),
                object_store: data.object_store.clone(),
            },
        }))
    }
}

pub struct LockdownDataStoreImpl {
    inner: PostgresDataStoreImpl,
    cache: std::sync::Arc<silverpelt::cache::SilverpeltCache>,
    lockdown_data: lockdowns::LockdownData,
}

#[async_trait]
impl DataStore for LockdownDataStoreImpl {
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    async fn start_transaction(&mut self) -> Result<(), SettingsError> {
        Ok(()) // No-op for our use case
    }

    async fn commit(&mut self) -> Result<(), SettingsError> {
        Ok(()) // No-op for our use case
    }

    async fn columns(&mut self) -> Result<Vec<String>, SettingsError> {
        self.inner.columns().await
    }

    async fn fetch_all(
        &mut self,
        fields: &[String],
        filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<Vec<module_settings::state::State>, SettingsError> {
        self.inner.fetch_all(fields, filters).await
    }

    async fn matching_entry_count(
        &mut self,
        filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<usize, SettingsError> {
        self.inner.matching_entry_count(filters).await
    }

    async fn create_entry(
        &mut self,
        entry: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<module_settings::state::State, SettingsError> {
        if !silverpelt::module_config::is_module_enabled(&self.cache, &self.inner.pool, self.inner.guild_id, "lockdown")
            .await
            .map_err(|e| SettingsError::Generic {
                message: format!("Error while checking if module is enabled: {}", e),
                src: "lockdown_create".to_string(),
                typ: "value_error".to_string(),
            })? {
            return Err(SettingsError::Generic {
                message: "Lockdown module is not enabled".to_string(),
                src: "lockdown_create".to_string(),
                typ: "value_error".to_string(),
            });
        }
        
        let Some(splashcore_rs::value::Value::String(typ)) = entry.get("type") else {
            return Err(SettingsError::MissingOrInvalidField {
                field: "type".to_string(),
                src: "lockdown_create_entry".to_string(),
            });
        };

        let Some(splashcore_rs::value::Value::String(reason)) = entry.get("reason") else {
            return Err(SettingsError::MissingOrInvalidField {
                field: "reason".to_string(),
                src: "lockdown_create_entry".to_string(),
            });
        };

        // Get the current lockdown set
        let mut lockdowns = lockdowns::LockdownSet::guild(self.inner.guild_id, &self.inner.pool)
            .await
            .map_err(|e| SettingsError::Generic {
                message: format!("Error while fetching lockdown set: {}", e),
                src: "lockdown_create_entry".to_string(),
                typ: "value_error".to_string(),
            })?;

        // Create the lockdown
        let lockdown_type =
            lockdowns::from_lockdown_mode_string(typ).map_err(|_| SettingsError::Generic {
                message: format!(
                    "Invalid lockdown mode: {}.\n\nTIP: The following lockdown modes are supported: {}", 
                    typ, 
                    {
                        let mut supported_lockdown_modes = String::new();

                        for mode in lockdowns::CREATE_LOCKDOWN_MODES.iter() {
                            let creator = mode.value();
                            supported_lockdown_modes.push_str(&format!("\n- {}", creator.syntax()));
                        }

                        supported_lockdown_modes
                    }
                ),
                src: "lockdown_create_entry".to_string(),
                typ: "value_error".to_string(),
            })?;

        lockdowns
            .easy_apply(lockdown_type, &self.lockdown_data, reason)
            .await
            .map_err(|e| SettingsError::Generic {
                message: format!("Error while applying lockdown: {}", e),
                src: "lockdown_create_entry".to_string(),
                typ: "value_error".to_string(),
            })?;

        let created_lockdown =
            lockdowns
                .lockdowns
                .last()
                .ok_or_else(|| SettingsError::Generic {
                    message: "No lockdowns created".to_string(),
                    src: "lockdown_create_entry".to_string(),
                    typ: "value_error".to_string(),
                })?;

        Ok(module_settings::state::State {
            state: created_lockdown.to_map(),
            bypass_ignore_for: std::collections::HashSet::new(),
        })
    }

    async fn update_matching_entries(
        &mut self,
        _filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
        _entry: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<(), SettingsError> {
        Err(SettingsError::OperationNotSupported {
            operation: OperationType::Update
        })
    }

    async fn delete_matching_entries(
        &mut self,
        filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<(), SettingsError> {
        if !silverpelt::module_config::is_module_enabled(&self.cache, &self.inner.pool, self.inner.guild_id, "lockdown")
        .await
        .map_err(|e| SettingsError::Generic {
            message: format!("Error while checking if module is enabled: {}", e),
            src: "lockdown_create".to_string(),
            typ: "value_error".to_string(),
        })? {
        return Err(SettingsError::Generic {
            message: "Lockdown module is not enabled".to_string(),
            src: "lockdown_create".to_string(),
            typ: "value_error".to_string(),
        });
    }
        
        for (k, _) in filters.iter() {
            if *k != self.inner.setting_primary_key {
                return Err(
                    SettingsError::Generic {
                        message: format!("Invalid filter key: {}. Lockdown deletion only supports the primary key as a filter", k),
                        src: "lockdown_delete_matching_entries".to_string(),
                        typ: "value_error".to_string(),
                    }
                );
            }
        }

        let primary_key = match filters.get(self.inner.setting_primary_key) {
            Some(Value::String(primary_key)) => {
                primary_key
                    .clone()
                    .parse()
                    .map_err(|_| SettingsError::Generic {
                        message: format!("Invalid primary key: {}", primary_key),
                        src: "lockdown_delete_matching_entries".to_string(),
                        typ: "value_error".to_string(),
                    })?
            }
            Some(Value::Uuid(primary_key)) => *primary_key,
            _ => {
                return Err(SettingsError::Generic {
                    message: "Primary key must be a string or UUID".to_string(),
                    src: "lockdown_delete_matching_entries".to_string(),
                    typ: "value_error".to_string(),
                })
            }
        };

        // Get the current lockdown set
        let mut lockdowns = lockdowns::LockdownSet::guild(self.inner.guild_id, &self.inner.pool)
            .await
            .map_err(|e| SettingsError::Generic {
                message: format!("Error while fetching lockdown set: {}", e),
                src: "lockdown_delete_matching_entries".to_string(),
                typ: "value_error".to_string(),
            })?;

        // Remove the lockdown
        lockdowns
            .easy_remove(primary_key, &self.lockdown_data)
            .await
            .map_err(|e| SettingsError::Generic {
                message: format!("Error while removing lockdown: {}", e),
                src: "lockdown_delete_matching_entries".to_string(),
                typ: "value_error".to_string(),
            })?;

        Ok(())
    }
}

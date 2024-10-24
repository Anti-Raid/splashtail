use async_trait::async_trait;
use module_settings::{
    data_stores::{PostgresDataStore, PostgresDataStoreImpl},
    types::{
        settings_wrap, Column, ColumnSuggestion, ColumnType, ConfigOption,
        CreateDataStore, DataStore, InnerColumnType, InnerColumnTypeStringKind, OperationSpecific,
        OperationType, SettingsData, SettingsError, NoOpValidator, NoOpPostAction, PostAction, HookContext
    },
    state::State,
};
use splashcore_rs::value::Value;
use std::sync::LazyLock;

pub static LOCKDOWN_SETTINGS: LazyLock<ConfigOption> = LazyLock::new(|| {
    ConfigOption {
        id: "lockdown_guilds",
        name: "Lockdown Settings",
        description: "Setup standard lockdown settings for a server",
        table: "lockdown__guilds",
        common_filters: indexmap::indexmap! {},
        default_common_filters: indexmap::indexmap! {
            "guild_id" => "{__guild_id}"
        },
        primary_key: "guild_id",
        max_entries: Some(1),
        max_return: 2,
        data_store: settings_wrap(PostgresDataStore {}),
        columns: settings_wrap(vec![
            module_settings::common_columns::guild_id(
                "guild_id",
                "Guild ID",
                "Guild ID of the server in question",
            ),
            Column {
                id: "member_roles",
                name: "Member Roles",
                description: "Which roles to use as member roles for the purpose of lockdown. These roles will be explicitly modified during lockdown",
                column_type: ColumnType::new_array(InnerColumnType::String {
                    kind: InnerColumnTypeStringKind::Role,
                    min_length: None,
                    max_length: None,
                    allowed_values: vec![],
                }),
                nullable: false,
                default: None,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
            },
            Column {
                id: "require_correct_layout",
                name: "Require Correct Layout",
                description: "Whether or not a lockdown can proceed even without correct critical role permissions. May lead to partial lockdowns if disabled",
                column_type: ColumnType::new_scalar(InnerColumnType::Boolean {}),
                nullable: false,
                default: None,
                unique: true,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![OperationType::Create],
                secret: false,
            },
            module_settings::common_columns::created_at(),
            module_settings::common_columns::created_by(),
            module_settings::common_columns::last_updated_at(),
            module_settings::common_columns::last_updated_by(),
        ]),
        title_template: "Lockdown Settings",
        operations: indexmap::indexmap! {
            OperationType::View => OperationSpecific {
                columns_to_set: indexmap::indexmap! {},
            },
            OperationType::Create => OperationSpecific {
                columns_to_set: indexmap::indexmap! {
                    "created_at" => "{__now}",
                    "created_by" => "{__author}",
                    "last_updated_at" => "{__now}",
                    "last_updated_by" => "{__author}",
                },
            },
            OperationType::Update => OperationSpecific {
                columns_to_set: indexmap::indexmap! {
                    "last_updated_at" => "{__now}",
                    "last_updated_by" => "{__author}",
                },
            },
            OperationType::Delete => OperationSpecific {
                columns_to_set: indexmap::indexmap! {},
            },
        },
        validator: settings_wrap(NoOpValidator {}),
        post_action: settings_wrap(NoOpPostAction {}),
    }
});

/// Post actions for Lockdown Settings to clear cache
pub struct LockdownSettingsPostActions;

#[async_trait::async_trait]
impl PostAction for LockdownSettingsPostActions {
    async fn post_action<'a>(
        &self,
        ctx: HookContext<'a>,
        _state: &'a mut State,
    ) -> Result<(), SettingsError> {
        if ctx.operation_type == OperationType::View {
            return Ok(());
        }
        super::cache::GUILD_LOCKDOWN_SETTINGS
            .invalidate(&ctx.guild_id)
            .await;

        Ok(())
    }
}


pub static LOCKDOWNS: LazyLock<ConfigOption> = LazyLock::new(|| ConfigOption {
    id: "lockdowns",
    name: "Lockdowns",
    description: "Lockdowns",
    table: "lockdown__guild_lockdowns",
    common_filters: indexmap::indexmap! {},
    default_common_filters: indexmap::indexmap! {
        "guild_id" => "{__guild_id}"
    },
    primary_key: "id",
    max_entries: Some(1),
    max_return: 5,
    data_store: settings_wrap(LockdownDataStore {}), // We use a custom data store here to make lockdown handling easier+more separate from settings
    columns: settings_wrap(vec![
        Column {
            id: "id",
            name: "ID",
            description: "The ID of the lockdown",
            column_type: ColumnType::new_scalar(InnerColumnType::Uuid {}),
            nullable: false,
            default: None,
            unique: false,
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
            id: "type",
            name: "Type",
            description: "The type of the lockdown.",
            column_type: ColumnType::new_scalar(InnerColumnType::String {
                kind: InnerColumnTypeStringKind::Normal,
                min_length: Some(1),
                max_length: Some(256),
                allowed_values: vec![],
            }),
            nullable: false,
            default: None,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
        },
        Column {
            id: "data",
            name: "Data",
            description: "The data stored of the lockdown.",
            column_type: ColumnType::new_scalar(InnerColumnType::Json { max_bytes: None }),
            nullable: false,
            default: None,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![OperationType::Create],
            secret: false,
        },
        Column {
            id: "reason",
            name: "Reason",
            description: "The reason for starting the lockdown.",
            column_type: ColumnType::new_scalar(InnerColumnType::String {
                kind: InnerColumnTypeStringKind::Normal,
                min_length: Some(1),
                max_length: Some(256),
                allowed_values: vec![],
            }),
            nullable: false,
            default: None,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
        },
        module_settings::common_columns::created_at(),
    ]),
    title_template: "Reason: {reason}",
    operations: indexmap::indexmap! {
        OperationType::View => OperationSpecific {
            columns_to_set: indexmap::indexmap! {},
        },
        OperationType::Create => OperationSpecific {
            columns_to_set: indexmap::indexmap! {
                "created_at" => "{__now}",
            },
        },
        OperationType::Delete => OperationSpecific {
            columns_to_set: indexmap::indexmap! {},
        }
    },
    validator: settings_wrap(NoOpValidator {}),
    post_action: settings_wrap(NoOpPostAction {}),
});

/// A custom data store is needed to handle the specific requirements of the lockdown module
pub struct LockdownDataStore {}

#[async_trait]
impl CreateDataStore for LockdownDataStore {
    async fn create(
        &self,
        setting: &ConfigOption,
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
            lockdown_data: super::core::LockdownData::from_settings_data(data),
        }))
    }
}

pub struct LockdownDataStoreImpl {
    inner: PostgresDataStoreImpl,
    cache: std::sync::Arc<silverpelt::cache::SilverpeltCache>,
    lockdown_data: super::core::LockdownData,
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
        let mut lockdowns = super::core::LockdownSet::guild(self.inner.guild_id, &self.inner.pool)
            .await
            .map_err(|e| SettingsError::Generic {
                message: format!("Error while fetching lockdown set: {}", e),
                src: "lockdown_create_entry".to_string(),
                typ: "value_error".to_string(),
            })?;

        // Create the lockdown
        let lockdown_type =
            super::core::from_lockdown_mode_string(typ).map_err(|_| SettingsError::Generic {
                message: format!(
                    "Invalid lockdown mode: {}.\n\nTIP: The following lockdown modes are supported: {}", 
                    typ, 
                    {
                        let mut supported_lockdown_modes = String::new();

                        for mode in super::core::CREATE_LOCKDOWN_MODES.iter() {
                            let creator = mode.value();
                            supported_lockdown_modes.push_str(&format!("\n- {}", creator.syntax()));
                        }

                        supported_lockdown_modes
                    }
                ),
                src: "lockdown_create_entry".to_string(),
                typ: "value_error".to_string(),
            })?;

        let mut pg = sandwich_driver::guild(
            &self.lockdown_data.cache_http,
            &self.lockdown_data.reqwest,
            self.inner.guild_id,
        )
        .await
        .map_err(|e| SettingsError::Generic {
            message: format!("Error while creating proxy guild: {}", e),
            src: "lockdown_create_entry".to_string(),
            typ: "value_error".to_string(),
        })?;

        let mut pgc = sandwich_driver::guild_channels(
            &self.lockdown_data.cache_http,
            &self.lockdown_data.reqwest,
            self.inner.guild_id,
        )
        .await
        .map_err(|e| SettingsError::Generic {
            message: format!("Error while creating proxy guild channels: {}", e),
            src: "lockdown_create_entry".to_string(),
            typ: "value_error".to_string(),
        })?;

        lockdowns
            .apply(lockdown_type, &self.lockdown_data, reason, &mut pg, &mut pgc)
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
        let mut lockdowns = super::core::LockdownSet::guild(self.inner.guild_id, &self.inner.pool)
            .await
            .map_err(|e| SettingsError::Generic {
                message: format!("Error while fetching lockdown set: {}", e),
                src: "lockdown_delete_matching_entries".to_string(),
                typ: "value_error".to_string(),
            })?;

        // Find the index of the lockdown element with the given primary key
        let index = lockdowns
            .lockdowns
            .iter()
            .position(|l| l.id == primary_key)
            .ok_or_else(|| SettingsError::RowDoesNotExist {
                column_id: self.inner.setting_primary_key.to_string(),
            })?;

        let mut pg = sandwich_driver::guild(
            &self.lockdown_data.cache_http,
            &self.lockdown_data.reqwest,
            self.inner.guild_id,
        )
        .await
        .map_err(|e| SettingsError::Generic {
            message: format!("Error while removing lockdown: {}", e),
            src: "lockdown_delete_matching_entries".to_string(),
            typ: "value_error".to_string(),
        })?;

        let mut pgc = sandwich_driver::guild_channels(
            &self.lockdown_data.cache_http,
            &self.lockdown_data.reqwest,
            self.inner.guild_id,
        )
        .await
        .map_err(|e| SettingsError::Generic {
            message: format!("Error while removing lockdown: {}", e),
            src: "lockdown_delete_matching_entries".to_string(),
            typ: "value_error".to_string(),
        })?;

        // Remove the lockdown
        lockdowns
            .remove(index, &self.lockdown_data, &mut pg, &mut pgc)
            .await
            .map_err(|e| SettingsError::Generic {
                message: format!("Error while removing lockdown: {}", e),
                src: "lockdown_delete_matching_entries".to_string(),
                typ: "value_error".to_string(),
            })?;

        Ok(())
    }
}

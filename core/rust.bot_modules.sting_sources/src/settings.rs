use async_trait::async_trait;
use module_settings::types::{OperationType, SettingsError};
use silverpelt::sting_sources::{StingCountFilters, StingFetchFilters, StingSourceData};
use std::sync::Arc;

// Data source for stings
pub struct StingsDataStore {}

#[async_trait]
impl module_settings::types::CreateDataStore for StingsDataStore {
    async fn create(
        &self,
        setting: &module_settings::types::ConfigOption,
        guild_id: serenity::all::GuildId,
        author: serenity::all::UserId,
        data: &module_settings::types::SettingsData,
        common_filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<Box<dyn module_settings::types::DataStore>, module_settings::types::SettingsError>
    {
        Ok(Box::new(StingsDataStoreImpl {
            setting_table: setting.table,
            setting_primary_key: setting.primary_key,
            author,
            guild_id,
            columns: setting.columns.clone(),
            valid_columns: setting.columns.iter().map(|c| c.id.to_string()).collect(),
            pool: data.pool.clone(),
            reqwest: data.reqwest.clone(),
            cache_http: data.cache_http.clone(),
            silverpelt_cache: silverpelt::data::Data::silverpelt_cache(data),
            common_filters,
        }))
    }
}

pub struct StingsDataStoreImpl {
    // Args needed for queries
    pub pool: sqlx::PgPool,
    pub reqwest: reqwest::Client,
    pub cache_http: botox::cache::CacheHttpImpl,
    pub setting_table: &'static str,
    pub setting_primary_key: &'static str,
    pub author: serenity::all::UserId,
    pub guild_id: serenity::all::GuildId,
    pub columns: Arc<Vec<module_settings::types::Column>>,
    pub valid_columns: std::collections::HashSet<String>, // Derived from columns
    pub common_filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    pub silverpelt_cache: std::sync::Arc<silverpelt::cache::SilverpeltCache>,
}

#[async_trait]
impl module_settings::types::DataStore for StingsDataStoreImpl {
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
        Ok(self.columns.iter().map(|c| c.id.to_string()).collect())
    }

    async fn fetch_all(
        &mut self,
        fields: &[String],
        filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<Vec<module_settings::state::State>, SettingsError> {
        let mut states = Vec::new();

        for refs in self.silverpelt_cache.module_cache.iter() {
            let module = refs.value();
            for source in module.sting_sources().iter() {
                let entries = source
                    .fetch(
                        &StingSourceData {
                            pool: self.pool.clone(),
                            reqwest: self.reqwest.clone(),
                            cache_http: self.cache_http.clone(),
                            silverpelt_cache: self.silverpelt_cache.clone(),
                        },
                        StingFetchFilters::from_map(&filters).map_err(|e| {
                            SettingsError::Generic {
                                message: format!("Failed to parse filters: {}", e),
                                src: "fetch_all".to_string(),
                                typ: "internal".to_string(),
                            }
                        })?,
                    )
                    .await
                    .map_err(|e| SettingsError::Generic {
                        message: format!("Failed to fetch stings: {}", e),
                        src: "fetch_all".to_string(),
                        typ: "internal".to_string(),
                    })?;

                for entry in entries {
                    let mut state = module_settings::state::State::default();

                    let serde_json::Value::Object(obj) = serde_json::to_value(entry.entry)
                        .map_err(|e| SettingsError::Generic {
                            message: format!("Failed to serialize sting entry: {}", e),
                            src: "fetch_all".to_string(),
                            typ: "internal".to_string(),
                        })?
                    else {
                        return Err(SettingsError::Generic {
                            message: "Failed to serialize sting entry".to_string(),
                            src: "fetch_all".to_string(),
                            typ: "internal".to_string(),
                        });
                    };

                    for (k, v) in obj {
                        if !fields.is_empty() && !fields.contains(&k) {
                            continue;
                        }

                        state
                            .state
                            .insert(k, splashcore_rs::value::Value::from_json(&v));
                    }

                    states.push(state);
                }
            }
        }

        Ok(states)
    }

    async fn matching_entry_count(
        &mut self,
        filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<usize, SettingsError> {
        let mut count = 0;

        for refs in self.silverpelt_cache.module_cache.iter() {
            let module = refs.value();
            for source in module.sting_sources().iter() {
                count += source
                    .count(
                        &StingSourceData {
                            pool: self.pool.clone(),
                            reqwest: self.reqwest.clone(),
                            cache_http: self.cache_http.clone(),
                            silverpelt_cache: self.silverpelt_cache.clone(),
                        },
                        StingCountFilters::from_map(&filters).map_err(|e| {
                            SettingsError::Generic {
                                message: format!("Failed to parse filters: {}", e),
                                src: "matching_entry_count".to_string(),
                                typ: "internal".to_string(),
                            }
                        })?,
                    )
                    .await
                    .map_err(|e| SettingsError::Generic {
                        message: format!("Failed to count stings: {}", e),
                        src: "matching_entry_count".to_string(),
                        typ: "internal".to_string(),
                    })?;
            }
        }

        Ok(count)
    }

    async fn create_entry(
        &mut self,
        _entry: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<module_settings::state::State, SettingsError> {
        Err(SettingsError::OperationNotSupported {
            operation: OperationType::Create,
        })
    }

    async fn update_matching_entries(
        &mut self,
        _filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
        _entry: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<(), SettingsError> {
        Err(SettingsError::OperationNotSupported {
            operation: OperationType::Update,
        })
    }

    async fn delete_matching_entries(
        &mut self,
        _filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<(), SettingsError> {
        Err(SettingsError::OperationNotSupported {
            operation: OperationType::Delete,
        })
    }
}

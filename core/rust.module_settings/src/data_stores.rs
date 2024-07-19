use super::state::State;
use super::types::{
    ColumnType, ConfigOption, CreateDataStore, DataStore, InnerColumnType, SettingsError,
};
use async_trait::async_trait;
use splashcore_rs::value::Value;

pub struct PostgresDataStore {}

#[async_trait]
impl CreateDataStore for PostgresDataStore {
    async fn create(
        &self,
        _setting: &ConfigOption,
        _cache_http: &botox::cache::CacheHttpImpl,
        _reqwest_client: &reqwest::Client,
        _pool: &sqlx::PgPool,
        _guild_id: serenity::all::GuildId,
        _author: serenity::all::UserId,
        _permodule_executor: &dyn base_data::permodule::PermoduleFunctionExecutor,
    ) -> Result<Box<dyn DataStore>, SettingsError> {
        Ok(Box::new(PostgresDataStoreImpl { tx: None }))
    }
}

struct PostgresDataStoreImpl {
    tx: Option<sqlx::Transaction<'static, sqlx::Postgres>>,
}

impl PostgresDataStoreImpl {
    /// Binds a value to a query
    ///
    /// Note that Maps are binded as JSONs
    ///
    /// `default_column_type` - The (default) column type to use if the value is None. This should be the column_type
    fn _query_bind_value<'a>(
        query: sqlx::query::Query<'a, sqlx::Postgres, sqlx::postgres::PgArguments>,
        value: Value,
        default_column_type: &ColumnType,
        state: &State,
    ) -> sqlx::query::Query<'a, sqlx::Postgres, sqlx::postgres::PgArguments> {
        match value {
            Value::Uuid(value) => query.bind(value),
            Value::String(value) => query.bind(value),
            Value::Timestamp(value) => query.bind(value),
            Value::TimestampTz(value) => query.bind(value),
            Value::Interval(value) => query.bind(value),
            Value::Integer(value) => query.bind(value),
            Value::Float(value) => query.bind(value),
            Value::Boolean(value) => query.bind(value),
            Value::List(values) => {
                // Get the type of the first element
                let first = values.first();

                if let Some(first) = first {
                    // This is hacky and long but sqlx doesn't support binding lists
                    //
                    // Loop over all values to make a Vec<T> then bind that
                    match first {
                        Value::Uuid(_) => {
                            let mut vec = Vec::new();

                            for value in values {
                                if let Value::Uuid(value) = value {
                                    vec.push(value);
                                }
                            }

                            query.bind(vec)
                        }
                        Value::String(_) => {
                            let mut vec = Vec::new();

                            for value in values {
                                if let Value::String(value) = value {
                                    vec.push(value);
                                }
                            }

                            query.bind(vec)
                        }
                        Value::Timestamp(_) => {
                            let mut vec = Vec::new();

                            for value in values {
                                if let Value::Timestamp(value) = value {
                                    vec.push(value);
                                }
                            }

                            query.bind(vec)
                        }
                        Value::TimestampTz(_) => {
                            let mut vec = Vec::new();

                            for value in values {
                                if let Value::TimestampTz(value) = value {
                                    vec.push(value);
                                }
                            }

                            query.bind(vec)
                        }
                        Value::Interval(_) => {
                            let mut vec = Vec::new();

                            for value in values {
                                if let Value::Interval(value) = value {
                                    vec.push(value);
                                }
                            }

                            query.bind(vec)
                        }
                        Value::Integer(_) => {
                            let mut vec = Vec::new();

                            for value in values {
                                if let Value::Integer(value) = value {
                                    vec.push(value);
                                }
                            }

                            query.bind(vec)
                        }
                        Value::Float(_) => {
                            let mut vec = Vec::new();

                            for value in values {
                                if let Value::Float(value) = value {
                                    vec.push(value);
                                }
                            }

                            query.bind(vec)
                        }
                        Value::Boolean(_) => {
                            let mut vec = Vec::new();

                            for value in values {
                                if let Value::Boolean(value) = value {
                                    vec.push(value);
                                }
                            }

                            query.bind(vec)
                        }
                        // In all other cases (list/map)
                        Value::Map(_) => {
                            let mut vec = Vec::new();

                            for value in values {
                                vec.push(value.to_json());
                            }

                            query.bind(vec)
                        }
                        Value::List(_) => {
                            let mut vec = Vec::new();

                            for value in values {
                                vec.push(value.to_json());
                            }

                            query.bind(vec)
                        }
                        Value::None => {
                            let vec: Vec<String> = Vec::new();
                            query.bind(vec)
                        }
                    }
                } else {
                    let vec: Vec<String> = Vec::new();
                    query.bind(vec)
                }
            }
            Value::Map(_) => query.bind(value.to_json()),
            Value::None => match default_column_type {
                ColumnType::Scalar {
                    column_type: column_type_hint,
                } => match column_type_hint {
                    InnerColumnType::Uuid {} => query.bind(None::<sqlx::types::uuid::Uuid>),
                    InnerColumnType::String { .. } => query.bind(None::<String>),
                    InnerColumnType::Timestamp {} => query.bind(None::<chrono::NaiveDateTime>),
                    InnerColumnType::TimestampTz {} => {
                        query.bind(None::<chrono::DateTime<chrono::Utc>>)
                    }
                    InnerColumnType::Interval {} => query.bind(None::<chrono::Duration>),
                    InnerColumnType::Integer {} => query.bind(None::<i64>),
                    InnerColumnType::Float {} => query.bind(None::<f64>),
                    InnerColumnType::BitFlag { .. } => query.bind(None::<i64>),
                    InnerColumnType::Boolean {} => query.bind(None::<bool>),
                    InnerColumnType::Json {} => query.bind(None::<serde_json::Value>),
                },
                ColumnType::Array {
                    inner: column_type_hint,
                } => match column_type_hint {
                    InnerColumnType::Uuid {} => query.bind(None::<Vec<sqlx::types::uuid::Uuid>>),
                    InnerColumnType::String { .. } => query.bind(None::<Vec<String>>),
                    InnerColumnType::Timestamp {} => query.bind(None::<Vec<chrono::NaiveDateTime>>),
                    InnerColumnType::TimestampTz {} => {
                        query.bind(None::<Vec<chrono::DateTime<chrono::Utc>>>)
                    }
                    InnerColumnType::Interval {} => query.bind(None::<Vec<chrono::Duration>>),
                    InnerColumnType::Integer {} => query.bind(None::<Vec<i64>>),
                    InnerColumnType::Float {} => query.bind(None::<Vec<f64>>),
                    InnerColumnType::BitFlag { .. } => query.bind(None::<Vec<i64>>),
                    InnerColumnType::Boolean {} => query.bind(None::<Vec<bool>>),
                    InnerColumnType::Json {} => query.bind(None::<Vec<serde_json::Value>>),
                },
                ColumnType::Dynamic { clauses } => {
                    for clause in clauses {
                        let _value = state.template_to_string(clause.field);

                        if _value == clause.value {
                            return Self::_query_bind_value(
                                query,
                                value,
                                &clause.column_type,
                                state,
                            );
                        }
                    }

                    query.bind(None::<String>) // Default to string
                }
            },
        }
    }
}

#[async_trait]
impl DataStore for PostgresDataStoreImpl {
    async fn start_transaction(
        &mut self,
        _setting: &ConfigOption,
        _cache_http: &botox::cache::CacheHttpImpl,
        _reqwest_client: &reqwest::Client,
        pool: &sqlx::PgPool,
        _guild_id: serenity::all::GuildId,
        _author: serenity::all::UserId,
        _permodule_executor: &dyn base_data::permodule::PermoduleFunctionExecutor,
    ) -> Result<(), SettingsError> {
        let tx: sqlx::Transaction<'_, sqlx::Postgres> =
            pool.begin().await.map_err(|e| SettingsError::Generic {
                message: e.to_string(),
                src: "PostgresDataStore::start_transaction [pool.begin]".to_string(),
                typ: "internal".to_string(),
            })?;

        self.tx = Some(tx);

        Ok(())
    }

    async fn commit(
        &mut self,
        _setting: &ConfigOption,
        _cache_http: &botox::cache::CacheHttpImpl,
        _reqwest_client: &reqwest::Client,
        _pool: &sqlx::PgPool,
        _guild_id: serenity::all::GuildId,
        _author: serenity::all::UserId,
        _permodule_executor: &dyn base_data::permodule::PermoduleFunctionExecutor,
    ) -> Result<(), SettingsError> {
        if let Some(tx) = self.tx.take() {
            tx.commit().await.map_err(|e| SettingsError::Generic {
                message: e.to_string(),
                src: "PostgresDataStore::commit [tx.commit]".to_string(),
                typ: "internal".to_string(),
            })?;
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn fetch_all(
        &self,
        setting: &ConfigOption,
        _cache_http: &botox::cache::CacheHttpImpl,
        _reqwest_client: &reqwest::Client,
        pool: &sqlx::PgPool,
        guild_id: serenity::all::GuildId,
        author: serenity::all::UserId,
        _permodule_executor: &dyn base_data::permodule::PermoduleFunctionExecutor,
        fields: &[String],
        filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<Vec<super::state::State>, SettingsError> {
        let mut filters_str = String::new();

        for (i, (key, v)) in filters.iter().enumerate() {
            // $1 is guild_id, $2 is the first filter
            if matches!(v, Value::None) {
                filters_str.push_str(format!(" AND {} IS NULL", key).as_str());
            } else {
                filters_str.push_str(format!(" AND {} = ${}", key, i + 2).as_str());
            }
        }

        let sql_stmt = format!(
            "SELECT {} FROM {} WHERE {} = $1 {}",
            fields.join(", "),
            setting.table,
            setting.guild_id,
            filters_str
        );

        let mut query = sqlx::query(sql_stmt.as_str()).bind(guild_id.to_string());

        if !filters.is_empty() {
            let filter_state = State::new_with_special_variables(author, guild_id); // TODO: Avoid needing filter state here
            for (field_name, value) in filters.iter() {
                if matches!(value, Value::None) {
                    continue;
                }

                let column = setting.columns.iter().find(|c| c.id == field_name).ok_or(
                    SettingsError::Generic {
                        message: format!("Column {} not found", field_name),
                        src: "PostgresDataStore [bind_filters_for_update]".to_string(),
                        typ: "internal".to_string(),
                    },
                )?;

                query = Self::_query_bind_value(
                    query,
                    value.clone(),
                    &column.column_type,
                    &filter_state,
                );
            }
        }

        let rows = query
            .fetch_all(pool)
            .await
            .map_err(|e| SettingsError::Generic {
                message: e.to_string(),
                src: "settings_view [query fetch_all]".to_string(),
                typ: "internal".to_string(),
            })?;

        if rows.is_empty() {
            return Ok(Vec::new());
        }

        let mut values: Vec<State> = Vec::new();

        for row in rows {
            let mut state = State::new_with_special_variables(author, guild_id);

            for (i, col) in fields.iter().enumerate() {
                let val = Value::from_sqlx(&row, i).map_err(|e| SettingsError::Generic {
                    message: e.to_string(),
                    src: "PostgresDataStore::fetch_all [Value::from_sqlx]".to_string(),
                    typ: "internal".to_string(),
                })?;

                state.state.insert(col.to_string(), val);
            }

            values.push(state);
        }

        Ok(values)
    }

    #[allow(clippy::too_many_arguments)]
    async fn matching_entry_count(
        &self,
        setting: &ConfigOption,
        _cache_http: &botox::cache::CacheHttpImpl,
        _reqwest_client: &reqwest::Client,
        pool: &sqlx::PgPool,
        guild_id: serenity::all::GuildId,
        author: serenity::all::UserId,
        _permodule_executor: &dyn base_data::permodule::PermoduleFunctionExecutor,
        filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<usize, SettingsError> {
        let mut filters_str = String::new();

        for (i, (key, v)) in filters.iter().enumerate() {
            if matches!(v, Value::None) {
                filters_str.push_str(format!(" AND {} IS NULL", key).as_str());
            } else {
                filters_str.push_str(format!(" AND {} = ${}", key, i + 2).as_str());
            }
        }

        let sql_stmt = format!(
            "SELECT COUNT(*) FROM {} WHERE {} = $1 {}",
            setting.table, setting.guild_id, filters_str
        );

        let mut query = sqlx::query(sql_stmt.as_str()).bind(guild_id.to_string());

        if !filters.is_empty() {
            let filter_state = State::new_with_special_variables(author, guild_id); // TODO: Avoid needing filter state here
            for (field_name, value) in filters.iter() {
                if matches!(value, Value::None) {
                    continue;
                }

                let column = setting.columns.iter().find(|c| c.id == field_name).ok_or(
                    SettingsError::Generic {
                        message: format!("Column {} not found", field_name),
                        src: "settings_view [fetch_all]".to_string(),
                        typ: "internal".to_string(),
                    },
                )?;

                query = Self::_query_bind_value(
                    query,
                    value.clone(),
                    &column.column_type,
                    &filter_state,
                );
            }
        }

        let row = query
            .fetch_one(pool)
            .await
            .map_err(|e| SettingsError::Generic {
                message: e.to_string(),
                src: "settings_view [query fetch_one]".to_string(),
                typ: "internal".to_string(),
            })?;

        let count = Value::from_sqlx(&row, 0).map_err(|e| SettingsError::Generic {
            message: e.to_string(),
            src: "PostgresDataStore::matching_entry_count [Value::from_sqlx]".to_string(),
            typ: "internal".to_string(),
        })?;

        if let Value::Integer(count) = count {
            Ok(count as usize)
        } else {
            Err(SettingsError::Generic {
                message: "Count is not an integer".to_string(),
                src: "settings_view [matching_entry_count]".to_string(),
                typ: "internal".to_string(),
            })
        }
    }

    /// Creates a new entry given a set of columns to set returning the newly created entry
    #[allow(clippy::too_many_arguments)]
    async fn create_entry(
        &self,
        setting: &ConfigOption,
        _cache_http: &botox::cache::CacheHttpImpl,
        _reqwest_client: &reqwest::Client,
        pool: &sqlx::PgPool,
        guild_id: serenity::all::GuildId,
        _author: serenity::all::UserId,
        _permodule_executor: &dyn base_data::permodule::PermoduleFunctionExecutor,
        entry: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<super::state::State, SettingsError> {
        // Create the row
        // First create the $N's from the cols starting with 2 as 1 is the guild_id
        let mut n_params = "".to_string();
        let mut col_params = "".to_string();
        for (i, (col, _)) in entry.iter().enumerate() {
            n_params.push_str(&format!("${}", i + 2));
            col_params.push_str(col);

            n_params.push(',');
            col_params.push(',');
        }

        // Remove the trailing comma
        n_params.pop();
        col_params.pop();

        // Execute the SQL statement
        let sql_stmt = format!(
            "INSERT INTO {} ({},{}) VALUES ($1,{}) RETURNING {}",
            setting.table, setting.guild_id, col_params, n_params, setting.primary_key
        );

        let mut query = sqlx::query(sql_stmt.as_str());

        // Bind the sql query arguments
        query = query.bind(guild_id.to_string());

        let mut state = State::from_indexmap(entry);

        for (col, value) in state.state.iter() {
            // Get column type from schema for db query hinting
            let Some(column) = setting.columns.iter().find(|c| c.id == col) else {
                return Err(SettingsError::Generic {
                    message: format!("Column `{}` not found in schema", col),
                    src: "PostgresDataStore::create_entry [column_type_let_else]".to_string(),
                    typ: "internal".to_string(),
                });
            };

            query = Self::_query_bind_value(query, value.clone(), &column.column_type, &state);
        }

        // Execute the query
        let pkey_row = query
            .fetch_one(pool)
            .await
            .map_err(|e| SettingsError::Generic {
                message: e.to_string(),
                src: "settings_create [query execute]".to_string(),
                typ: "internal".to_string(),
            })?;

        // Save pkey to state
        state.state.insert(
            setting.primary_key.to_string(),
            Value::from_sqlx(&pkey_row, 0).map_err(|e| SettingsError::Generic {
                message: e.to_string(),
                src: "settings_create [Value::from_sqlx]".to_string(),
                typ: "internal".to_string(),
            })?,
        );

        Ok(state)
    }

    /// Updates an entry given a set of columns to set and a set of filters returning the updated entry
    ///
    /// Note that only the fields to be updated should be passed to this function
    #[allow(clippy::too_many_arguments)]
    async fn update_matching_entries(
        &self,
        setting: &ConfigOption,
        _cache_http: &botox::cache::CacheHttpImpl,
        _reqwest_client: &reqwest::Client,
        pool: &sqlx::PgPool,
        guild_id: serenity::all::GuildId,
        _author: serenity::all::UserId,
        _permodule_executor: &dyn base_data::permodule::PermoduleFunctionExecutor,
        filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
        entry: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<(), SettingsError> {
        let mut col_params = "".to_string();
        for (i, (col, _)) in entry.iter().enumerate() {
            // $1 is guild id, $2 is first col param
            col_params.push_str(&format!("{}=${},", col, i + 2));
        }

        // Remove the trailing comma
        col_params.pop();

        // Make the filter string
        let mut filters_str = String::new();

        for (i, (key, _)) in filters.iter().enumerate() {
            // $1 is guild_id, $2-$N are the filters, $N+1-$M are the columns to set
            filters_str.push_str(format!(" AND {} = ${}", key, (i + 2) + entry.len()).as_str());
        }

        // Execute the SQL statement
        let sql_stmt = format!(
            "UPDATE {} SET {} WHERE {} = $1 {}",
            setting.table, col_params, setting.guild_id, filters_str
        );

        let mut query = sqlx::query(sql_stmt.as_str());

        // Bind the sql query arguments
        query = query.bind(guild_id.to_string());

        let entry_state = State::from_indexmap(entry);

        // Add in entry values first
        for (col, value) in entry_state.state.iter() {
            // Get column type from schema for db query hinting
            let Some(column) = setting.columns.iter().find(|c| c.id == col) else {
                return Err(SettingsError::Generic {
                    message: format!("Column `{}` not found in schema", col),
                    src: "PostgresDataStore [column_type_let_else_for_update]".to_string(),
                    typ: "internal".to_string(),
                });
            };

            query =
                Self::_query_bind_value(query, value.clone(), &column.column_type, &entry_state);
        }

        // Add in filter values
        for (field_name, value) in filters.iter() {
            let column = setting.columns.iter().find(|c| c.id == field_name).ok_or(
                SettingsError::Generic {
                    message: format!("Column {} not found", field_name),
                    src: "PostgresDataStore [bind_filters_for_update]".to_string(),
                    typ: "internal".to_string(),
                },
            )?;

            query =
                Self::_query_bind_value(query, value.clone(), &column.column_type, &entry_state);
        }

        // Execute the query
        query
            .execute(pool)
            .await
            .map_err(|e| SettingsError::Generic {
                message: e.to_string(),
                src: "settings_update [query execute]".to_string(),
                typ: "internal".to_string(),
            })?;

        // Return the updated entry

        Ok(())
    }

    /// Deletes entries given a set of filters
    ///
    /// Returns all deleted rows
    #[allow(clippy::too_many_arguments)]
    async fn delete_matching_entries(
        &self,
        setting: &ConfigOption,
        _cache_http: &botox::cache::CacheHttpImpl,
        _reqwest_client: &reqwest::Client,
        pool: &sqlx::PgPool,
        guild_id: serenity::all::GuildId,
        author: serenity::all::UserId,
        _permodule_executor: &dyn base_data::permodule::PermoduleFunctionExecutor,
        filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<(), SettingsError> {
        let mut filters_str = String::new();

        for (i, (key, _)) in filters.iter().enumerate() {
            // $1 is guild_id, $2 is the first filter
            filters_str.push_str(format!(" AND {} = ${}", key, i + 2).as_str());
        }

        let sql_stmt = format!(
            "DELETE FROM {} WHERE {} = $1 {}",
            setting.table, setting.guild_id, filters_str
        );

        let mut query = sqlx::query(sql_stmt.as_str());

        query = query.bind(guild_id.to_string());

        if !filters.is_empty() {
            let filter_state = State::new_with_special_variables(author, guild_id); // TODO: Avoid needing filter state here
            for (field_name, value) in filters.iter() {
                let column = setting.columns.iter().find(|c| c.id == field_name).ok_or(
                    SettingsError::Generic {
                        message: format!("Column {} not found", field_name),
                        src: "PostgresDataStore [bind_filters_for_update]".to_string(),
                        typ: "internal".to_string(),
                    },
                )?;

                query = Self::_query_bind_value(
                    query,
                    value.clone(),
                    &column.column_type,
                    &filter_state,
                );
            }
        }

        let res = query
            .execute(pool)
            .await
            .map_err(|e| SettingsError::Generic {
                message: e.to_string(),
                src: "PostgresDataStore::delete_matching_entries [query_execute]".to_string(),
                typ: "internal".to_string(),
            })?;

        if res.rows_affected() == 0 {
            return Err(SettingsError::RowDoesNotExist {
                column_id: setting.primary_key.to_string(),
            });
        }

        Ok(())
    }
}

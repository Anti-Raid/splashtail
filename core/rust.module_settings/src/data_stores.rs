use super::state::State;
use super::types::{
    Column, ColumnType, ConfigOption, CreateDataStore, DataStore, InnerColumnType, SettingsData,
    SettingsError,
};
use async_trait::async_trait;
use splashcore_rs::{utils::sql_utils, value::Value};
use sqlx::{Execute, Row};
use std::sync::Arc;

/// Simple macro to combine two indexmaps into one
macro_rules! combine_indexmaps {
    ($map1:expr, $map2:expr) => {{
        let mut map = $map1;
        map.extend($map2);
        map
    }};
}

pub struct PostgresDataStore {}

impl PostgresDataStore {
    /// Creates a new PostgresDataStoreImpl. This is exposed as it is useful for making wrapper data stores
    pub async fn create_impl(
        &self,
        setting: &ConfigOption,
        guild_id: serenity::all::GuildId,
        author: serenity::all::UserId,
        data: &SettingsData,
        common_filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<PostgresDataStoreImpl, SettingsError> {
        Ok(PostgresDataStoreImpl {
            tx: None,
            setting_table: setting.table,
            setting_primary_key: setting.primary_key,
            author,
            guild_id,
            columns: setting.columns.clone(),
            valid_columns: setting.columns.iter().map(|c| c.id.to_string()).collect(),
            pool: data.pool.clone(),
            common_filters,
        })
    }
}

#[async_trait]
impl CreateDataStore for PostgresDataStore {
    async fn create(
        &self,
        setting: &ConfigOption,
        guild_id: serenity::all::GuildId,
        author: serenity::all::UserId,
        data: &SettingsData,
        common_filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<Box<dyn DataStore>, SettingsError> {
        Ok(Box::new(
            self.create_impl(setting, guild_id, author, data, common_filters)
                .await?,
        ))
    }
}

pub struct PostgresDataStoreImpl {
    // Args needed for queries
    pub pool: sqlx::PgPool,
    pub setting_table: &'static str,
    pub setting_primary_key: &'static str,
    pub author: serenity::all::UserId,
    pub guild_id: serenity::all::GuildId,
    pub columns: Arc<Vec<Column>>,
    pub valid_columns: std::collections::HashSet<String>, // Derived from columns
    pub common_filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,

    // Transaction (if ongoing)
    pub tx: Option<sqlx::Transaction<'static, sqlx::Postgres>>,
}

impl PostgresDataStoreImpl {
    pub fn from_data_store(d: &mut dyn DataStore) -> Result<&mut Self, SettingsError> {
        d.as_any()
            .downcast_mut::<Self>()
            .ok_or(SettingsError::Generic {
                message: "Failed to downcast to PostgresDataStoreImpl".to_string(),
                src: "PostgresDataStoreImpl::from_data_store".to_string(),
                typ: "internal".to_string(),
            })
    }

    /// Binds a value to a query
    ///
    /// Note that Maps are binded as JSONs
    ///
    /// `default_column_type` - The (default) column type to use if the value is None. This should be the column_type
    fn _query_bind_value<'a>(
        query: sqlx::query::Query<'a, sqlx::Postgres, sqlx::postgres::PgArguments>,
        value: Value,
        default_column_type: &ColumnType,
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
            Value::Json(value) => query.bind(value),
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
                        Value::Json(_) => {
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
                    InnerColumnType::Json { .. } => query.bind(None::<serde_json::Value>),
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
                    InnerColumnType::Json { .. } => query.bind(None::<Vec<serde_json::Value>>),
                },
            },
        }
    }

    /// Binds filters to a query
    ///
    /// If bind_nulls is true, then entries with Value::None are also binded. This should be disabled on filters and enabled on entries
    fn bind_map<'a>(
        query: sqlx::query::Query<'a, sqlx::Postgres, sqlx::postgres::PgArguments>,
        map: indexmap::IndexMap<String, Value>,
        bind_nulls: bool,
        columns: &[Column],
    ) -> Result<sqlx::query::Query<'a, sqlx::Postgres, sqlx::postgres::PgArguments>, SettingsError>
    {
        let mut query = query;

        let mut spec_limit: Option<i64> = None;
        let mut spec_offset: Option<i64> = None;
        for (field_name, value) in map {
            if field_name == "__limit" {
                if let Value::Integer(value) = value {
                    if value < 1 {
                        return Err(SettingsError::Generic {
                            message: "__limit must be greater than 0".to_string(),
                            src: "PostgresDataStore#bind_map".to_string(),
                            typ: "internal".to_string(),
                        });
                    }
                    spec_limit = Some(value);
                }
                continue;
            } else if field_name == "__offset" {
                if let Value::Integer(value) = value {
                    if value < 0 {
                        return Err(SettingsError::Generic {
                            message: "__offset must be greater than or equal to 0".to_string(),
                            src: "PostgresDataStore#bind_map".to_string(),
                            typ: "internal".to_string(),
                        });
                    }
                    spec_offset = Some(value);
                }
                continue;
            }

            // If None, we omit the value from binding
            if !bind_nulls && matches!(value, Value::None) {
                continue;
            }

            let column =
                columns
                    .iter()
                    .find(|c| c.id == field_name)
                    .ok_or(SettingsError::Generic {
                        message: format!("Column {} not found", field_name),
                        src: "settings_view [fetch_all]".to_string(),
                        typ: "internal".to_string(),
                    })?;

            query = Self::_query_bind_value(query, value, &column.column_type);
        }

        // Add limit and offset last
        if let Some(limit) = spec_limit {
            query = query.bind(limit);
        }

        if let Some(offset) = spec_offset {
            query = query.bind(offset);
        }

        Ok(query)
    }

    /// Helper method to either perform a perform a query using either the transaction or the pool
    async fn execute_query<'a>(
        &mut self,
        query: sqlx::query::Query<'a, sqlx::Postgres, sqlx::postgres::PgArguments>,
    ) -> Result<sqlx::postgres::PgQueryResult, SettingsError> {
        // Get the transaction connection or acquire one from pool if not in a transaction
        let conn = if self.tx.is_some() {
            self.tx.as_deref_mut().unwrap()
        } else {
            &mut *self
                .pool
                .acquire()
                .await
                .map_err(|e| SettingsError::Generic {
                    message: format!("Failed to get connection: {:?}", e),
                    src: "PostgresDataStore::execute_query [query_execute]".to_string(),
                    typ: "internal".to_string(),
                })?
        };

        query
            .execute(&mut *conn)
            .await
            .map_err(|e| SettingsError::Generic {
                message: e.to_string(),
                src: "PostgresDataStore::execute_query [query_execute]".to_string(),
                typ: "internal".to_string(),
            })
    }

    /// Helper method to either perform a perform a query using either the transaction or the pool
    async fn fetchone_query<'a>(
        &mut self,
        query: sqlx::query::Query<'a, sqlx::Postgres, sqlx::postgres::PgArguments>,
    ) -> Result<sqlx::postgres::PgRow, SettingsError> {
        let query_sql = query.sql();

        // Get the transaction connection or acquire one from pool if not in a transaction
        let conn = if self.tx.is_some() {
            self.tx.as_deref_mut().unwrap()
        } else {
            &mut *self
                .pool
                .acquire()
                .await
                .map_err(|e| SettingsError::Generic {
                    message: format!("Failed to get connection: {:?}", e),
                    src: "PostgresDataStore::fetchone_query".to_string(),
                    typ: "internal".to_string(),
                })?
        };

        query
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| SettingsError::Generic {
                message: e.to_string(),
                src: format!(
                    "PostgresDataStore::fetchone_query [query_execute]: {}",
                    query_sql
                ),
                typ: "internal".to_string(),
            })
    }

    /// Helper method to either perform a perform a query using either the transaction or the pool
    async fn fetchall_query<'a>(
        &mut self,
        query: sqlx::query::Query<'a, sqlx::Postgres, sqlx::postgres::PgArguments>,
    ) -> Result<Vec<sqlx::postgres::PgRow>, SettingsError> {
        let query_sql = query.sql();

        let conn = if self.tx.is_some() {
            self.tx.as_deref_mut().unwrap()
        } else {
            &mut *self
                .pool
                .acquire()
                .await
                .map_err(|e| SettingsError::Generic {
                    message: format!("Failed to get connection: {:?}", e),
                    src: "PostgresDataStore::fetchall_query".to_string(),
                    typ: "internal".to_string(),
                })?
        };

        query
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| SettingsError::Generic {
                message: e.to_string(),
                src: format!(
                    "PostgresDataStore::fetchall_query [query_execute]: {}",
                    query_sql
                ),
                typ: "internal".to_string(),
            })
    }

    fn filter_fields(
        fields: &[String],
        valid_columns: &std::collections::HashSet<String>,
    ) -> Vec<String> {
        let mut new_fields = Vec::new();

        for f in fields {
            if valid_columns.contains(f) {
                new_fields.push(f.to_string());
            }
        }

        new_fields
    }
}

#[async_trait]
impl DataStore for PostgresDataStoreImpl {
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    async fn start_transaction(&mut self) -> Result<(), SettingsError> {
        let tx: sqlx::Transaction<'_, sqlx::Postgres> =
            self.pool
                .begin()
                .await
                .map_err(|e| SettingsError::Generic {
                    message: e.to_string(),
                    src: "PostgresDataStore::start_transaction [pool.begin]".to_string(),
                    typ: "internal".to_string(),
                })?;

        self.tx = Some(tx);

        Ok(())
    }

    async fn commit(&mut self) -> Result<(), SettingsError> {
        if let Some(tx) = self.tx.take() {
            tx.commit().await.map_err(|e| SettingsError::Generic {
                message: e.to_string(),
                src: "PostgresDataStore::commit [tx.commit]".to_string(),
                typ: "internal".to_string(),
            })?;
        }

        Ok(())
    }

    async fn columns(&mut self) -> Result<Vec<String>, SettingsError> {
        // Get columns from database
        let query = sqlx::query("SELECT column_name FROM information_schema.columns WHERE table_name = $1 ORDER BY ordinal_position")
            .bind(self.setting_table);

        let rows = self.fetchall_query(query).await?;

        let mut columns = Vec::new();

        for row in rows {
            let column_name: String =
                row.try_get("column_name")
                    .map_err(|e| SettingsError::Generic {
                        message: e.to_string(),
                        src: "PostgresDataStore::columns [row try_get]".to_string(),
                        typ: "internal".to_string(),
                    })?;

            columns.push(column_name);
        }

        Ok(columns)
    }

    async fn fetch_all(
        &mut self,
        fields: &[String],
        filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<Vec<super::state::State>, SettingsError> {
        let filters = combine_indexmaps!(filters, self.common_filters.clone());

        let sql_stmt = format!(
            "SELECT {} FROM {} WHERE {}",
            PostgresDataStoreImpl::filter_fields(fields, &self.valid_columns).join(", "),
            self.setting_table,
            sql_utils::create_where_clause(&self.valid_columns, &filters, 0).map_err(|e| {
                SettingsError::Generic {
                    message: e.to_string(),
                    src: "PostgresDataStore::fetch_all [create_where_clause]".to_string(),
                    typ: "internal".to_string(),
                }
            })?
        );

        let mut query = sqlx::query(sql_stmt.as_str());

        if !filters.is_empty() {
            query = Self::bind_map(query, filters, false, &self.columns)?;
        }

        // Execute the query and process it to a Vec<state>
        let rows = self.fetchall_query(query).await?;

        let mut values: Vec<State> = Vec::new();
        for row in rows {
            let mut state = State::new_with_special_variables(self.author, self.guild_id); // Ensure special vars are in the state

            for (i, col) in fields.iter().enumerate() {
                let val = Value::from_sqlx(&row, i).map_err(|e| SettingsError::Generic {
                    message: e.to_string(),
                    src: "PostgresDataStore::rows_to_states [Value::from_sqlx]".to_string(),
                    typ: "internal".to_string(),
                })?;

                state.state.insert(col.to_string(), val);
            }

            values.push(state);
        }

        Ok(values)
    }

    async fn matching_entry_count(
        &mut self,
        filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<usize, SettingsError> {
        let filters = combine_indexmaps!(filters, self.common_filters.clone());

        let sql_stmt = format!(
            "SELECT COUNT(*) FROM {} WHERE {}",
            self.setting_table,
            sql_utils::create_where_clause(&self.valid_columns, &filters, 0).map_err(|e| {
                SettingsError::Generic {
                    message: e.to_string(),
                    src: "PostgresDataStore::matching_entry_count [create_where_clause]"
                        .to_string(),
                    typ: "internal".to_string(),
                }
            })?
        );

        let mut query = sqlx::query(sql_stmt.as_str());

        if !filters.is_empty() {
            query = Self::bind_map(query, filters, false, &self.columns)?;
        }

        // Execute the query
        let row = self.fetchone_query(query).await?;

        let count: i64 = row.try_get(0).map_err(|e| SettingsError::Generic {
            message: e.to_string(),
            src: "PostgresDataStore::matching_entry_count [row try_get]".to_string(),
            typ: "internal".to_string(),
        })?;

        Ok(count as usize)
    }

    /// Creates a new entry given a set of columns to set returning the newly created entry
    async fn create_entry(
        &mut self,
        entry: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<super::state::State, SettingsError> {
        let entry = combine_indexmaps!(entry, self.common_filters.clone());

        // Create the row
        let (col_params, n_params) =
            sql_utils::create_col_and_n_params(&self.valid_columns, &entry, 0).map_err(|e| {
                SettingsError::Generic {
                    message: e.to_string(),
                    src: "settings_create [create_col_and_n_params]".to_string(),
                    typ: "internal".to_string(),
                }
            })?;

        // Execute the SQL statement
        let sql_stmt = format!(
            "INSERT INTO {} ({}) VALUES ({}) RETURNING {}",
            self.setting_table, col_params, n_params, self.setting_primary_key
        );

        let mut query = sqlx::query(sql_stmt.as_str());

        // Bind the sql query arguments
        let mut state = State::from_indexmap(entry.clone());

        query = Self::bind_map(query, entry, true, &self.columns)?;

        // Execute the query
        let pkey_row = self.fetchone_query(query).await?;

        // Save pkey to state
        state.state.insert(
            self.setting_primary_key.to_string(),
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
    async fn update_matching_entries(
        &mut self,
        filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
        entry: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<(), SettingsError> {
        let filters = combine_indexmaps!(filters, self.common_filters.clone());

        // Create the SQL statement
        let sql_stmt = format!(
            "UPDATE {} SET {} WHERE {}",
            self.setting_table,
            sql_utils::create_update_set_clause(&self.valid_columns, &entry, 0).map_err(|e| {
                SettingsError::Generic {
                    message: e.to_string(),
                    src: "settings_update [create_update_set_clause]".to_string(),
                    typ: "internal".to_string(),
                }
            })?,
            sql_utils::create_where_clause(&self.valid_columns, &filters, entry.len()).map_err(
                |e| {
                    SettingsError::Generic {
                        message: e.to_string(),
                        src: "settings_update [create_where_clause]".to_string(),
                        typ: "internal".to_string(),
                    }
                }
            )?
        );

        let mut query = sqlx::query(sql_stmt.as_str());

        query = Self::bind_map(query, entry, true, &self.columns)?; // Bind the entry
        query = Self::bind_map(query, filters, false, &self.columns)?; // Bind the filters

        // Execute the query
        self.execute_query(query).await?;

        Ok(())
    }

    /// Deletes entries given a set of filters
    ///
    /// Returns all deleted rows
    async fn delete_matching_entries(
        &mut self,
        filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<(), SettingsError> {
        let filters = combine_indexmaps!(filters, self.common_filters.clone());

        // Create the SQL statement
        let sql_stmt = format!(
            "DELETE FROM {} WHERE {}",
            self.setting_table,
            sql_utils::create_where_clause(&self.valid_columns, &filters, 0).map_err(|e| {
                SettingsError::Generic {
                    message: e.to_string(),
                    src: "settings_delete [create_where_clause]".to_string(),
                    typ: "internal".to_string(),
                }
            })?
        );

        let mut query = sqlx::query(sql_stmt.as_str());

        if !filters.is_empty() {
            query = Self::bind_map(query, filters, false, &self.columns)?;
        }

        // Execute the query
        let res = self.execute_query(query).await?;

        if res.rows_affected() == 0 {
            return Err(SettingsError::RowDoesNotExist {
                column_id: self.setting_primary_key.to_string(),
            });
        }

        Ok(())
    }
}

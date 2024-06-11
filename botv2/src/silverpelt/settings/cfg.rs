use super::config_opts::{ColumnType, ConfigOption, InnerColumnType, OperationType};
use super::state::State;
use crate::silverpelt::value::Value;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SettingsError {
    Generic {
        message: String,
        src: String,
        typ: String,
    },
    SchemaTypeValidationError {
        column: String,
        expected_type: String,
        got_type: String,
    },
    SchemaNullValueValidationError {
        column: String,
    },
    SchemaCheckValidationError {
        column: String,
        check: String,
        value: serde_json::Value,
        accepted_range: String,
    },
    MissingField {
        field: String,
    },
    RowExists {
        primary_key: String,
        count: usize,
    },
}

impl std::fmt::Display for SettingsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SettingsError::Generic { message, src, typ } => {
                write!(f, "{} from src `{}` of type `{}`", message, src, typ)
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
                value,
                accepted_range,
            } => {
                write!(
                    f,
                    "Column `{}` failed check `{}` with value `{}`, accepted range: `{}`",
                    column, check, value, accepted_range
                )
            }
            SettingsError::MissingField { field } => write!(f, "Missing field `{}`", field),
            SettingsError::RowExists { primary_key, count } => write!(
                f,
                "A row with the same primary key `{}` already exists. Count: {}",
                primary_key, count
            ),
        }
    }
}

/// Validates the value against the schema's column type handling schema checks if `perform_schema_checks` is true
#[allow(dead_code)]
fn _validate_value(
    v: &Value,
    column_type: &ColumnType,
    column_id: &str,
    is_nullable: bool,
    perform_schema_checks: bool,
) -> Result<(), SettingsError> {
    match column_type {
        ColumnType::Scalar { column_type } => {
            if matches!(v, Value::None) {
                if is_nullable {
                    return Ok(());
                } else {
                    return Err(SettingsError::SchemaNullValueValidationError {
                        column: column_id.to_string(),
                    });
                }
            }

            if matches!(v, Value::List(_)) {
                return Err(SettingsError::SchemaTypeValidationError {
                    column: column_id.to_string(),
                    expected_type: "Scalar".to_string(),
                    got_type: "Array".to_string(),
                });
            }

            match column_type {
                InnerColumnType::Uuid {} => {
                    if !matches!(v, Value::Uuid(_)) {
                        return Err(SettingsError::SchemaTypeValidationError {
                            column: column_id.to_string(),
                            expected_type: "Uuid".to_string(),
                            got_type: format!("{:?}", v),
                        });
                    }
                }
                InnerColumnType::String {
                    min_length,
                    max_length,
                    allowed_values,
                } => {
                    if !matches!(v, Value::String(_) | Value::Uuid(_)) {
                        return Err(SettingsError::SchemaTypeValidationError {
                            column: column_id.to_string(),
                            expected_type: "String".to_string(),
                            got_type: format!("{:?}", v),
                        });
                    }

                    if perform_schema_checks {
                        let s = match v {
                            Value::String(s) => s,
                            _ => unreachable!(),
                        };

                        if let Some(min) = min_length {
                            if s.len() < *min {
                                return Err(SettingsError::SchemaCheckValidationError {
                                    column: column_id.to_string(),
                                    check: "minlength".to_string(),
                                    value: v.to_json(),
                                    accepted_range: format!(">{}", min),
                                });
                            }
                        }

                        if let Some(max) = max_length {
                            if s.len() > *max {
                                return Err(SettingsError::SchemaCheckValidationError {
                                    column: column_id.to_string(),
                                    check: "maxlength".to_string(),
                                    value: v.to_json(),
                                    accepted_range: format!("<{}", max),
                                });
                            }
                        }

                        if !allowed_values.is_empty() && !allowed_values.contains(&s.as_str()) {
                            return Err(SettingsError::SchemaCheckValidationError {
                                column: column_id.to_string(),
                                check: "allowed_values".to_string(),
                                value: v.to_json(),
                                accepted_range: format!("{:?}", allowed_values),
                            });
                        }
                    }
                }
                InnerColumnType::Timestamp {} => {
                    if !matches!(v, Value::Timestamp(_)) {
                        return Err(SettingsError::SchemaTypeValidationError {
                            column: column_id.to_string(),
                            expected_type: "Timestamp".to_string(),
                            got_type: format!("{:?}", v),
                        });
                    }

                    // No further checks needed
                }
                InnerColumnType::TimestampTz {} => {
                    if !matches!(v, Value::TimestampTz(_)) {
                        return Err(SettingsError::SchemaTypeValidationError {
                            column: column_id.to_string(),
                            expected_type: "TimestampTz".to_string(),
                            got_type: format!("{:?}", v),
                        });
                    }

                    // No further checks needed
                }
                InnerColumnType::Integer {} => {
                    if !matches!(v, Value::Integer(_)) {
                        return Err(SettingsError::SchemaTypeValidationError {
                            column: column_id.to_string(),
                            expected_type: "Integer".to_string(),
                            got_type: format!("{:?}", v),
                        });
                    }
                }
                InnerColumnType::Float {} => {
                    if !matches!(v, Value::Float(_)) {
                        return Err(SettingsError::SchemaTypeValidationError {
                            column: column_id.to_string(),
                            expected_type: "Float".to_string(),
                            got_type: format!("{:?}", v),
                        });
                    }
                }
                InnerColumnType::BitFlag { .. } => {
                    if !matches!(v, Value::Integer(_)) {
                        return Err(SettingsError::SchemaTypeValidationError {
                            column: column_id.to_string(),
                            expected_type: "Integer".to_string(),
                            got_type: format!("{:?}", v),
                        });
                    }

                    // TODO: Add value parsing for bit flags
                }
                InnerColumnType::Boolean {} => {
                    if !matches!(v, Value::Boolean(_)) {
                        return Err(SettingsError::SchemaTypeValidationError {
                            column: column_id.to_string(),
                            expected_type: "Boolean".to_string(),
                            got_type: format!("{:?}", v),
                        });
                    }
                }
                InnerColumnType::User {} => {
                    if !matches!(v, Value::String(_)) {
                        return Err(SettingsError::SchemaTypeValidationError {
                            column: column_id.to_string(),
                            expected_type: "User (string)".to_string(),
                            got_type: format!("{:?}", v),
                        });
                    }

                    if perform_schema_checks {
                        let s = match v {
                            Value::String(s) => s,
                            _ => unreachable!(),
                        };

                        // Try parsing to a UserId
                        if s.parse::<serenity::all::UserId>().is_err() {
                            return Err(SettingsError::SchemaCheckValidationError {
                                column: column_id.to_string(),
                                check: "snowflake_parse".to_string(),
                                value: v.to_json(),
                                accepted_range: "Valid user id".to_string(),
                            });
                        }
                    }
                }
                InnerColumnType::Channel {} => {
                    if !matches!(v, Value::String(_)) {
                        return Err(SettingsError::SchemaTypeValidationError {
                            column: column_id.to_string(),
                            expected_type: "Channel (string)".to_string(),
                            got_type: format!("{:?}", v),
                        });
                    }

                    if perform_schema_checks {
                        let s = match v {
                            Value::String(s) => s,
                            _ => unreachable!(),
                        };

                        // Try parsing to a ChannelId
                        if s.parse::<serenity::all::ChannelId>().is_err() {
                            return Err(SettingsError::SchemaCheckValidationError {
                                column: column_id.to_string(),
                                check: "snowflake_parse".to_string(),
                                value: v.to_json(),
                                accepted_range: "Valid channel id".to_string(),
                            });
                        }
                    }
                }
                InnerColumnType::Role {} => {
                    if !matches!(v, Value::String(_)) {
                        return Err(SettingsError::SchemaTypeValidationError {
                            column: column_id.to_string(),
                            expected_type: "Role (string)".to_string(),
                            got_type: format!("{:?}", v),
                        });
                    }

                    if perform_schema_checks {
                        let s = match v {
                            Value::String(s) => s,
                            _ => unreachable!(),
                        };

                        // Try parsing to a RoleId
                        if s.parse::<serenity::all::RoleId>().is_err() {
                            return Err(SettingsError::SchemaCheckValidationError {
                                column: column_id.to_string(),
                                check: "snowflake_parse".to_string(),
                                value: v.to_json(),
                                accepted_range: "Valid role id".to_string(),
                            });
                        }
                    }
                }
                InnerColumnType::Emoji {} => {
                    if !matches!(v, Value::String(_)) {
                        return Err(SettingsError::SchemaTypeValidationError {
                            column: column_id.to_string(),
                            expected_type: "Emoji (string)".to_string(),
                            got_type: format!("{:?}", v),
                        });
                    }

                    if perform_schema_checks {
                        let s = match v {
                            Value::String(s) => s,
                            _ => unreachable!(),
                        };

                        // Try parsing to an EmojiId
                        if s.parse::<serenity::all::EmojiId>().is_err() {
                            return Err(SettingsError::SchemaCheckValidationError {
                                column: column_id.to_string(),
                                check: "snowflake_parse".to_string(),
                                value: v.to_json(),
                                accepted_range: "Valid emoji id".to_string(),
                            });
                        }
                    }
                }
                InnerColumnType::Message {} => {
                    if !matches!(v, Value::String(_)) {
                        return Err(SettingsError::SchemaTypeValidationError {
                            column: column_id.to_string(),
                            expected_type: "Message (string)".to_string(),
                            got_type: format!("{:?}", v),
                        });
                    }

                    if perform_schema_checks {
                        let s = match v {
                            Value::String(s) => s,
                            _ => unreachable!(),
                        };

                        // The format of a message on db should be channel_id/message_id
                        //
                        // So, split by '/' and check if the first part is a valid channel id
                        // and the second part is a valid message id
                        let parts: Vec<&str> = s.split('/').collect();

                        if parts.len() != 2 {
                            return Err(SettingsError::SchemaCheckValidationError {
                                column: column_id.to_string(),
                                check: "message_parse_plength".to_string(),
                                value: v.to_json(),
                                accepted_range:
                                    "Valid message id in format <channel_id>/<message_id>"
                                        .to_string(),
                            });
                        }

                        // Try parsing to a ChannelId
                        if parts[0].parse::<serenity::all::ChannelId>().is_err() {
                            return Err(SettingsError::SchemaCheckValidationError {
                                column: column_id.to_string(),
                                check: "message_parse_0".to_string(),
                                value: v.to_json(),
                                accepted_range:
                                    "Valid message id in format <channel_id>/<message_id>"
                                        .to_string(),
                            });
                        }

                        if parts[1].parse::<serenity::all::MessageId>().is_err() {
                            return Err(SettingsError::SchemaCheckValidationError {
                                column: column_id.to_string(),
                                check: "message_parse_1".to_string(),
                                value: v.to_json(),
                                accepted_range:
                                    "Valid message id in format <channel_id>/<message_id>"
                                        .to_string(),
                            });
                        }
                    }
                }
                InnerColumnType::Json {} => {
                    if !matches!(v, Value::Map(_)) {
                        return Err(SettingsError::SchemaTypeValidationError {
                            column: column_id.to_string(),
                            expected_type: "Json".to_string(),
                            got_type: format!("{:?}", v),
                        });
                    }
                }
            }
        }
        ColumnType::Array { inner } => {
            if matches!(v, Value::None) {
                if is_nullable {
                    return Ok(());
                } else {
                    return Err(SettingsError::SchemaNullValueValidationError {
                        column: column_id.to_string(),
                    });
                }
            }

            if !matches!(v, Value::List(_)) {
                return Err(SettingsError::SchemaTypeValidationError {
                    column: column_id.to_string(),
                    expected_type: "Array".to_string(),
                    got_type: format!("{:?}", v),
                });
            }

            let l = match v {
                Value::List(l) => l,
                _ => unreachable!(),
            };

            let column_type = ColumnType::new_scalar(inner.clone());
            for v in l {
                _validate_value(
                    v,
                    &column_type,
                    column_id,
                    is_nullable,
                    perform_schema_checks,
                )?;
            }
        }
    }

    Ok(())
}

/// Returns the column ids for the given operation given the config option (setting) and the operation type.
/// The returned column ids are sorted based on the order of the columns in the setting.
///
/// Note that fields like ``ignored_for`` are not handled here as they are operation specific
fn _getcols(setting: &ConfigOption) -> Vec<String> {
    let mut cols = vec![];

    for col in &setting.columns {
        cols.push(col.id.to_string());
    }

    // Sort the cols vec based on the setting.columns order
    cols.sort_by(|a, b| {
        let a = setting.columns.iter().position(|c| c.id == a);
        let b = setting.columns.iter().position(|c| c.id == b);

        a.cmp(&b)
    });

    cols
}

/// Binds a value to a query
///
/// Note that Maps are binded as JSONs
fn _query_bind_value(
    query: sqlx::query::Query<'_, sqlx::Postgres, sqlx::postgres::PgArguments>,
    value: Value,
) -> sqlx::query::Query<'_, sqlx::Postgres, sqlx::postgres::PgArguments> {
    match value {
        Value::Uuid(value) => query.bind(value),
        Value::String(value) => query.bind(value),
        Value::Timestamp(value) => query.bind(value),
        Value::TimestampTz(value) => query.bind(value),
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
                    // TODO: Improve this, right now, we fallback to string
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
        Value::None => query.bind(None::<String>),
    }
}

/// Settings API: View implementation
pub async fn settings_view(
    setting: &ConfigOption,
    ctx: &serenity::all::Context,
    pool: &sqlx::PgPool,
    guild_id: serenity::all::GuildId,
    author: serenity::all::UserId,
) -> Result<Vec<State>, SettingsError> {
    let cols = _getcols(setting);

    let row = sqlx::query(
        format!(
            "SELECT {} FROM {} WHERE {} = $1",
            cols.join(", "),
            setting.table,
            setting.guild_id
        )
        .as_str(),
    )
    .bind(guild_id.to_string())
    .fetch_all(pool)
    .await
    .map_err(|e| SettingsError::Generic {
        message: e.to_string(),
        src: "settings_view [query fetch_all]".to_string(),
        typ: "internal".to_string(),
    })?;

    if row.is_empty() {
        return Ok(Vec::new());
    }

    let mut values: Vec<State> = Vec::new();

    for row in row {
        let mut state = State::new();

        for (i, col) in setting.columns.iter().enumerate() {
            // Fetch and validate the value
            let val = Value::from_sqlx(&row, i).map_err(|e| SettingsError::Generic {
                message: e.to_string(),
                src: "_parse_row [Value::from_sqlx]".to_string(),
                typ: "internal".to_string(),
            })?;
            _validate_value(&val, &col.column_type, col.id, col.nullable, false)?;

            let actions = col
                .pre_checks
                .get(&OperationType::View)
                .unwrap_or(&col.default_pre_checks);

            // Insert the value into the map
            state.state.insert(col.id.to_string(), val);

            crate::silverpelt::settings::action_executor::execute_actions(
                &mut state, actions, ctx, author, guild_id,
            )
            .await
            .map_err(|e| SettingsError::Generic {
                message: e.to_string(),
                src: "_parse_row [execute_actions]".to_string(),
                typ: "internal".to_string(),
            })?;
        }

        // Post operation column set
        //
        // We optimize this to perform one query per table
        if let Some(op_specific) = setting.operations.get(&OperationType::View) {
            for (table_name, col_values) in op_specific.columns_to_set.iter() {
                let mut set_stmt = "".to_string();
                let mut values = Vec::new();
                for (i, (column, value)) in col_values.iter().enumerate() {
                    set_stmt.push_str(&format!("{} = ${}", column, i + 1));

                    if i != col_values.len() - 1 {
                        set_stmt.push(',');
                    }

                    let value = state.template_to_string(author, guild_id, value);
                    values.push(value.clone());

                    // For auditing/state checking purposes, add to state as __{tablename}_{columnname}_postop
                    state
                        .state
                        .insert(format!("__{}_{}_postop", table_name, column), value);
                }

                let sql_stmt = format!(
                    "UPDATE {} SET {} WHERE {} = ${}",
                    table_name,
                    set_stmt,
                    setting.guild_id,
                    cols.len() + 1
                );

                let mut query = sqlx::query(sql_stmt.as_str());

                for value in values {
                    query = _query_bind_value(query, value);
                }

                query
                    .bind(guild_id.to_string())
                    .execute(pool)
                    .await
                    .map_err(|e| SettingsError::Generic {
                        message: e.to_string(),
                        src: "_parse_row [query execute]".to_string(),
                        typ: "internal".to_string(),
                    })?;
            }
        }

        // Remove ignored columns now that the actions have been executed +
        for col in &setting.columns {
            if col.ignored_for.contains(&OperationType::View) {
                state.state.shift_remove(col.id);
            }
        }

        values.push(state);
    }

    Ok(values)
}

pub async fn settings_create(
    setting: &ConfigOption,
    ctx: &serenity::all::Context,
    pool: &sqlx::PgPool,
    guild_id: serenity::all::GuildId,
    author: serenity::all::UserId,
    fields: indexmap::IndexMap<String, Value>,
) -> Result<State, SettingsError> {
    let cols = _getcols(setting);

    // Ensure all columns exist in fields, note that we can ignore extra fields so this one single loop is enough
    let mut ignored_for = Vec::new();
    let mut state: State = State::new();
    for col in cols.iter() {
        // Get the column from the setting
        let Some(column) = setting.columns.iter().find(|c| c.id == col) else {
            return Err(SettingsError::Generic {
                message: format!("Column `{}` not found in setting", col),
                src: "settings_create [column not found]".to_string(),
                typ: "internal/backend".to_string(), // internal/backend as this is a clear backend error
            });
        };

        // If the column is ignored for create, skip
        if column.ignored_for.contains(&OperationType::Create) {
            // Add to ignore_for and set null placeholder for actions
            ignored_for.push(col.clone());
            state.state.insert(col.to_string(), Value::None);
        } else {
            // Find value and validate it
            let value = match fields.get(col) {
                Some(val) => val.clone(),
                None => {
                    // Check if the column is nullable
                    if !column.nullable {
                        return Err(SettingsError::MissingField {
                            field: col.to_string(),
                        });
                    }

                    Value::None
                }
            };

            _validate_value(
                &value,
                &column.column_type,
                column.id,
                column.nullable,
                true,
            )?;
        }

        // Execute actions
        let actions = column
            .pre_checks
            .get(&OperationType::Create)
            .unwrap_or(&column.default_pre_checks);

        crate::silverpelt::settings::action_executor::execute_actions(
            &mut state, actions, ctx, author, guild_id,
        )
        .await
        .map_err(|e| SettingsError::Generic {
            message: e.to_string(),
            src: "_parse_row [execute_actions]".to_string(),
            typ: "internal".to_string(),
        })?;
    }

    // Ensure that a field with the same primary key doesn't exist
    let row = sqlx::query(
        format!(
            "SELECT {} FROM {} WHERE {} = $1",
            setting.primary_key, setting.table, setting.guild_id
        )
        .as_str(),
    )
    .bind(guild_id.to_string())
    .fetch_all(pool)
    .await
    .map_err(|e| SettingsError::Generic {
        message: e.to_string(),
        src: "settings_create [query fetch_all]".to_string(),
        typ: "internal".to_string(),
    })?;

    if !row.is_empty() {
        return Err(SettingsError::RowExists {
            primary_key: setting.primary_key.to_string(),
            count: row.len(),
        });
    }

    // Add table_colsets for our table to state as well, as the actual insert uses state as well, this should just work TM
    //
    // Note that we only add the columns for our own table here, the rest happen after the initial update
    if let Some(op_specific) = setting.operations.get(&OperationType::Create) {
        let table_colsets = op_specific.columns_to_set.get(&setting.table);

        if let Some(table_colsets) = table_colsets {
            for (column, value) in table_colsets.iter() {
                state.state.insert(
                    column.to_string(),
                    state.template_to_string(author, guild_id, value),
                );
            }
        }
    }

    // Create the row

    // First create the $N's from the cols starting with 2 as 1 is the guild_id
    let mut n_params = "".to_string();
    let mut col_params = "".to_string();
    for (i, (col, _)) in state.state.iter().enumerate() {
        if ignored_for.contains(col) {
            continue;
        }

        n_params.push_str(&format!("${}", i + 2));
        col_params.push_str(col);

        if i != cols.len() - 1 {
            n_params.push(',');
            col_params.push(',');
        }
    }

    // Execute the SQL statement
    let sql_stmt = format!(
        "INSERT INTO {} ({}, {}) VALUES ($1, {})",
        setting.table, setting.guild_id, col_params, n_params
    );

    let mut query = sqlx::query(sql_stmt.as_str());

    // Bind the sql query arguments
    query = query.bind(guild_id.to_string());

    for (col, value) in state.state.iter() {
        if ignored_for.contains(col) {
            continue;
        }

        query = _query_bind_value(query, value.clone());
    }

    // Execute the query
    query
        .execute(pool)
        .await
        .map_err(|e| SettingsError::Generic {
            message: e.to_string(),
            src: "settings_create [query execute]".to_string(),
            typ: "internal".to_string(),
        })?;

    // Post operation column set
    //
    // We optimize this to perform one query per table
    if let Some(op_specific) = setting.operations.get(&OperationType::Create) {
        for (table_name, col_values) in op_specific.columns_to_set.iter() {
            if table_name == &setting.table {
                continue; // Skip the table we just inserted into
            }

            let mut set_stmt = "".to_string();
            let mut values = Vec::new();
            for (i, (column, value)) in col_values.iter().enumerate() {
                set_stmt.push_str(&format!("{} = ${}", column, i + 1));

                if i != col_values.len() - 1 {
                    set_stmt.push(',');
                }

                let value = state.template_to_string(author, guild_id, value);
                values.push(value.clone());

                // For auditing/state checking purposes, add to state as __{tablename}_{columnname}_postop
                state
                    .state
                    .insert(format!("__{}_{}_postop", table_name, column), value);
            }

            let sql_stmt = format!(
                "UPDATE {} SET {} WHERE {} = ${}",
                table_name,
                set_stmt,
                setting.guild_id,
                cols.len() + 1
            );

            let mut query = sqlx::query(sql_stmt.as_str());

            for value in values {
                query = _query_bind_value(query, value);
            }

            query
                .bind(guild_id.to_string())
                .execute(pool)
                .await
                .map_err(|e| SettingsError::Generic {
                    message: e.to_string(),
                    src: "_parse_row [query execute]".to_string(),
                    typ: "internal".to_string(),
                })?;
        }
    }

    Ok(state)
}

use super::state::State;
use super::types::SettingsError;
use super::types::{
    ColumnType, ConfigOption, InnerColumnType, InnerColumnTypeStringKind, OperationType,
};
use splashcore_rs::value::Value;
use sqlx::Row;

/// Validates the value against the schema's column type handling schema checks if `perform_schema_checks` is true
#[allow(dead_code)]
#[async_recursion::async_recursion]
async fn _validate_and_parse_value(
    v: Value,
    column_type: &ColumnType,
    column_id: &str,
    is_nullable: bool,
    perform_schema_checks: bool,
) -> Result<Value, SettingsError> {
    match column_type {
        ColumnType::Scalar { column_type } => {
            if matches!(v, Value::None) {
                if is_nullable {
                    return Ok(Value::None);
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
                InnerColumnType::Uuid {} => match v {
                    Value::String(s) => {
                        let value = s.parse::<sqlx::types::Uuid>().map_err(|e| {
                            SettingsError::SchemaCheckValidationError {
                                column: column_id.to_string(),
                                check: "uuid_parse".to_string(),
                                accepted_range: "Valid UUID".to_string(),
                                error: e.to_string(),
                            }
                        })?;

                        Ok(Value::Uuid(value))
                    }
                    Value::Uuid(_) => Ok(v),
                    _ => Err(SettingsError::SchemaTypeValidationError {
                        column: column_id.to_string(),
                        expected_type: "Uuid".to_string(),
                        got_type: format!("{:?}", v),
                    }),
                },
                InnerColumnType::String {
                    min_length,
                    max_length,
                    allowed_values,
                    kind,
                } => match v {
                    Value::String(ref s) => {
                        if perform_schema_checks {
                            if let Some(min) = min_length {
                                if s.len() < *min {
                                    return Err(SettingsError::SchemaCheckValidationError {
                                        column: column_id.to_string(),
                                        check: "minlength".to_string(),
                                        accepted_range: format!(">{}", min),
                                        error: "s.len() < *min".to_string(),
                                    });
                                }
                            }

                            if let Some(max) = max_length {
                                if s.len() > *max {
                                    return Err(SettingsError::SchemaCheckValidationError {
                                        column: column_id.to_string(),
                                        check: "maxlength".to_string(),
                                        accepted_range: format!("<{}", max),
                                        error: "s.len() > *max".to_string(),
                                    });
                                }
                            }

                            if !allowed_values.is_empty() && !allowed_values.contains(&s.as_str()) {
                                return Err(SettingsError::SchemaCheckValidationError {
                                    column: column_id.to_string(),
                                    check: "allowed_values".to_string(),
                                    accepted_range: format!("{:?}", allowed_values),
                                    error: "!allowed_values.is_empty() && !allowed_values.contains(&s.as_str())".to_string()
                                });
                            }

                            match kind {
                                InnerColumnTypeStringKind::Normal => {}
                                InnerColumnTypeStringKind::Template => {
                                    let compiled = templating::compile_template(
                                        s,
                                        templating::CompileTemplateOptions {
                                            cache_result: false, // Don't uselessly cache the template to decrease memory footprint
                                            ignore_cache: false, // Don't ignore the cache to avoid recompiling the same template over and over
                                        },
                                    )
                                    .await;

                                    if let Err(err) = compiled {
                                        return Err(SettingsError::SchemaCheckValidationError {
                                            column: column_id.to_string(),
                                            check: "template_compile".to_string(),
                                            accepted_range: "Valid tera template".to_string(),
                                            error: err.to_string(),
                                        });
                                    }
                                }
                                InnerColumnTypeStringKind::User => {
                                    // Try parsing to a UserId
                                    if let Err(err) = s.parse::<serenity::all::UserId>() {
                                        return Err(SettingsError::SchemaCheckValidationError {
                                            column: column_id.to_string(),
                                            check: "snowflake_parse".to_string(),
                                            accepted_range: "Valid user id".to_string(),
                                            error: err.to_string(),
                                        });
                                    }
                                }
                                InnerColumnTypeStringKind::Channel => {
                                    // Try parsing to a ChannelId
                                    if let Err(err) = s.parse::<serenity::all::ChannelId>() {
                                        return Err(SettingsError::SchemaCheckValidationError {
                                            column: column_id.to_string(),
                                            check: "snowflake_parse".to_string(),
                                            accepted_range: "Valid channel id".to_string(),
                                            error: err.to_string(),
                                        });
                                    }
                                }
                                InnerColumnTypeStringKind::Role => {
                                    // Try parsing to a ChannelId
                                    if let Err(err) = s.parse::<serenity::all::RoleId>() {
                                        return Err(SettingsError::SchemaCheckValidationError {
                                            column: column_id.to_string(),
                                            check: "snowflake_parse".to_string(),
                                            accepted_range: "Valid role id".to_string(),
                                            error: err.to_string(),
                                        });
                                    }
                                }
                                InnerColumnTypeStringKind::Emoji => {
                                    // Try parsing to a ChannelId
                                    if let Err(err) = s.parse::<serenity::all::EmojiId>() {
                                        return Err(SettingsError::SchemaCheckValidationError {
                                            column: column_id.to_string(),
                                            check: "snowflake_parse".to_string(),
                                            accepted_range: "Valid emoji id".to_string(),
                                            error: err.to_string(),
                                        });
                                    }
                                }
                                InnerColumnTypeStringKind::Message => {
                                    // The format of a message on db should be channel_id/message_id
                                    //
                                    // So, split by '/' and check if the first part is a valid channel id
                                    // and the second part is a valid message id
                                    let parts: Vec<&str> = s.split('/').collect();

                                    if parts.len() != 2 {
                                        return Err(SettingsError::SchemaCheckValidationError {
                                            column: column_id.to_string(),
                                            check: "message_parse_plength".to_string(),
                                            accepted_range:
                                                "Valid message id in format <channel_id>/<message_id>"
                                                    .to_string(),
                                            error: "parts.len() != 2".to_string(),
                                        });
                                    }

                                    // Try parsing to a ChannelId
                                    if let Err(err) = parts[0].parse::<serenity::all::ChannelId>() {
                                        return Err(SettingsError::SchemaCheckValidationError {
                                            column: column_id.to_string(),
                                            check: "message_parse_0".to_string(),
                                            accepted_range:
                                                "Valid message id in format <channel_id>/<message_id>"
                                                    .to_string(),
                                            error: format!("p1: {}", err),
                                        });
                                    }

                                    // Try parsing to a MessageId
                                    if let Err(err) = parts[1].parse::<serenity::all::MessageId>() {
                                        return Err(SettingsError::SchemaCheckValidationError {
                                            column: column_id.to_string(),
                                            check: "message_parse_1".to_string(),
                                            accepted_range:
                                                "Valid message id in format <channel_id>/<message_id>"
                                                    .to_string(),
                                            error: format!("p2: {}", err),
                                        });
                                    }
                                }
                            }
                        }
                        Ok(v)
                    }
                    Value::Uuid(v) => Ok(Value::String(v.to_string())),
                    _ => Err(SettingsError::SchemaTypeValidationError {
                        column: column_id.to_string(),
                        expected_type: "String".to_string(),
                        got_type: format!("{:?}", v),
                    }),
                },
                InnerColumnType::Timestamp {} => match v {
                    Value::String(s) => {
                        let value = chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
                            .map_err(|e| SettingsError::SchemaCheckValidationError {
                                column: column_id.to_string(),
                                check: "timestamp_parse".to_string(),
                                accepted_range: "Valid timestamp".to_string(),
                                error: e.to_string(),
                            })?;

                        Ok(Value::Timestamp(value))
                    }
                    Value::Timestamp(_) => Ok(v),
                    Value::TimestampTz(v) => Ok(Value::Timestamp(v.naive_utc())),
                    _ => Err(SettingsError::SchemaTypeValidationError {
                        column: column_id.to_string(),
                        expected_type: "Timestamp".to_string(),
                        got_type: format!("{:?}", v),
                    }),
                },
                InnerColumnType::TimestampTz {} => match v {
                    Value::String(s) => {
                        let value = chrono::DateTime::parse_from_rfc3339(&s).map_err(|e| {
                            SettingsError::SchemaCheckValidationError {
                                column: column_id.to_string(),
                                check: "timestamp_tz_parse".to_string(),
                                accepted_range: "Valid timestamp with timezone".to_string(),
                                error: e.to_string(),
                            }
                        })?;

                        // Convert value to DateTime<Utc> from DateTime<FixedOffset>
                        let value: chrono::DateTime<chrono::Utc> =
                            chrono::DateTime::from_naive_utc_and_offset(
                                value.naive_utc(),
                                chrono::Utc,
                            );

                        Ok(Value::TimestampTz(value))
                    }
                    Value::Timestamp(v) => Ok(Value::TimestampTz(
                        chrono::DateTime::from_naive_utc_and_offset(v, chrono::Utc),
                    )),
                    Value::TimestampTz(_) => Ok(v),
                    _ => Err(SettingsError::SchemaTypeValidationError {
                        column: column_id.to_string(),
                        expected_type: "TimestampTz".to_string(),
                        got_type: format!("{:?}", v),
                    }),
                },
                InnerColumnType::Integer {} => match v {
                    Value::String(s) => {
                        let value = s.parse::<i64>().map_err(|e| {
                            SettingsError::SchemaCheckValidationError {
                                column: column_id.to_string(),
                                check: "integer_parse".to_string(),
                                accepted_range: "Valid integer".to_string(),
                                error: e.to_string(),
                            }
                        })?;

                        Ok(Value::Integer(value))
                    }
                    Value::Integer(v) => Ok(Value::Integer(v)),
                    _ => Err(SettingsError::SchemaTypeValidationError {
                        column: column_id.to_string(),
                        expected_type: "Integer".to_string(),
                        got_type: format!("{:?}", v),
                    }),
                },
                InnerColumnType::Float {} => match v {
                    Value::String(s) => {
                        let value = s.parse::<f64>().map_err(|e| {
                            SettingsError::SchemaCheckValidationError {
                                column: column_id.to_string(),
                                check: "float_parse".to_string(),
                                accepted_range: "Valid float".to_string(),
                                error: e.to_string(),
                            }
                        })?;

                        Ok(Value::Float(value))
                    }
                    Value::Float(v) => Ok(Value::Float(v)),
                    _ => Err(SettingsError::SchemaTypeValidationError {
                        column: column_id.to_string(),
                        expected_type: "Float".to_string(),
                        got_type: format!("{:?}", v),
                    }),
                },
                InnerColumnType::BitFlag { values } => match v {
                    Value::Integer(v) => {
                        let mut final_value = 0;

                        // Set all the valid bits in final_value to ensure no unknown bits are being set
                        for (_, bit) in values.iter() {
                            if *bit & v == *bit {
                                final_value |= *bit;
                            }
                        }

                        Ok(Value::Integer(final_value))
                    }
                    _ => Err(SettingsError::SchemaTypeValidationError {
                        column: column_id.to_string(),
                        expected_type: "Integer".to_string(),
                        got_type: format!("{:?}", v),
                    }),
                },
                InnerColumnType::Boolean {} => match v {
                    Value::String(s) => {
                        let value = s.parse::<bool>().map_err(|e| {
                            SettingsError::SchemaCheckValidationError {
                                column: column_id.to_string(),
                                check: "boolean_parse".to_string(),
                                accepted_range: "Valid boolean".to_string(),
                                error: e.to_string(),
                            }
                        })?;

                        Ok(Value::Boolean(value))
                    }
                    Value::Boolean(v) => Ok(Value::Boolean(v)),
                    _ => Err(SettingsError::SchemaTypeValidationError {
                        column: column_id.to_string(),
                        expected_type: "Boolean".to_string(),
                        got_type: format!("{:?}", v),
                    }),
                },
                InnerColumnType::Json {} => match v {
                    Value::Map(_) => Ok(v),
                    _ => Err(SettingsError::SchemaTypeValidationError {
                        column: column_id.to_string(),
                        expected_type: "Json".to_string(),
                        got_type: format!("{:?}", v),
                    }),
                },
            }
        }
        ColumnType::Array { inner } => {
            if matches!(v, Value::None) {
                if is_nullable {
                    return Ok(Value::None);
                } else {
                    return Err(SettingsError::SchemaNullValueValidationError {
                        column: column_id.to_string(),
                    });
                }
            }

            match v {
                Value::List(l) => {
                    let mut values: Vec<Value> = Vec::new();

                    let column_type = ColumnType::new_scalar(inner.clone());
                    for v in l {
                        let new_v = _validate_and_parse_value(
                            v,
                            &column_type,
                            column_id,
                            is_nullable,
                            perform_schema_checks,
                        )
                        .await?;

                        values.push(new_v);
                    }

                    Ok(Value::List(values))
                }
                _ => Err(SettingsError::SchemaTypeValidationError {
                    column: column_id.to_string(),
                    expected_type: "Array".to_string(),
                    got_type: format!("{:?}", v),
                }),
            }
        }
    }
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
                InnerColumnType::Integer {} => query.bind(None::<Vec<i64>>),
                InnerColumnType::Float {} => query.bind(None::<Vec<f64>>),
                InnerColumnType::BitFlag { .. } => query.bind(None::<Vec<i64>>),
                InnerColumnType::Boolean {} => query.bind(None::<Vec<bool>>),
                InnerColumnType::Json {} => query.bind(None::<Vec<serde_json::Value>>),
            },
        },
    }
}

/// Settings API: View implementation
pub async fn settings_view(
    setting: &ConfigOption,
    cache_http: &botox::cache::CacheHttpImpl,
    pool: &sqlx::PgPool,
    guild_id: serenity::all::GuildId,
    author: serenity::all::UserId,
    permodule_executor: &dyn base_data::permodule::PermoduleFunctionExecutor,
) -> Result<Vec<State>, SettingsError> {
    let Some(operation_specific) = setting.operations.get(&OperationType::View) else {
        return Err(SettingsError::OperationNotSupported {
            operation: OperationType::View,
        });
    };

    let cols = setting
        .columns
        .iter()
        .map(|c| c.id.to_string())
        .collect::<Vec<String>>();

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

        // We know that the columns are in the same order as the row
        for (i, col) in setting.columns.iter().enumerate() {
            // Fetch and validate the value
            let mut val = Value::from_sqlx(&row, i).map_err(|e| SettingsError::Generic {
                message: e.to_string(),
                src: "settings_view [Value::from_sqlx]".to_string(),
                typ: "internal".to_string(),
            })?;

            // Validate the value. returning the parsed value
            val = _validate_and_parse_value(val, &col.column_type, col.id, col.nullable, false)
                .await?;

            let actions = col
                .pre_checks
                .get(&OperationType::View)
                .unwrap_or(&col.default_pre_checks);

            // Insert the value into the map
            state.state.insert(col.id.to_string(), val);

            super::action_executor::execute_actions(
                &mut state,
                actions,
                cache_http,
                pool,
                author,
                guild_id,
                permodule_executor,
            )
            .await?;
        }

        // Get out the pkey and pkey_column data here as we need it for the rest of the update
        let Some(pkey) = state.state.get(setting.primary_key) else {
            return Err(SettingsError::MissingOrInvalidField {
                field: setting.primary_key.to_string(),
                src: "settings_update [pkey_let]".to_string(),
            });
        };

        let pkey = pkey.clone(); // Clone to avoid immutable borrow

        let Some(pkey_column) = setting.columns.iter().find(|c| c.id == setting.primary_key) else {
            return Err(SettingsError::Generic {
                message: "Primary key column not found".to_string(),
                src: "settings_update [pkey_column_let_else]".to_string(),
                typ: "internal".to_string(),
            });
        };

        // Apply columns_to_set in operation specific data if there are columns to set
        if !operation_specific.columns_to_set.is_empty() {
            let mut set_stmt = "".to_string();
            let mut values = Vec::new();

            let mut i = 0;
            for (col, value) in operation_specific.columns_to_set.iter() {
                // Get column type from schema for db query hinting
                let Some(column) = setting.columns.iter().find(|c| c.id == *col) else {
                    return Err(SettingsError::Generic {
                        message: format!("Column `{}` not found in schema", col),
                        src: "settings_create [column_type_let_else]".to_string(),
                        typ: "internal".to_string(),
                    });
                };

                set_stmt.push_str(&format!("{} = ${}, ", col, i + 1));

                let value = state.template_to_string(author, guild_id, value);
                values.push((value.clone(), &column.column_type));

                // Add directly to state
                state.state.insert(col.to_string(), value);

                i += 1;
            }

            // Remove the trailing comma
            set_stmt.pop();

            let sql_stmt = format!(
                "UPDATE {} SET {} WHERE {} = ${} AND {} = ${}",
                setting.table,
                set_stmt,
                setting.guild_id,
                i + 1,
                setting.primary_key,
                i + 2
            );

            let mut query = sqlx::query(sql_stmt.as_str());

            for (value, column_type) in values {
                query = _query_bind_value(query, value, column_type);
            }

            query = query.bind(guild_id.to_string());
            query = _query_bind_value(query, pkey, &pkey_column.column_type);

            query
                .execute(pool)
                .await
                .map_err(|e| SettingsError::Generic {
                    message: e.to_string(),
                    src: "settings_view [query execute]".to_string(),
                    typ: "internal".to_string(),
                })?;
        }

        // Remove ignored columns + secret columns now that the actions have been executed
        for col in setting.columns.iter() {
            if col.secret.is_some() {
                state.state.swap_remove(col.id);
                continue; // Skip secret columns in view. **this applies to view and update only as create is creating a new object**
            }

            if state.bypass_ignore_for.contains(col.id) {
                continue;
            }

            if col.ignored_for.contains(&OperationType::View) {
                state.state.swap_remove(col.id);
            }
        }

        values.push(state);
    }

    Ok(values)
}

/// Settings API: Create implementation
pub async fn settings_create(
    setting: &ConfigOption,
    cache_http: &botox::cache::CacheHttpImpl,
    pool: &sqlx::PgPool,
    guild_id: serenity::all::GuildId,
    author: serenity::all::UserId,
    fields: indexmap::IndexMap<String, Value>,
    permodule_executor: &dyn base_data::permodule::PermoduleFunctionExecutor,
) -> Result<State, SettingsError> {
    let Some(operation_specific) = setting.operations.get(&OperationType::Create) else {
        return Err(SettingsError::OperationNotSupported {
            operation: OperationType::Create,
        });
    };

    let mut fields = fields; // Make fields mutable, consuming the input

    // Ensure all columns exist in fields, note that we can ignore extra fields so this one single loop is enough
    let mut state: State = State::new();
    for column in setting.columns.iter() {
        // If the column is ignored for create, skip
        // If the column is a secret column, then ensure we set it to something random as this is a create operation
        let value = {
            if column.ignored_for.contains(&OperationType::Create) {
                match column.secret {
                    Some(length) => Value::String(botox::crypto::gen_random(length)),
                    None => Value::None,
                }
            } else {
                match fields.swap_remove(column.id) {
                    Some(val) => {
                        if matches!(val, Value::None) {
                            // If the value is None, then it should be treated as if omitted (null)
                            match column.secret {
                                Some(length) => Value::String(botox::crypto::gen_random(length)),
                                None => Value::None,
                            }
                        } else {
                            _validate_and_parse_value(
                                val,
                                &column.column_type,
                                column.id,
                                column.nullable,
                                true,
                            )
                            .await?
                        }
                    }
                    None => match column.secret {
                        Some(length) => Value::String(botox::crypto::gen_random(length)),
                        None => Value::None,
                    },
                }
            }
        };

        // Insert the value into the state
        state.state.insert(column.id.to_string(), value);
    }

    drop(fields); // Drop fields to avoid accidental use of user data
    #[allow(unused_variables)]
    let fields = (); // Reset fields to avoid accidental use of user data

    // Start the transaction now that basic validation is done
    let mut tx = pool.begin().await.map_err(|e| SettingsError::Generic {
        message: e.to_string(),
        src: "settings_create [pool.begin]".to_string(),
        typ: "internal".to_string(),
    })?;

    // Get all ids we currently have to check max_entries and uniqueness of the primary key in one shot
    let sql_stmt = format!(
        "SELECT {} FROM {} WHERE {} = $1",
        setting.primary_key, setting.table, setting.guild_id
    );

    let query = sqlx::query(sql_stmt.as_str()).bind(guild_id.to_string());

    let row = query
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| SettingsError::Generic {
            message: e.to_string(),
            src: "settings_create [query fetch_all]".to_string(),
            typ: "internal".to_string(),
        })?;

    if row.len() > setting.max_entries {
        return Err(SettingsError::MaximumCountReached {
            max: setting.max_entries,
            current: row.len(),
        });
    }

    // Check pkey uniqueness here
    let mut ids: Vec<Value> = Vec::with_capacity(row.len());

    for (i, row) in row.iter().enumerate() {
        let id = Value::from_sqlx(row, i).map_err(|e| SettingsError::Generic {
            message: e.to_string(),
            src: "settings_create [Value::from_sqlx]".to_string(),
            typ: "internal".to_string(),
        })?;

        // Check if the pkey is unique
        if state.state.get(setting.primary_key) == Some(&id) {
            return Err(SettingsError::RowExists {
                column_id: setting.primary_key.to_string(),
                count: 1,
            });
        }

        ids.push(id);
    }

    // Now execute all actions and handle null/unique/pkey checks
    for column in setting.columns.iter() {
        // Execute actions
        let actions = column
            .pre_checks
            .get(&OperationType::Create)
            .unwrap_or(&column.default_pre_checks);

        super::action_executor::execute_actions(
            &mut state,
            actions,
            cache_http,
            pool,
            author,
            guild_id,
            permodule_executor,
        )
        .await?;

        // Checks should only happen if the column is not being intentionally ignored
        if column.ignored_for.contains(&OperationType::Create) {
            continue;
        }

        let Some(value) = state.state.get(column.id) else {
            return Err(SettingsError::Generic {
                message: format!(
                    "Column `{}` not found in state despite just being parsed",
                    column.id
                ),
                src: "settings_create [ext_checks]".to_string(),
                typ: "internal".to_string(),
            });
        };

        // Check if the column is nullable
        if !column.nullable && matches!(value, Value::None) {
            return Err(SettingsError::MissingOrInvalidField {
                field: column.id.to_string(),
                src: "settings_create [null check]".to_string(),
            });
        }

        // Handle cases of uniqueness
        //
        // In the case of create, we can do this directly within the column validation
        if column.unique {
            match value {
                Value::None => {
                    let sql_stmt = format!(
                        "SELECT COUNT(*) FROM {} WHERE {} = $1 AND {} IS NULL",
                        setting.table, setting.guild_id, column.id
                    );

                    let query = sqlx::query(sql_stmt.as_str()).bind(guild_id.to_string());

                    let row = query
                        .fetch_one(&mut *tx)
                        .await
                        .map_err(|e| SettingsError::Generic {
                            message: e.to_string(),
                            src: format!("settings_create [unique check (null value), query.fetch_one] for column `{}`", column.id),
                            typ: "internal".to_string(),
                        })?;

                    let count = row.try_get::<i64, _>(0)
                        .map_err(|e| SettingsError::Generic {
                            message: e.to_string(),
                            src: format!("settings_create [unique check (null value), row.try_get] for column `{}`", column.id),
                            typ: "internal".to_string(),
                        })?;

                    if count > 0 {
                        return Err(SettingsError::RowExists {
                            column_id: column.id.to_string(),
                            count,
                        });
                    }
                }
                _ => {
                    let sql_stmt = format!(
                        "SELECT COUNT(*) FROM {} WHERE {} = $1 AND {} = $2",
                        setting.table, setting.guild_id, column.id
                    );

                    let mut query = sqlx::query(sql_stmt.as_str()).bind(guild_id.to_string());

                    query = _query_bind_value(query, value.clone(), &column.column_type);

                    let row =
                        query
                            .fetch_one(&mut *tx)
                            .await
                            .map_err(|e| SettingsError::Generic {
                                message: e.to_string(),
                                src: format!(
                                "settings_create [unique check, query.fetch_one] for column `{}`",
                                column.id
                            ),
                                typ: "internal".to_string(),
                            })?;

                    let count = row
                        .try_get::<i64, _>(0)
                        .map_err(|e| SettingsError::Generic {
                            message: e.to_string(),
                            src: format!(
                                "settings_create [unique check, row.try_get] for column `{}`",
                                column.id
                            ),
                            typ: "internal".to_string(),
                        })?;

                    if count > 0 {
                        return Err(SettingsError::RowExists {
                            column_id: column.id.to_string(),
                            count,
                        });
                    }
                }
            }
        }
    }

    // Remove ignored columns now that the actions have been executed
    for col in setting.columns.iter() {
        if state.bypass_ignore_for.contains(col.id) {
            continue;
        }

        if col.ignored_for.contains(&OperationType::Create) {
            state.state.swap_remove(col.id);
        }
    }

    // Now insert all the columns_to_set into state
    // As we have removed the ignored columns, we can just directly insert the columns_to_set into the state
    for (column, value) in operation_specific.columns_to_set.iter() {
        let value = state.template_to_string(author, guild_id, value);
        state.state.insert(column.to_string(), value);
    }

    // Create the row
    // First create the $N's from the cols starting with 2 as 1 is the guild_id
    let mut n_params = "".to_string();
    let mut col_params = "".to_string();
    for (i, (col, _)) in state.get_public().iter().enumerate() {
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

    for (col, value) in state.get_public().iter() {
        // Get column type from schema for db query hinting
        let Some(column) = setting.columns.iter().find(|c| c.id == col) else {
            return Err(SettingsError::Generic {
                message: format!("Column `{}` not found in schema", col),
                src: "settings_create [column_type_let_else]".to_string(),
                typ: "internal".to_string(),
            });
        };

        query = _query_bind_value(query, value.clone(), &column.column_type);
    }

    // Execute the query
    let pkey_row = query
        .fetch_one(&mut *tx)
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

    // Commit the transaction
    tx.commit().await.map_err(|e| SettingsError::Generic {
        message: e.to_string(),
        src: "settings_create [tx.commit]".to_string(),
        typ: "internal".to_string(),
    })?;

    Ok(state)
}

/// Settings API: Update implementation
pub async fn settings_update(
    setting: &ConfigOption,
    cache_http: &botox::cache::CacheHttpImpl,
    pool: &sqlx::PgPool,
    guild_id: serenity::all::GuildId,
    author: serenity::all::UserId,
    fields: indexmap::IndexMap<String, Value>,
    permodule_executor: &dyn base_data::permodule::PermoduleFunctionExecutor,
) -> Result<State, SettingsError> {
    let Some(operation_specific) = setting.operations.get(&OperationType::Update) else {
        return Err(SettingsError::OperationNotSupported {
            operation: OperationType::Update,
        });
    };

    let mut fields = fields; // Make fields mutable, consuming the input

    // Ensure all columns exist in fields, note that we can ignore extra fields so this one single loop is enough
    let mut state: State = State::new();
    let mut unchanged_fields = Vec::new();
    let mut pkey = None;
    for column in setting.columns.iter() {
        // If the column is ignored for create, skip
        let value = {
            if column.ignored_for.contains(&OperationType::Update) {
                unchanged_fields.push(column.id.to_string()); // Ensure that ignored_for columns are still seen as unchanged
                Value::None
            } else {
                match fields.swap_remove(column.id) {
                    Some(val) => {
                        if matches!(val, Value::None) {
                            Value::None // If the value is None, then it should be treated as if omitted (null)
                        } else {
                            _validate_and_parse_value(
                                val,
                                &column.column_type,
                                column.id,
                                column.nullable,
                                true,
                            )
                            .await?
                        }
                    }
                    None => {
                        unchanged_fields.push(column.id.to_string());
                        Value::None
                    }
                }
            }
        };

        if column.id == setting.primary_key {
            pkey = Some((column, value.clone()));
        }

        // Insert the value into the state
        state.state.insert(column.id.to_string(), value);
    }

    drop(fields); // Drop fields to avoid accidental use of user data
    #[allow(unused_variables)]
    let fields = (); // Reset fields to avoid accidental use of user data

    // Get out the pkey and pkey_column data here as we need it for the rest of the update
    let Some((pkey_column, pkey)) = pkey else {
        return Err(SettingsError::MissingOrInvalidField {
            field: setting.primary_key.to_string(),
            src: "settings_update [pkey_let]".to_string(),
        });
    };

    let pkey = _validate_and_parse_value(
        pkey,
        &pkey_column.column_type,
        setting.primary_key,
        false,
        true,
    )
    .await?;

    // Start the transaction now that basic validation is done
    let mut tx = pool.begin().await.map_err(|e| SettingsError::Generic {
        message: e.to_string(),
        src: "settings_create [pool.begin]".to_string(),
        typ: "internal".to_string(),
    })?;

    // Now retrieve all the unchanged fields
    if !unchanged_fields.is_empty() {
        let sql_stmt = format!(
            "SELECT {} FROM {} WHERE {} = $1 AND {} = $2",
            unchanged_fields.join(", "),
            setting.table,
            setting.guild_id,
            setting.primary_key
        );

        let mut query = sqlx::query(sql_stmt.as_str()).bind(guild_id.to_string());

        query = _query_bind_value(query, pkey.clone(), &pkey_column.column_type);

        let row = query
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| SettingsError::Generic {
                message: e.to_string(),
                src: "settings_update [retrieve_unchanged, query.fetch_one]".to_string(),
                typ: "internal".to_string(),
            })?;

        for (i, col) in unchanged_fields.iter().enumerate() {
            let val = Value::from_sqlx(&row, i).map_err(|e| SettingsError::Generic {
                message: e.to_string(),
                src: "settings_update [retrieve_unchanged, Value::from_sqlx]".to_string(),
                typ: "internal".to_string(),
            })?;

            state.state.insert(col.to_string(), val);
        }
    }

    // Handle all the actual checks here, now that all validation and needed fetches are done
    for column in setting.columns.iter() {
        // Execute actions
        let actions = column
            .pre_checks
            .get(&OperationType::Update)
            .unwrap_or(&column.default_pre_checks);

        super::action_executor::execute_actions(
            &mut state,
            actions,
            cache_http,
            pool,
            author,
            guild_id,
            permodule_executor,
        )
        .await?;

        if column.ignored_for.contains(&OperationType::Update) {
            continue;
        }

        let Some(value) = state.state.get(column.id) else {
            return Err(SettingsError::Generic {
                message: format!(
                    "Column `{}` not found in state despite just being parsed",
                    column.id
                ),
                src: "settings_update [ext_checks]".to_string(),
                typ: "internal".to_string(),
            });
        };

        // Nullability checks should only happen if the column is not being intentionally ignored
        // Check if the column is nullable
        if !column.nullable && matches!(value, Value::None) {
            return Err(SettingsError::MissingOrInvalidField {
                field: column.id.to_string(),
                src: "settings_update [nullability check]".to_string(),
            });
        }

        // Handle cases of uniqueness
        //
        // ** Difference from create: We can't treat unique and primary key the same as the unique check must take into account the existing row **
        if column.unique {
            match value {
                Value::None => {
                    let sql_stmt = format!(
                        "SELECT COUNT(*) FROM {} WHERE {} = $1 AND {} IS NULL AND {} != $2",
                        setting.table, setting.guild_id, column.id, setting.primary_key
                    );

                    let mut query = sqlx::query(sql_stmt.as_str()).bind(guild_id.to_string());

                    query = _query_bind_value(query, pkey.clone(), &pkey_column.column_type);

                    let row = query
                        .fetch_one(&mut *tx)
                        .await
                        .map_err(|e| SettingsError::Generic {
                            message: e.to_string(),
                            src: format!("settings_update [unique check (null value), query.fetch_one] for column `{}`", column.id),
                            typ: "internal".to_string(),
                        })?;

                    let count = row.try_get::<i64, _>(0)
                        .map_err(|e| SettingsError::Generic {
                            message: e.to_string(),
                            src: format!("settings_update [unique check (null value), row.try_get] for column `{}`", column.id),
                            typ: "internal".to_string(),
                        })?;

                    if count > 0 {
                        return Err(SettingsError::RowExists {
                            column_id: column.id.to_string(),
                            count,
                        });
                    }
                }
                _ => {
                    let sql_stmt = format!(
                        "SELECT COUNT(*) FROM {} WHERE {} = $1 AND {} = $2 AND {} != $3",
                        setting.table, setting.guild_id, column.id, setting.primary_key
                    );

                    let mut query = sqlx::query(sql_stmt.as_str()).bind(guild_id.to_string());

                    query = _query_bind_value(query, value.clone(), &column.column_type);
                    query = _query_bind_value(query, pkey.clone(), &pkey_column.column_type);

                    let row =
                        query
                            .fetch_one(&mut *tx)
                            .await
                            .map_err(|e| SettingsError::Generic {
                                message: e.to_string(),
                                src: format!(
                                "settings_update [unique check, query.fetch_one] for column `{}`",
                                column.id
                            ),
                                typ: "internal".to_string(),
                            })?;

                    let count = row
                        .try_get::<i64, _>(0)
                        .map_err(|e| SettingsError::Generic {
                            message: e.to_string(),
                            src: format!(
                                "settings_update [unique check, row.try_get] for column `{}`",
                                column.id
                            ),
                            typ: "internal".to_string(),
                        })?;

                    if count > 0 {
                        return Err(SettingsError::RowExists {
                            column_id: column.id.to_string(),
                            count,
                        });
                    }
                }
            }
        }

        // Handle cases of primary key next
        // ** This is unique to updates **
        if column.id == setting.primary_key {
            match value {
                Value::None => {
                    let sql_stmt = format!(
                        "SELECT COUNT(*) FROM {} WHERE {} = $1 AND {} IS NULL",
                        setting.table, setting.guild_id, column.id
                    );

                    let query = sqlx::query(sql_stmt.as_str()).bind(guild_id.to_string());

                    let row = query
                        .fetch_one(&mut *tx)
                        .await
                        .map_err(|e| SettingsError::Generic {
                            message: e.to_string(),
                            src: format!("settings_update [unique check (null value), query.fetch_one] for column `{}`", column.id),
                            typ: "internal".to_string(),
                        })?;

                    let count = row.try_get::<i64, _>(0)
                        .map_err(|e| SettingsError::Generic {
                            message: e.to_string(),
                            src: format!("settings_update [unique check (null value), row.try_get] for column `{}`", column.id),
                            typ: "internal".to_string(),
                        })?;

                    if count == 0 {
                        return Err(SettingsError::RowDoesNotExist {
                            column_id: column.id.to_string(),
                        });
                    }
                }
                _ => {
                    let sql_stmt = format!(
                        "SELECT COUNT(*) FROM {} WHERE {} = $1 AND {} = $2",
                        setting.table, setting.guild_id, column.id
                    );

                    let mut query = sqlx::query(sql_stmt.as_str()).bind(guild_id.to_string());

                    query = _query_bind_value(query, value.clone(), &column.column_type);

                    let row =
                        query
                            .fetch_one(&mut *tx)
                            .await
                            .map_err(|e| SettingsError::Generic {
                                message: e.to_string(),
                                src: format!(
                                "settings_update [unique check, query.fetch_one] for column `{}`",
                                column.id
                            ),
                                typ: "internal".to_string(),
                            })?;

                    let count = row
                        .try_get::<i64, _>(0)
                        .map_err(|e| SettingsError::Generic {
                            message: e.to_string(),
                            src: format!(
                                "settings_update [unique check, row.try_get] for column `{}`",
                                column.id
                            ),
                            typ: "internal".to_string(),
                        })?;

                    if count == 0 {
                        return Err(SettingsError::RowDoesNotExist {
                            column_id: column.id.to_string(),
                        });
                    }
                }
            }
        }
    }

    // Remove ignored columns now that the actions have been executed
    for col in setting.columns.iter() {
        if col.secret.is_some() {
            state.state.swap_remove(col.id);
            continue; // Skip secret columns in update. **this applies to view and update only as create is creating a new object**
        }

        if state.bypass_ignore_for.contains(col.id) {
            continue;
        }

        if col.ignored_for.contains(&OperationType::Update) {
            state.state.swap_remove(col.id);
        }
    }

    // Now insert all the columns_to_set into state
    // As we have removed the ignored columns, we can just directly insert the columns_to_set into the state
    for (column, value) in operation_specific.columns_to_set.iter() {
        let value = state.template_to_string(author, guild_id, value);
        state.state.insert(column.to_string(), value);
    }

    // Create the row
    let mut col_params = "".to_string();
    for (i, (col, _)) in state.get_public().iter().enumerate() {
        col_params.push_str(&format!("{}=${},", col, i + 3));
    }

    // Remove the trailing comma
    col_params.pop();

    // Execute the SQL statement
    let sql_stmt = format!(
        "UPDATE {} SET {} WHERE {} = $1 AND {} = $2",
        setting.table, col_params, setting.guild_id, setting.primary_key
    );

    let mut query = sqlx::query(sql_stmt.as_str());

    // Bind the sql query arguments
    query = query.bind(guild_id.to_string());
    query = _query_bind_value(query, pkey.clone(), &pkey_column.column_type);

    for (col, value) in state.get_public().iter() {
        // Get column type from schema for db query hinting
        let Some(column) = setting.columns.iter().find(|c| c.id == col) else {
            return Err(SettingsError::Generic {
                message: format!("Column `{}` not found in schema", col),
                src: "settings_update [column_type_let_else]".to_string(),
                typ: "internal".to_string(),
            });
        };

        query = _query_bind_value(query, value.clone(), &column.column_type);
    }

    // Execute the query
    query
        .execute(&mut *tx)
        .await
        .map_err(|e| SettingsError::Generic {
            message: e.to_string(),
            src: "settings_update [query execute]".to_string(),
            typ: "internal".to_string(),
        })?;

    // Commit the transaction
    tx.commit().await.map_err(|e| SettingsError::Generic {
        message: e.to_string(),
        src: "settings_update [tx.commit]".to_string(),
        typ: "internal".to_string(),
    })?;

    Ok(state)
}

/// Settings API: Delete implementation
pub async fn settings_delete(
    setting: &ConfigOption,
    cache_http: &botox::cache::CacheHttpImpl,
    pool: &sqlx::PgPool,
    guild_id: serenity::all::GuildId,
    author: serenity::all::UserId,
    pkey: Value,
    permodule_executor: &dyn base_data::permodule::PermoduleFunctionExecutor,
) -> Result<State, SettingsError> {
    let Some(_operation_specific) = setting.operations.get(&OperationType::Delete) else {
        return Err(SettingsError::OperationNotSupported {
            operation: OperationType::Delete,
        });
    };

    let mut state = State::new();

    let Some(pkey_column) = setting.columns.iter().find(|c| c.id == setting.primary_key) else {
        return Err(SettingsError::Generic {
            message: "Primary key column not found".to_string(),
            src: "settings_update [pkey_column_let_else]".to_string(),
            typ: "internal".to_string(),
        });
    };

    let pkey = _validate_and_parse_value(
        pkey,
        &pkey_column.column_type,
        setting.primary_key,
        false,
        false,
    )
    .await?;

    let mut tx = pool.begin().await.map_err(|e| SettingsError::Generic {
        message: e.to_string(),
        src: "settings_delete [pool.begin]".to_string(),
        typ: "internal".to_string(),
    })?;

    // Fetch entire row to execute actions on before deleting
    let mut cols = Vec::new();
    let mut column_types = Vec::new();

    for col in setting.columns.iter() {
        if col.ignored_for.contains(&OperationType::Delete) {
            continue;
        }

        cols.push(col.id.to_string());
        column_types.push(&col.column_type);
    }

    if !cols.is_empty() {
        let sql_stmt = format!(
            "SELECT {} FROM {} WHERE {} = $1 AND {} = $2",
            cols.join(", "),
            setting.table,
            setting.guild_id,
            setting.primary_key
        );

        let mut query = sqlx::query(sql_stmt.as_str()).bind(guild_id.to_string());

        query = _query_bind_value(query, pkey.clone(), &pkey_column.column_type);

        let Some(row) =
            query
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| SettingsError::Generic {
                    message: e.to_string(),
                    src: "settings_delete [retrieve_unchanged, query.fetch_one]".to_string(),
                    typ: "internal".to_string(),
                })?
        else {
            return Err(SettingsError::RowDoesNotExist {
                column_id: setting.primary_key.to_string(),
            });
        };

        for (i, col) in cols.iter().enumerate() {
            let val = Value::from_sqlx(&row, i).map_err(|e| SettingsError::Generic {
                message: e.to_string(),
                src: "settings_delete [retrieve_unchanged, Value::from_sqlx]".to_string(),
                typ: "internal".to_string(),
            })?;

            // Validate the actual value, note that as this is a delete operation, we don't care about nullability
            let val = _validate_and_parse_value(val, column_types[i], col, true, false).await?;

            state.state.insert(col.to_string(), val);
        }
    }

    // Execute all actions
    for column in setting.columns.iter() {
        // Execute actions
        let actions = column
            .pre_checks
            .get(&OperationType::Delete)
            .unwrap_or(&column.default_pre_checks);

        super::action_executor::execute_actions(
            &mut state,
            actions,
            cache_http,
            pool,
            author,
            guild_id,
            permodule_executor,
        )
        .await?;
    }

    // Now delete the entire row, the ignored_for does not matter here as we are deleting the entire row
    let sql_stmt = format!(
        "DELETE FROM {} WHERE {} = $1 AND {} = $2",
        setting.table, setting.guild_id, setting.primary_key
    );

    let mut query = sqlx::query(sql_stmt.as_str());

    query = query.bind(guild_id.to_string());
    query = _query_bind_value(query, pkey, &pkey_column.column_type);

    let res = query
        .execute(&mut *tx)
        .await
        .map_err(|e| SettingsError::Generic {
            message: e.to_string(),
            src: "settings_delete [query execute]".to_string(),
            typ: "internal".to_string(),
        })?;

    if res.rows_affected() == 0 {
        return Err(SettingsError::RowDoesNotExist {
            column_id: setting.primary_key.to_string(),
        });
    }

    // Commit the transaction
    tx.commit().await.map_err(|e| SettingsError::Generic {
        message: e.to_string(),
        src: "settings_delete [tx.commit]".to_string(),
        typ: "internal".to_string(),
    })?;

    Ok(state)
}

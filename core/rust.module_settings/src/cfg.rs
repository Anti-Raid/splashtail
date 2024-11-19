use crate::types::HookContext;

use super::state::State;
use super::types::SettingsError;
use super::types::{
    ColumnType, ConfigOption, InnerColumnType, InnerColumnTypeStringKind, OperationType,
    SettingsData,
};
use splashcore_rs::value::Value;

/// Parse a value against the schema's column type
fn _parse_value(
    v: Value,
    column_type: &ColumnType,
    column_id: &str,
) -> Result<Value, SettingsError> {
    match column_type {
        ColumnType::Scalar { inner } => {
            // Special case: JSON columns can be any type
            if matches!(v, Value::List(_)) && !matches!(inner, InnerColumnType::Json { .. }) {
                return Err(SettingsError::SchemaTypeValidationError {
                    column: column_id.to_string(),
                    expected_type: "Scalar".to_string(),
                    got_type: "Array".to_string(),
                });
            }

            match inner {
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
                    Value::None => Ok(v),
                    _ => Err(SettingsError::SchemaTypeValidationError {
                        column: column_id.to_string(),
                        expected_type: "Uuid".to_string(),
                        got_type: format!("{:?}", v),
                    }),
                },
                InnerColumnType::String { kind, .. } => match v {
                    Value::String(ref s) => {
                        if s.is_empty() {
                            match kind {
                                InnerColumnTypeStringKind::Token { default_length } => {
                                    Ok(Value::String(botox::crypto::gen_random(*default_length)))
                                }
                                _ => Ok(Value::None),
                            }
                        } else {
                            Ok(v)
                        }
                    }
                    Value::Uuid(v) => Ok(Value::String(v.to_string())),
                    Value::None => match kind {
                        InnerColumnTypeStringKind::Token { default_length } => {
                            Ok(Value::String(botox::crypto::gen_random(*default_length)))
                        }
                        _ => Ok(v),
                    },
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
                    Value::None => Ok(v),
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
                    Value::None => Ok(v),
                    _ => Err(SettingsError::SchemaTypeValidationError {
                        column: column_id.to_string(),
                        expected_type: "TimestampTz".to_string(),
                        got_type: format!("{:?}", v),
                    }),
                },
                InnerColumnType::Interval {} => match v {
                    Value::String(s) => {
                        let dur =
                            splashcore_rs::utils::parse_duration_string_to_chrono_duration(&s)
                                .map_err(|e| SettingsError::SchemaCheckValidationError {
                                    column: column_id.to_string(),
                                    check: "interval_parse".to_string(),
                                    accepted_range: "Valid interval".to_string(),
                                    error: e.to_string(),
                                })?;

                        Ok(Value::Interval(dur))
                    }
                    Value::Integer(v) => {
                        let duration = chrono::Duration::seconds(v);
                        Ok(Value::Interval(duration))
                    }
                    Value::Interval(_) => Ok(v),
                    Value::None => Ok(v),
                    _ => Err(SettingsError::SchemaTypeValidationError {
                        column: column_id.to_string(),
                        expected_type: "Interval".to_string(),
                        got_type: format!("{:?}", v),
                    }),
                },
                InnerColumnType::Integer {} => match v {
                    Value::String(s) => {
                        if s.is_empty() {
                            Ok(Value::None)
                        } else {
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
                    }
                    Value::Integer(v) => Ok(Value::Integer(v)),
                    Value::None => Ok(v),
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
                    Value::None => Ok(v),
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

                        if final_value == 0 {
                            // Set the first value as the default value
                            let Some(fv) = values.values().next() else {
                                return Err(SettingsError::SchemaCheckValidationError {
                                    column: column_id.to_string(),
                                    check: "bitflag_default".to_string(),
                                    accepted_range: "Valid bitflag".to_string(),
                                    error: "No default value found".to_string(),
                                });
                            };

                            final_value = *fv;
                        }

                        Ok(Value::Integer(final_value))
                    }
                    Value::String(s) => {
                        if s.is_empty() {
                            // Set the first value as the default value
                            let Some(fv) = values.values().next() else {
                                return Err(SettingsError::SchemaCheckValidationError {
                                    column: column_id.to_string(),
                                    check: "bitflag_default".to_string(),
                                    accepted_range: "Valid bitflag".to_string(),
                                    error: "No default value found".to_string(),
                                });
                            };

                            Ok(Value::Integer(*fv))
                        } else {
                            let v = s.parse::<i64>().map_err(|e| {
                                SettingsError::SchemaCheckValidationError {
                                    column: column_id.to_string(),
                                    check: "bitflag_parse".to_string(),
                                    accepted_range: "Valid bitflag".to_string(),
                                    error: e.to_string(),
                                }
                            })?;

                            let mut final_value = 0;

                            // Set all the valid bits in final_value to ensure no unknown bits are being set
                            for (_, bit) in values.iter() {
                                if *bit & v == *bit {
                                    final_value |= *bit;
                                }
                            }

                            if final_value == 0 {
                                // Set the first value as the default value
                                let Some(fv) = values.values().next() else {
                                    return Err(SettingsError::SchemaCheckValidationError {
                                        column: column_id.to_string(),
                                        check: "bitflag_default".to_string(),
                                        accepted_range: "Valid bitflag".to_string(),
                                        error: "No default value found".to_string(),
                                    });
                                };

                                final_value = *fv;
                            }

                            Ok(Value::Integer(final_value))
                        }
                    }
                    Value::None => Ok(v),
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
                    Value::None => Ok(v),
                    _ => Err(SettingsError::SchemaTypeValidationError {
                        column: column_id.to_string(),
                        expected_type: "Boolean".to_string(),
                        got_type: format!("{:?}", v),
                    }),
                },
                InnerColumnType::Json { max_bytes } => {
                    // Convert back to json to get bytes
                    match v {
                        Value::String(s) => {
                            if s.as_bytes().len() > max_bytes.unwrap_or(0) {
                                return Err(SettingsError::SchemaCheckValidationError {
                                    column: column_id.to_string(),
                                    check: "json_max_bytes".to_string(),
                                    accepted_range: format!("<{}", max_bytes.unwrap_or(0)),
                                    error: format!(
                                        "s.as_bytes().len() > *max_bytes: {}",
                                        s.as_bytes().len()
                                    ),
                                });
                            }

                            let v: serde_json::Value = {
                                if !s.starts_with("[") && !s.starts_with("{") {
                                    serde_json::Value::String(s)
                                } else {
                                    serde_json::from_str(&s).map_err(|e| {
                                        SettingsError::SchemaCheckValidationError {
                                            column: column_id.to_string(),
                                            check: "json_parse".to_string(),
                                            accepted_range: "Valid JSON".to_string(),
                                            error: e.to_string(),
                                        }
                                    })?
                                }
                            };

                            Ok(Value::Json(v))
                        }
                        _ => {
                            let bytes = serde_json::to_string(&v.to_json()).map_err(|e| {
                                SettingsError::SchemaCheckValidationError {
                                    column: column_id.to_string(),
                                    check: "json_parse".to_string(),
                                    accepted_range: "Valid JSON".to_string(),
                                    error: e.to_string(),
                                }
                            })?;

                            if let Some(max_bytes) = max_bytes {
                                if bytes.as_bytes().len() > *max_bytes {
                                    return Err(SettingsError::SchemaCheckValidationError {
                                        column: column_id.to_string(),
                                        check: "json_max_bytes".to_string(),
                                        accepted_range: format!("<{}", max_bytes),
                                        error: format!(
                                            "json.as_bytes().len() > *max_bytes: {}",
                                            bytes.as_bytes().len()
                                        ),
                                    });
                                }
                            }

                            Ok(v)
                        }
                    }
                }
            }
        }
        ColumnType::Array { inner } => {
            if let InnerColumnType::Json { max_bytes } = inner {
                // Convert back to json to get bytes of the full payload as a whole
                let json = serde_json::to_string(&v.to_json()).map_err(|e| {
                    SettingsError::SchemaCheckValidationError {
                        column: column_id.to_string(),
                        check: "json_parse".to_string(),
                        accepted_range: "Valid JSON".to_string(),
                        error: e.to_string(),
                    }
                })?;

                if let Some(max_bytes) = max_bytes {
                    if json.as_bytes().len() > *max_bytes {
                        return Err(SettingsError::SchemaCheckValidationError {
                            column: column_id.to_string(),
                            check: "json_max_bytes".to_string(),
                            accepted_range: format!("<{}", max_bytes),
                            error: format!(
                                "json.as_bytes().len() > *max_bytes: {}",
                                json.as_bytes().len()
                            ),
                        });
                    }
                }
            }

            match v {
                Value::List(l) => {
                    let mut values: Vec<Value> = Vec::new();

                    let column_type = ColumnType::new_scalar(inner.clone());
                    for v in l {
                        let new_v = _parse_value(v, &column_type, column_id)?;

                        values.push(new_v);
                    }

                    Ok(Value::List(values))
                }
                Value::None => Ok(v),
                _ => Err(SettingsError::SchemaTypeValidationError {
                    column: column_id.to_string(),
                    expected_type: "Array".to_string(),
                    got_type: format!("{:?}", v),
                }),
            }
        }
    }
}

/// Validates the value against the schema's column type
///
/// NOTE: This may make HTTP/Discord API requests to parse values such as channels etc.
#[allow(dead_code)]
#[async_recursion::async_recursion]
#[allow(clippy::too_many_arguments)]
async fn _validate_value(
    v: Value,
    guild_id: serenity::all::GuildId,
    data: &SettingsData,
    column_type: &ColumnType,
    column_id: &str,
    is_nullable: bool,
) -> Result<Value, SettingsError> {
    let v = match column_type {
        ColumnType::Scalar { inner } => {
            // Special case: JSON columns can be any type
            if matches!(v, Value::List(_)) && !matches!(inner, InnerColumnType::Json { .. }) {
                return Err(SettingsError::SchemaTypeValidationError {
                    column: column_id.to_string(),
                    expected_type: "Scalar".to_string(),
                    got_type: "Array".to_string(),
                });
            }

            match inner {
                InnerColumnType::String {
                    min_length,
                    max_length,
                    allowed_values,
                    kind,
                } => {
                    match v {
                        Value::String(ref s) => {
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

                            if !allowed_values.is_empty() && !allowed_values.contains(&s) {
                                return Err(SettingsError::SchemaCheckValidationError {
                                    column: column_id.to_string(),
                                    check: "allowed_values".to_string(),
                                    accepted_range: format!("{:?}", allowed_values),
                                    error: "!allowed_values.is_empty() && !allowed_values.contains(&s.as_str())".to_string()
                                });
                            }

                            let parsed_value = match kind {
                                InnerColumnTypeStringKind::Normal => v,
                                InnerColumnTypeStringKind::Token { .. } => v, // Handled in parse_value
                                InnerColumnTypeStringKind::Textarea { .. } => v,
                                InnerColumnTypeStringKind::TemplateRef { .. } => {
                                    // Check that the template exists
                                    let count = sqlx::query!(
                                        "SELECT COUNT(*) FROM guild_templates WHERE guild_id = $1 AND name = $2",
                                        guild_id.to_string(),
                                        s
                                    )
                                    .fetch_one(&data.pool)
                                    .await
                                    .map_err(|e| SettingsError::SchemaCheckValidationError {
                                        column: column_id.to_string(),
                                        check: "template_ref".to_string(),
                                        accepted_range: "Valid template name".to_string(),
                                        error: e.to_string(),
                                    })?;

                                    if count.count.unwrap_or(0) == 0 {
                                        return Err(SettingsError::SchemaCheckValidationError {
                                            column: column_id.to_string(),
                                            check: "template_ref".to_string(),
                                            accepted_range: "Valid template name".to_string(),
                                            error: "Template not found".to_string(),
                                        });
                                    }

                                    v
                                }
                                InnerColumnTypeStringKind::KittycatPermission => v, // All kittycat permissions are valid
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

                                    v
                                }
                                InnerColumnTypeStringKind::Role => {
                                    // Try parsing to a RoleId
                                    if let Err(err) = s.parse::<serenity::all::RoleId>() {
                                        return Err(SettingsError::SchemaCheckValidationError {
                                            column: column_id.to_string(),
                                            check: "snowflake_parse".to_string(),
                                            accepted_range: "Valid role id".to_string(),
                                            error: err.to_string(),
                                        });
                                    }

                                    v
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

                                    v
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

                                    v
                                }
                                InnerColumnTypeStringKind::Modifier => {
                                    splashcore_rs::modifier::Modifier::from_repr(s).map_err(
                                        |e| SettingsError::SchemaCheckValidationError {
                                            column: column_id.to_string(),
                                            check: "modifier_parse".to_string(),
                                            accepted_range: "Valid modifier".to_string(),
                                            error: e.to_string(),
                                        },
                                    )?;

                                    v
                                }
                            };
                            Ok(parsed_value)
                        }
                        Value::None => Ok(v),
                        _ => Err(SettingsError::SchemaTypeValidationError {
                            column: column_id.to_string(),
                            expected_type: "String".to_string(),
                            got_type: format!("{:?}", v),
                        }),
                    }
                }
                _ => Ok(v),
            }
        }
        ColumnType::Array { inner } => match v {
            Value::List(l) => {
                let mut values: Vec<Value> = Vec::new();

                let column_type = ColumnType::new_scalar(inner.clone());
                for v in l {
                    let new_v =
                        _validate_value(v, guild_id, data, &column_type, column_id, is_nullable)
                            .await?;

                    values.push(new_v);
                }

                Ok(Value::List(values))
            }
            Value::None => Ok(v),
            _ => Err(SettingsError::SchemaTypeValidationError {
                column: column_id.to_string(),
                expected_type: "Array".to_string(),
                got_type: format!("{:?}", v),
            }),
        },
    }?;

    if matches!(v, Value::None) && !is_nullable {
        return Err(SettingsError::SchemaNullValueValidationError {
            column: column_id.to_string(),
        });
    }

    Ok(v)
}

/// Validate keys for basic sanity
///
/// This *MUST* be called at the start of any operation to ensure that the keys are valid and safe
pub fn validate_keys(
    _setting: &ConfigOption,
    fields: &indexmap::IndexMap<String, Value>,
) -> Result<(), SettingsError> {
    const MAX_FIELDS: usize = 50;
    if fields.len() > MAX_FIELDS {
        return Err(SettingsError::Generic {
            message: format!("Too many fields: {}", fields.len()),
            src: "settings_common#validate_keys".to_string(),
            typ: "internal".to_string(),
        });
    }

    Ok(())
}

/// Settings API: View implementation
pub async fn settings_view(
    setting: &ConfigOption,
    data: &SettingsData,
    guild_id: serenity::all::GuildId,
    author: serenity::all::UserId,
    fields: indexmap::IndexMap<String, Value>, // The filters to apply
) -> Result<Vec<State>, SettingsError> {
    if !setting.supported_operations.contains(&OperationType::View) {
        return Err(SettingsError::OperationNotSupported {
            operation: OperationType::View,
        });
    }

    // WARNING: The ``validate_keys`` function call here should never be omitted, add back at once if you see this message without the function call
    validate_keys(setting, &fields)?;

    let Some(ref executor) = setting.executor.0 else {
        return Ok(Vec::new());
    };

    let states = executor
        .view(HookContext {
            guild_id,
            author,
            data,
        })
        .await
        .map_err(|e| SettingsError::Generic {
            message: e.to_string(),
            src: "settings_view".to_string(),
            typ: "internal".to_string(),
        })?;

    let mut values: Vec<State> = Vec::new();

    for mut state in states {
        // We know that the columns are in the same order as the row
        for col in setting.columns.iter() {
            let mut val = state.state.swap_remove(&col.id).unwrap_or(Value::None);

            // Validate the value. returning the parsed value
            val = _parse_value(val, &col.column_type, &col.id)?;

            // Reinsert
            state.state.insert(col.id.to_string(), val);
        }

        // Remove ignored columns + secret columns now that the actions have been executed
        for col in setting.columns.iter() {
            if col.secret {
                state.state.swap_remove(&col.id);
                continue; // Skip secret columns in view. **this applies to view and update only as create is creating a new object**
            }

            if state.bypass_ignore_for.contains(&col.id) {
                continue;
            }

            if col.ignored_for.contains(&OperationType::View) {
                state.state.swap_remove(&col.id);
            }
        }

        values.push(state);
    }

    Ok(values)
}

/// Settings API: Save implementation
pub async fn settings_save(
    setting: &ConfigOption,
    data: &SettingsData,
    guild_id: serenity::all::GuildId,
    author: serenity::all::UserId,
    fields: indexmap::IndexMap<String, Value>,
) -> Result<State, SettingsError> {
    if !setting.supported_operations.contains(&OperationType::Save) {
        return Err(SettingsError::OperationNotSupported {
            operation: OperationType::Save,
        });
    };

    // WARNING: The ``validate_keys`` function call here should never be omitted, add back at once if you see this message without the function call
    validate_keys(setting, &fields)?;

    let mut fields = fields; // Make fields mutable, consuming the input

    // Ensure all columns exist in fields, note that we can ignore extra fields so this one single loop is enough
    let mut state: State = State::new_with_special_variables(author, guild_id);
    for column in setting.columns.iter() {
        // If the column is ignored for create, skip
        // If the column is a secret column, then ensure we set it to something random as this is a create operation
        let value = {
            if column.ignored_for.contains(&OperationType::Save) {
                _parse_value(Value::None, &column.column_type, &column.id)?
            } else {
                // Get the value
                let val = fields.swap_remove(&column.id).unwrap_or(Value::None);

                let parsed_value = _parse_value(val, &column.column_type, &column.id)?;

                // Validate and parse the value
                _validate_value(
                    parsed_value,
                    guild_id,
                    data,
                    &column.column_type,
                    &column.id,
                    column.nullable,
                )
                .await?
            }
        };

        // Insert the value into the state
        state.state.insert(column.id.to_string(), value);
    }

    drop(fields); // Drop fields to avoid accidental use of user data
    #[allow(unused_variables)]
    let fields = (); // Reset fields to avoid accidental use of user data

    // Now execute all actions and handle null checks
    for column in setting.columns.iter() {
        // Checks should only happen if the column is not being intentionally ignored
        if column.ignored_for.contains(&OperationType::Save) {
            continue;
        }

        let Some(value) = state.state.get(&column.id) else {
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
    }

    // Remove ignored columns now that the actions have been executed
    for col in setting.columns.iter() {
        if state.bypass_ignore_for.contains(&col.id) {
            continue;
        }

        if col.ignored_for.contains(&OperationType::Save) {
            state.state.swap_remove(&col.id);
        }
    }

    // Create the row
    let Some(ref executor) = setting.executor.0 else {
        return Err(SettingsError::Generic {
            message: "No executor found".to_string(),
            src: "settings_save".to_string(),
            typ: "internal".to_string(),
        });
    };

    let new_state = executor
        .save(
            HookContext {
                guild_id,
                author,
                data,
            },
            &mut state,
        )
        .await
        .map_err(|e| SettingsError::Generic {
            message: e.to_string(),
            src: "settings_save".to_string(),
            typ: "internal".to_string(),
        })?;

    Ok(new_state)
}

/// Settings API: Delete implementation
#[allow(clippy::too_many_arguments)]
pub async fn settings_delete(
    setting: &ConfigOption,
    data: &SettingsData,
    guild_id: serenity::all::GuildId,
    author: serenity::all::UserId,
    pkey: Value,
) -> Result<State, SettingsError> {
    if !setting
        .supported_operations
        .contains(&OperationType::Delete)
    {
        return Err(SettingsError::OperationNotSupported {
            operation: OperationType::Delete,
        });
    }

    let Some(pkey_column) = setting.columns.iter().find(|c| c.id == setting.primary_key) else {
        return Err(SettingsError::Generic {
            message: "Primary key column not found".to_string(),
            src: "settings_update [pkey_column_let_else]".to_string(),
            typ: "internal".to_string(),
        });
    };

    let pkey = _parse_value(pkey, &pkey_column.column_type, &setting.primary_key)?;

    // Create the row
    let Some(ref executor) = setting.executor.0 else {
        return Err(SettingsError::Generic {
            message: "No executor found".to_string(),
            src: "settings_save".to_string(),
            typ: "internal".to_string(),
        });
    };

    let state = executor
        .delete(
            HookContext {
                guild_id,
                author,
                data,
            },
            pkey,
        )
        .await
        .map_err(|e| SettingsError::Generic {
            message: e.to_string(),
            src: "settings_delete".to_string(),
            typ: "internal".to_string(),
        })?;

    Ok(state)
}

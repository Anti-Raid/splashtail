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
    state: &State,
    column_type: &ColumnType,
    column_id: &str,
) -> Result<Value, SettingsError> {
    match column_type {
        ColumnType::Scalar { column_type } => {
            // Special case: JSON columns can be any type
            if matches!(v, Value::List(_)) && !matches!(column_type, InnerColumnType::Json {}) {
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
                InnerColumnType::Json {} => Ok(v),
            }
        }
        ColumnType::Array { inner } => match v {
            Value::List(l) => {
                let mut values: Vec<Value> = Vec::new();

                let column_type = ColumnType::new_scalar(inner.clone());
                for v in l {
                    let new_v = _parse_value(v, state, &column_type, column_id)?;

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
        ColumnType::Dynamic { clauses } => {
            for clause in clauses {
                let value = state.template_to_string(clause.field);

                if value == clause.value {
                    // We got the kind
                    return _parse_value(v, state, &clause.column_type, column_id);
                }
            }

            Err(SettingsError::SchemaCheckValidationError {
                column: column_id.to_string(),
                check: "dynamic_clause".to_string(),
                accepted_range: "Valid dynamic clause".to_string(),
                error: "No valid dynamic clause matched".to_string(),
            })
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
    state: &State,
    guild_id: serenity::all::GuildId,
    data: &SettingsData,
    column_type: &ColumnType,
    column_id: &str,
    is_nullable: bool,
) -> Result<Value, SettingsError> {
    let v = match column_type {
        ColumnType::Scalar { column_type } => {
            // Special case: JSON columns can be any type
            if matches!(v, Value::List(_)) && !matches!(column_type, InnerColumnType::Json {}) {
                return Err(SettingsError::SchemaTypeValidationError {
                    column: column_id.to_string(),
                    expected_type: "Scalar".to_string(),
                    got_type: "Array".to_string(),
                });
            }

            match column_type {
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

                            if !allowed_values.is_empty() && !allowed_values.contains(&s.as_str()) {
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
                                InnerColumnTypeStringKind::Textarea => v,
                                InnerColumnTypeStringKind::Template { .. } => {
                                    let compiled = templating::compile_template(
                                        guild_id,
                                        s,
                                        data.pool.clone(),
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
                                InnerColumnTypeStringKind::Channel {
                                    allowed_types,
                                    needed_bot_permissions,
                                } => {
                                    // Try parsing to a ChannelId
                                    let channel_id = s
                                        .parse::<serenity::all::ChannelId>()
                                        .map_err(|e| SettingsError::SchemaCheckValidationError {
                                            column: column_id.to_string(),
                                            check: "snowflake_parse".to_string(),
                                            accepted_range: "Valid channel id".to_string(),
                                            error: e.to_string(),
                                        })?;

                                    // Get the channel
                                    let channel = proxy_support::channel(
                                        &data.cache_http,
                                        &data.reqwest,
                                        Some(guild_id),
                                        channel_id,
                                    )
                                    .await
                                    .map_err(|e| SettingsError::SchemaCheckValidationError {
                                        column: column_id.to_string(),
                                        check: "channel_get".to_string(),
                                        accepted_range: "Valid channel id".to_string(),
                                        error: e.to_string(),
                                    })?;

                                    let Some(channel) = channel else {
                                        return Err(SettingsError::SchemaCheckValidationError {
                                            column: column_id.to_string(),
                                            check: "channel_get".to_string(),
                                            accepted_range: "Valid channel id".to_string(),
                                            error: "Channel not found".to_string(),
                                        });
                                    };

                                    if !allowed_types.is_empty() {
                                        match channel {
                                            serenity::all::Channel::Guild(gc) => {
                                                if !allowed_types.contains(&gc.kind) {
                                                    return Err(
                                                        SettingsError::SchemaCheckValidationError {
                                                            column: column_id.to_string(),
                                                            check: "channel_type".to_string(),
                                                            accepted_range: "Text channel"
                                                                .to_string(),
                                                            error: format!(
                                                                "Channel type is not text: {:?}",
                                                                gc.kind
                                                            ),
                                                        },
                                                    );
                                                }

                                                if gc.guild_id != guild_id {
                                                    return Err(SettingsError::SchemaCheckValidationError {
                                                    column: column_id.to_string(),
                                                    check: "channel_guild".to_string(),
                                                    accepted_range: "Valid channel id".to_string(),
                                                    error: "Channel is not in the guild specified".to_string(),
                                                });
                                                }

                                                if !needed_bot_permissions.is_empty() {
                                                    let perms = gc.permissions_for_user(&data.cache_http.cache, data.cache_http.cache.current_user().id).map_err(|e| SettingsError::SchemaCheckValidationError {
                                                        column: column_id.to_string(),
                                                        check: "channel_perms".to_string(),
                                                        accepted_range: "Valid channel id".to_string(),
                                                        error: e.to_string(),
                                                    })?;

                                                    for perm in needed_bot_permissions.iter() {
                                                        if !perms.contains(perm) {
                                                            return Err(SettingsError::SchemaCheckValidationError {
                                                                column: column_id.to_string(),
                                                                check: "channel_perms".to_string(),
                                                                accepted_range: "Valid channel id".to_string(),
                                                                error: format!("Missing permission: {}", perm),
                                                            });
                                                        }
                                                    }
                                                }
                                            }
                                            serenity::all::Channel::Private(pc) => {
                                                if !allowed_types.contains(&pc.kind) {
                                                    return Err(
                                                        SettingsError::SchemaCheckValidationError {
                                                            column: column_id.to_string(),
                                                            check: "channel_type".to_string(),
                                                            accepted_range: "Text channel"
                                                                .to_string(),
                                                            error: format!(
                                                                "Channel type is not text: {:?}",
                                                                pc.kind
                                                            ),
                                                        },
                                                    );
                                                }
                                            }
                                            _ => {
                                                return Err(
                                                    SettingsError::SchemaCheckValidationError {
                                                        column: column_id.to_string(),
                                                        check: "channel_type".to_string(),
                                                        accepted_range: "Valid channel".to_string(),
                                                        error: "Channel type is unknown"
                                                            .to_string(),
                                                    },
                                                );
                                            }
                                        }
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
                    let new_v = _validate_value(
                        v,
                        state,
                        guild_id,
                        data,
                        &column_type,
                        column_id,
                        is_nullable,
                    )
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
        ColumnType::Dynamic { clauses } => {
            for clause in clauses {
                let value = state.template_to_string(clause.field);

                if value == clause.value {
                    // We got the kind
                    return _validate_value(
                        v,
                        state,
                        guild_id,
                        data,
                        &clause.column_type,
                        column_id,
                        is_nullable,
                    )
                    .await;
                }
            }

            Err(SettingsError::SchemaCheckValidationError {
                column: column_id.to_string(),
                check: "dynamic_clause".to_string(),
                accepted_range: "Valid dynamic clause".to_string(),
                error: "No valid dynamic clause matched".to_string(),
            })
        }
    }?;

    if matches!(v, Value::None) && !is_nullable {
        return Err(SettingsError::SchemaNullValueValidationError {
            column: column_id.to_string(),
        });
    }

    Ok(v)
}

/// Returns the common filters for a given operation type
fn common_filters(
    setting: &ConfigOption,
    operation_type: OperationType,
    base_state: &State,
) -> indexmap::IndexMap<String, splashcore_rs::value::Value> {
    let common_filters_unparsed = setting
        .common_filters
        .get(&operation_type)
        .unwrap_or(&setting.default_common_filters);

    let mut common_filters = indexmap::IndexMap::new();

    for (key, value) in common_filters_unparsed.iter() {
        let value = base_state.template_to_string(value);
        common_filters.insert(key.to_string(), value);
    }

    common_filters
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
    let Some(operation_specific) = setting.operations.get(&OperationType::View) else {
        return Err(SettingsError::OperationNotSupported {
            operation: OperationType::View,
        });
    };

    // WARNING: The ``validate_keys`` function call here should never be omitted, add back at once if you see this message without the function call
    validate_keys(setting, &fields)?;

    let mut fields = fields; // Make fields mutable, consuming the input

    // Ensure limit is good
    let mut use_limit = setting.max_return;
    if let Some(Value::Integer(limit)) = fields.get("__limit") {
        use_limit = std::cmp::min(*limit, use_limit);
    }
    fields.insert("__limit".to_string(), Value::Integer(use_limit));

    let mut data_store = setting
        .data_store
        .create(
            setting,
            guild_id,
            author,
            data,
            common_filters(
                setting,
                OperationType::View,
                &super::state::State::new_with_special_variables(author, guild_id),
            ),
        )
        .await?;

    if let Some(Value::Boolean(true)) = fields.get("__count") {
        // We only need to count the number of rows
        fields.shift_remove("__limit");
        fields.shift_remove("__count");

        let count = data_store.matching_entry_count(fields).await?;

        let count = count.try_into().map_err(|e| SettingsError::Generic {
            message: format!("Count too large: {:?}", e),
            src: "settings_view".to_string(),
            typ: "internal".to_string(),
        })?;

        let mut state = super::state::State::new();

        state
            .state
            .insert("count".to_string(), Value::Integer(count));

        return Ok(vec![state]);
    }

    let cols = setting
        .columns
        .iter()
        .map(|c| c.id.to_string())
        .collect::<Vec<String>>();

    let states = data_store.fetch_all(&cols, fields).await?;

    if states.is_empty() {
        return Ok(Vec::new());
    }

    let mut values: Vec<State> = Vec::new();

    for mut state in states {
        // We know that the columns are in the same order as the row
        for col in setting.columns.iter() {
            let mut val = state.state.swap_remove(col.id).unwrap_or(Value::None);

            // Validate the value. returning the parsed value
            val = _parse_value(val, &state, &col.column_type, col.id)?;

            // Reinsert
            state.state.insert(col.id.to_string(), val);
        }

        // Run validators

        setting
            .validator
            .validate(
                super::types::HookContext {
                    author,
                    guild_id,
                    operation_type: OperationType::View,
                    data_store: &mut *data_store,
                    data,
                    unchanged_fields: vec![],
                },
                &mut state,
            )
            .await?;

        // Get out the pkey and pkey_column data here as we need it for the rest of the update
        let Some(pkey) = state.state.get(setting.primary_key) else {
            return Err(SettingsError::MissingOrInvalidField {
                field: setting.primary_key.to_string(),
                src: "settings_update [pkey_let]".to_string(),
            });
        };

        // Apply columns_to_set in operation specific data if there are columns to set
        if !operation_specific.columns_to_set.is_empty() {
            let filters = indexmap::indexmap! {
                setting.primary_key.to_string() => pkey.clone(),
            };
            let mut update = indexmap::IndexMap::new();

            for (col, value) in operation_specific.columns_to_set.iter() {
                let value = state.template_to_string(value);

                // Add directly to state
                state.state.insert(col.to_string(), value.clone());
                update.insert(col.to_string(), value);
            }

            data_store.update_matching_entries(filters, update).await?;
        }

        // Remove ignored columns + secret columns now that the actions have been executed
        for col in setting.columns.iter() {
            if col.secret {
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

        setting
            .post_action
            .post_action(
                super::types::HookContext {
                    author,
                    guild_id,
                    operation_type: OperationType::View,
                    data_store: &mut *data_store,
                    data,
                    unchanged_fields: vec![],
                },
                &mut state,
            )
            .await?;

        values.push(state);
    }

    Ok(values)
}

/// Settings API: Create implementation
pub async fn settings_create(
    setting: &ConfigOption,
    data: &SettingsData,
    guild_id: serenity::all::GuildId,
    author: serenity::all::UserId,
    fields: indexmap::IndexMap<String, Value>,
) -> Result<State, SettingsError> {
    let Some(operation_specific) = setting.operations.get(&OperationType::Create) else {
        return Err(SettingsError::OperationNotSupported {
            operation: OperationType::Create,
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
            if column.ignored_for.contains(&OperationType::Create) {
                _parse_value(Value::None, &state, &column.column_type, column.id)?
            } else {
                // Get the value
                let val = fields.swap_remove(column.id).unwrap_or(Value::None);

                let parsed_value = _parse_value(val, &state, &column.column_type, column.id)?;

                // Validate and parse the value
                _validate_value(
                    parsed_value,
                    &state,
                    guild_id,
                    data,
                    &column.column_type,
                    column.id,
                    column.nullable,
                )
                .await?
            }
        };

        // Insert the value into the state
        state.state.insert(
            column.id.to_string(),
            match value {
                Value::None => {
                    // Check for default
                    if let Some(default) = &column.default {
                        (default)(false)
                    } else {
                        value
                    }
                }
                _ => value,
            },
        );
    }

    drop(fields); // Drop fields to avoid accidental use of user data
    #[allow(unused_variables)]
    let fields = (); // Reset fields to avoid accidental use of user data

    // Start the transaction now that basic validation is done
    let mut data_store = setting
        .data_store
        .create(
            setting,
            guild_id,
            author,
            data,
            common_filters(setting, OperationType::Create, &state),
        )
        .await?;

    data_store.start_transaction().await?;

    // Get all ids we currently have to check max_entries and uniqueness of the primary key in one shot
    let ids = data_store
        .fetch_all(
            &[setting.primary_key.to_string()],
            indexmap::IndexMap::new(),
        )
        .await?;

    if let Some(max_entries) = setting.max_entries {
        if ids.len() > max_entries {
            return Err(SettingsError::MaximumCountReached {
                max: max_entries,
                current: ids.len(),
            });
        }
    }

    for id in ids.iter() {
        let id = id.state.get(setting.primary_key).unwrap_or(&Value::None);
        // Check if the pkey is unique
        if state.state.get(setting.primary_key) == Some(id) {
            return Err(SettingsError::RowExists {
                column_id: setting.primary_key.to_string(),
                count: 1,
            });
        }
    }

    drop(ids); // Drop ids as it is no longer needed

    // Now execute all actions and handle null/unique/pkey checks
    for column in setting.columns.iter() {
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
            let count = data_store
                .matching_entry_count(indexmap::indexmap! {
                    column.id.to_string() => value.clone()
                })
                .await?;

            if count > 0 {
                return Err(SettingsError::RowExists {
                    column_id: column.id.to_string(),
                    count: count.try_into().unwrap_or(i64::MAX),
                });
            }
        }
    }

    // Run validator
    setting
        .validator
        .validate(
            super::types::HookContext {
                author,
                guild_id,
                operation_type: OperationType::Create,
                data_store: &mut *data_store,
                data,
                unchanged_fields: vec![],
            },
            &mut state,
        )
        .await?;

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
        let value = state.template_to_string(value);
        state.state.insert(column.to_string(), value);
    }

    // Create the row
    let mut new_state = data_store.create_entry(state.get_public()).await?;

    // Insert any internal columns
    for (key, value) in state
        .state
        .into_iter()
        .filter(|(k, _)| k.starts_with(super::state::INTERNAL_KEY))
    {
        new_state.state.insert(key, value);
    }

    // Commit the transaction
    data_store.commit().await?;

    // Execute post actions
    setting
        .post_action
        .post_action(
            super::types::HookContext {
                author,
                guild_id,
                operation_type: OperationType::Create,
                data_store: &mut *data_store,
                data,
                unchanged_fields: vec![],
            },
            &mut new_state,
        )
        .await?;

    Ok(new_state)
}

/// Settings API: Update implementation
pub async fn settings_update(
    setting: &ConfigOption,
    data: &SettingsData,
    guild_id: serenity::all::GuildId,
    author: serenity::all::UserId,
    fields: indexmap::IndexMap<String, Value>,
) -> Result<State, SettingsError> {
    let Some(operation_specific) = setting.operations.get(&OperationType::Update) else {
        return Err(SettingsError::OperationNotSupported {
            operation: OperationType::Update,
        });
    };

    // WARNING: The ``validate_keys`` function call here should never be omitted, add back at once if you see this message without the function call
    validate_keys(setting, &fields)?;

    let mut fields = fields; // Make fields mutable, consuming the input

    // Ensure all columns exist in fields, note that we can ignore extra fields so this one single loop is enough
    let mut state: State = State::new_with_special_variables(author, guild_id);
    let mut unchanged_fields = indexmap::IndexSet::new();
    let mut pkey = None;
    for column in setting.columns.iter() {
        // If the column is ignored for update, skip
        if column.ignored_for.contains(&OperationType::Update) && column.id != setting.primary_key {
            if !column.secret {
                unchanged_fields.insert(column.id.to_string()); // Ensure that ignored_for columns are still seen as unchanged but only if not secret
            }
        } else {
            match fields.swap_remove(column.id) {
                Some(val) => {
                    let parsed_value = _parse_value(val, &state, &column.column_type, column.id)?;

                    let parsed_value = _validate_value(
                        parsed_value,
                        &state,
                        guild_id,
                        data,
                        &column.column_type,
                        column.id,
                        column.nullable,
                    )
                    .await?;

                    if column.id == setting.primary_key {
                        pkey = Some((column, parsed_value.clone()));
                    }

                    state.state.insert(column.id.to_string(), parsed_value);
                }
                None => {
                    if !column.secret {
                        unchanged_fields.insert(column.id.to_string()); // Don't retrieve the value if it's a secret column
                    }
                }
            }
        }
    }

    drop(fields); // Drop fields to avoid accidental use of user data
    #[allow(unused_variables)]
    let fields = (); // Reset fields to avoid accidental use of user data

    // Get out the pkey and pkey_column data here as we need it for the rest of the update
    let Some((_pkey_column, pkey)) = pkey else {
        return Err(SettingsError::MissingOrInvalidField {
            field: setting.primary_key.to_string(),
            src: "settings_update [pkey_let]".to_string(),
        });
    };

    // PKEY should already have passed the validation checks
    if matches!(pkey, Value::None) {
        return Err(SettingsError::MissingOrInvalidField {
            field: setting.primary_key.to_string(),
            src: "settings_update [pkey_none]".to_string(),
        });
    }

    let mut data_store = setting
        .data_store
        .create(
            setting,
            guild_id,
            author,
            data,
            common_filters(setting, OperationType::Update, &state),
        )
        .await?;

    // Start the transaction now that basic validation is done
    data_store.start_transaction().await?;

    // Now retrieve all the unchanged fields
    if !unchanged_fields.is_empty() {
        let mut data = data_store
            .fetch_all(
                &unchanged_fields
                    .iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<String>>(),
                indexmap::indexmap! {
                    setting.primary_key.to_string() => pkey.clone(),
                },
            )
            .await?;

        if data.is_empty() {
            return Err(SettingsError::RowDoesNotExist {
                column_id: setting.primary_key.to_string(),
            });
        }

        let unchanged_state = data.pop().unwrap(); // We know there is only one row

        for (k, v) in unchanged_state.state.into_iter() {
            state.state.insert(k.to_string(), v);
        }
    }

    // Handle all the actual checks here, now that all validation and needed fetches are done
    for column in setting.columns.iter() {
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
            if unchanged_fields.contains(&column.id.to_string()) {
                continue; // Skip uniqueness check if the field is unchanged
            }

            let ids = data_store
                .fetch_all(
                    &[setting.primary_key.to_string()],
                    indexmap::indexmap! {
                        column.id.to_string() => value.clone(),
                    },
                )
                .await?;

            let ids = ids
                .into_iter()
                .filter(|id| {
                    let id = id.state.get(column.id).unwrap_or(&Value::None);
                    id != &pkey
                })
                .collect::<Vec<State>>();

            if !ids.is_empty() {
                return Err(SettingsError::RowExists {
                    column_id: column.id.to_string(),
                    count: ids.len().try_into().unwrap_or(i64::MAX),
                });
            }
        }

        // Handle cases of primary key next
        // ** This is unique to updates **
        if column.id == setting.primary_key {
            let count = data_store
                .matching_entry_count(indexmap::indexmap! {
                    column.id.to_string() => value.clone(),
                })
                .await?;

            if count == 0 {
                return Err(SettingsError::RowDoesNotExist {
                    column_id: column.id.to_string(),
                });
            }
        }
    }

    // Run validator
    setting
        .validator
        .validate(
            super::types::HookContext {
                author,
                guild_id,
                operation_type: OperationType::Update,
                data_store: &mut *data_store,
                data,
                unchanged_fields: unchanged_fields.iter().map(|f| f.to_string()).collect(),
            },
            &mut state,
        )
        .await?;

    // Remove ignored columns now that the actions have been executed
    //
    // Note that we cannot mutate state here
    let mut columns_to_set = State::from_indexmap(state.get_public()); // Start with current public state
    for col in setting.columns.iter() {
        if state.bypass_ignore_for.contains(col.id) {
            continue;
        }

        if col.ignored_for.contains(&OperationType::Update) {
            columns_to_set.state.swap_remove(col.id);
        }
    }

    // Now insert all the columns_to_set into state
    // As we have removed the ignored columns, we can just directly insert the columns_to_set into the state
    for (column, value) in operation_specific.columns_to_set.iter() {
        let value = state.template_to_string(value);
        state.state.insert(column.to_string(), value.clone()); // Ensure its in returned state
        columns_to_set.state.insert(column.to_string(), value); // And in the columns to set
    }

    // Create the row
    data_store
        .update_matching_entries(
            indexmap::indexmap! {
                setting.primary_key.to_string() => pkey.clone(),
            },
            columns_to_set.state,
        )
        .await?;

    // Commit the transaction
    data_store.commit().await?;

    // Execute post actions
    setting
        .post_action
        .post_action(
            super::types::HookContext {
                author,
                guild_id,
                operation_type: OperationType::Update,
                data_store: &mut *data_store,
                data,
                unchanged_fields: unchanged_fields.iter().map(|f| f.to_string()).collect(),
            },
            &mut state,
        )
        .await?;

    Ok(state)
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
    let Some(_operation_specific) = setting.operations.get(&OperationType::Delete) else {
        return Err(SettingsError::OperationNotSupported {
            operation: OperationType::Delete,
        });
    };

    let state = State::new_with_special_variables(author, guild_id);

    let Some(pkey_column) = setting.columns.iter().find(|c| c.id == setting.primary_key) else {
        return Err(SettingsError::Generic {
            message: "Primary key column not found".to_string(),
            src: "settings_update [pkey_column_let_else]".to_string(),
            typ: "internal".to_string(),
        });
    };

    let pkey = _parse_value(pkey, &state, &pkey_column.column_type, setting.primary_key)?;

    let mut data_store = setting
        .data_store
        .create(
            setting,
            guild_id,
            author,
            data,
            common_filters(setting, OperationType::Delete, &state),
        )
        .await?;

    // Start the transaction now that basic validation is done
    data_store.start_transaction().await?;

    // Fetch entire row to execute actions on before deleting
    let cols = setting
        .columns
        .iter()
        .map(|c| c.id.to_string())
        .collect::<Vec<String>>();

    let mut state = data_store
        .fetch_all(
            &cols,
            indexmap::indexmap! {
                setting.primary_key.to_string() => pkey.clone(),
            },
        )
        .await?;

    if state.is_empty() {
        return Err(SettingsError::RowDoesNotExist {
            column_id: setting.primary_key.to_string(),
        });
    }

    let mut state = state.pop().unwrap(); // We know there is only one row

    // Run validator
    setting
        .validator
        .validate(
            super::types::HookContext {
                author,
                guild_id,
                operation_type: OperationType::Delete,
                data_store: &mut *data_store,
                data,
                unchanged_fields: vec![],
            },
            &mut state,
        )
        .await?;

    // Now delete the entire row, the ignored_for does not matter here as we are deleting the entire row
    data_store
        .delete_matching_entries(indexmap::indexmap! {
            setting.primary_key.to_string() => pkey.clone(),
        })
        .await?;

    // Commit the transaction
    data_store.commit().await?;

    // Execute post actions
    setting
        .post_action
        .post_action(
            super::types::HookContext {
                author,
                guild_id,
                operation_type: OperationType::Delete,
                data_store: &mut *data_store,
                data,
                unchanged_fields: vec![],
            },
            &mut state,
        )
        .await?;

    Ok(state)
}

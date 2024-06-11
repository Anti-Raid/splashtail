use super::config_opts::{ColumnType, ConfigOption, InnerColumnType, OperationType};
use super::state::State;
use crate::silverpelt::value::Value;

/// Validates the value against the schema's column type handling schema checks if `perform_schema_checks` is true
#[allow(dead_code)]
fn _validate_value(
    v: &Value,
    column_type: &ColumnType,
    is_nullable: bool,
    perform_schema_checks: bool,
) -> Result<(), crate::Error> {
    match column_type {
        ColumnType::Scalar { column_type } => {
            if matches!(v, Value::None) {
                if is_nullable {
                    return Ok(());
                } else {
                    return Err("Value is null, but column is not nullable".into());
                }
            }

            if matches!(v, Value::List(_)) {
                return Err(format!("Expected scalar, got list {}", v).into());
            }

            match column_type {
                InnerColumnType::Uuid {} => {
                    if !matches!(v, Value::Uuid(_)) {
                        return Err(format!("Expected Uuid, got {}", v).into());
                    }
                }
                InnerColumnType::String {
                    min_length,
                    max_length,
                    allowed_values,
                } => {
                    if !matches!(v, Value::String(_) | Value::Uuid(_)) {
                        return Err(format!("Expected String, got {}", v).into());
                    }

                    if perform_schema_checks {
                        let s = match v {
                            Value::String(s) => s,
                            _ => unreachable!(),
                        };

                        if let Some(min) = min_length {
                            if s.len() < *min {
                                return Err(
                                    format!("String is too short, min length is {}", min).into()
                                );
                            }
                        }

                        if let Some(max) = max_length {
                            if s.len() > *max {
                                return Err(
                                    format!("String is too long, max length is {}", max).into()
                                );
                            }
                        }

                        if !allowed_values.is_empty() && !allowed_values.contains(&s.as_str()) {
                            return Err("String is not in the allowed values".into());
                        }
                    }
                }
                InnerColumnType::Timestamp {} => {
                    if !matches!(v, Value::Timestamp(_)) {
                        return Err(format!("Expected Timestamp, got {}", v).into());
                    }

                    // No further checks needed
                }
                InnerColumnType::TimestampTz {} => {
                    if !matches!(v, Value::TimestampTz(_)) {
                        return Err(format!("Expected TimestampTz, got {}", v).into());
                    }

                    // No further checks needed
                }
                InnerColumnType::Integer {} => {
                    if !matches!(v, Value::Integer(_)) {
                        return Err(format!("Expected Integer, got {}", v).into());
                    }
                }
                InnerColumnType::Float {} => {
                    if !matches!(v, Value::Float(_)) {
                        return Err(format!("Expected Float, got {}", v).into());
                    }
                }
                InnerColumnType::BitFlag { .. } => {
                    if !matches!(v, Value::Integer(_)) {
                        return Err(format!("Expected Integer, got {}", v).into());
                    }

                    // TODO: Add value parsing for bit flags
                }
                InnerColumnType::Boolean {} => {
                    if !matches!(v, Value::Boolean(_)) {
                        return Err(format!("Expected Boolean, got {}", v).into());
                    }
                }
                InnerColumnType::User {} => {
                    if !matches!(v, Value::String(_)) {
                        return Err(format!("Expected a user id (string), got {}", v).into());
                    }

                    if perform_schema_checks {
                        let s = match v {
                            Value::String(s) => s,
                            _ => unreachable!(),
                        };

                        // Try parsing to a UserId
                        if s.parse::<serenity::all::UserId>().is_err() {
                            return Err("Invalid user id".into());
                        }
                    }
                }
                InnerColumnType::Channel {} => {
                    if !matches!(v, Value::String(_)) {
                        return Err(format!("Expected a channel id (string), got {}", v).into());
                    }

                    if perform_schema_checks {
                        let s = match v {
                            Value::String(s) => s,
                            _ => unreachable!(),
                        };

                        // Try parsing to a ChannelId
                        if s.parse::<serenity::all::ChannelId>().is_err() {
                            return Err("Invalid channel id".into());
                        }
                    }
                }
                InnerColumnType::Role {} => {
                    if !matches!(v, Value::String(_)) {
                        return Err(format!("Expected a role id (string), got {}", v).into());
                    }

                    if perform_schema_checks {
                        let s = match v {
                            Value::String(s) => s,
                            _ => unreachable!(),
                        };

                        // Try parsing to a RoleId
                        if s.parse::<serenity::all::RoleId>().is_err() {
                            return Err("Invalid role id".into());
                        }
                    }
                }
                InnerColumnType::Emoji {} => {
                    if !matches!(v, Value::String(_)) {
                        return Err(format!("Expected an emoji id (string), got {}", v).into());
                    }

                    if perform_schema_checks {
                        let s = match v {
                            Value::String(s) => s,
                            _ => unreachable!(),
                        };

                        // Try parsing to an EmojiId
                        if s.parse::<serenity::all::EmojiId>().is_err() {
                            return Err("Invalid emoji id".into());
                        }
                    }
                }
                InnerColumnType::Message {} => {
                    if !matches!(v, Value::String(_)) {
                        return Err(format!("Expected a message id (string), got {}", v).into());
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
                            return Err("Invalid message id".into());
                        }

                        // Try parsing to a ChannelId
                        if parts[0].parse::<serenity::all::ChannelId>().is_err() {
                            return Err("Invalid channel id".into());
                        }

                        if parts[1].parse::<serenity::all::MessageId>().is_err() {
                            return Err("Invalid message id".into());
                        }
                    }
                }
                InnerColumnType::Json {} => {
                    if !matches!(v, Value::Map(_)) {
                        return Err(format!("Expected a map (json), got {}", v).into());
                    }
                }
            }
        }
        ColumnType::Array { inner } => {
            if matches!(v, Value::None) {
                if is_nullable {
                    return Ok(());
                } else {
                    return Err("Value is null, but column is not nullable".into());
                }
            }

            if !matches!(v, Value::List(_)) {
                return Err(format!("Expected list, got scalar {}", v).into());
            }

            let l = match v {
                Value::List(l) => l,
                _ => unreachable!(),
            };

            let column_type = ColumnType::new_scalar(inner.clone());
            for v in l {
                _validate_value(v, &column_type, is_nullable, perform_schema_checks)?;
            }
        }
    }

    Ok(())
}

/// Returns the column ids for the given operation given the config option (setting) and the operation type
///
/// This also handles operation specific data as well
fn _getcols(setting: &ConfigOption, op: OperationType) -> Vec<String> {
    let mut cols = vec![];

    if let Some(op_specific) = setting.operations.get(&op) {
        let mut cols = Vec::new();

        if op_specific.column_ids.is_empty() {
            // No column ids, use setting.columns
            for col in &setting.columns {
                cols.push(col.id.to_string());
            }
        } else {
            for col in &op_specific.column_ids {
                cols.push(col.to_string());
            }
        }

        cols
    } else {
        for col in &setting.columns {
            cols.push(col.id.to_string());
        }

        cols
    }
}

/// Parses a row, handling its actions and adding the converted/created silverpelt Value to the state
async fn _parse_row(
    setting: &ConfigOption,
    row: &sqlx::postgres::PgRow,
    state: &mut State,
    ctx: &serenity::all::Context,
    author: serenity::all::UserId,
    guild_id: serenity::all::GuildId,
    op: OperationType,
) -> Result<(), crate::Error> {
    for (i, col) in setting.columns.iter().enumerate() {
        // Fetch and validate the value itv
        let val = Value::from_sqlx(row, i)?;
        _validate_value(&val, &col.column_type, col.nullable, false)
            .map_err(|e| format!("Error validating value for column {}: {}", col.id, e))?;

        // Insert the value into the map
        state.state.insert(col.id.to_string(), val);

        let actions = col.pre_checks.get(&op).unwrap_or(&col.default_pre_checks);

        crate::silverpelt::settings::action_executor::execute_actions(
            state, actions, ctx, author, guild_id,
        )
        .await?;
    }

    Ok(())
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

async fn _post_op_colset(
    setting: &ConfigOption,
    state: &mut State,
    pool: &sqlx::PgPool,
    author: serenity::all::UserId,
    guild_id: serenity::all::GuildId,
    op: OperationType,
) -> Result<(), crate::Error> {
    let Some(op_specific) = setting.operations.get(&op) else {
        return Ok(()); // No operation specific data
    };

    for ((table_name, column_name), value) in op_specific.columns_to_set.iter() {
        let value = state.template_to_string(author, guild_id, value);

        let sql_stmt = format!(
            "UPDATE {} SET {} = $1 WHERE {} = $2",
            table_name, column_name, setting.guild_id
        );

        let query = sqlx::query(sql_stmt.as_str());

        let query = _query_bind_value(query, value);

        query.bind(guild_id.to_string()).execute(pool).await?;
    }

    Ok(())
}

pub async fn settings_view(
    setting: &ConfigOption,
    ctx: &serenity::all::Context,
    pool: &sqlx::PgPool,
    guild_id: serenity::all::GuildId,
    author: serenity::all::UserId,
) -> Result<Vec<State>, crate::Error> {
    let cols = _getcols(setting, OperationType::View);

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
    .await?;

    if row.is_empty() {
        return Ok(Vec::new());
    }

    let mut values: Vec<State> = Vec::new();

    for row in row {
        let mut state = State::new();
        _parse_row(
            setting,
            &row,
            &mut state,
            ctx,
            author,
            guild_id,
            OperationType::View,
        )
        .await?;
        _post_op_colset(
            setting,
            &mut state,
            pool,
            author,
            guild_id,
            OperationType::View,
        )
        .await?;
        values.push(state);
    }

    Ok(values)
}

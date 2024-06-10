use super::config_opts::{ColumnType, ConfigOption, InnerColumnType, OperationType};
use super::state::State;
use crate::silverpelt::value::Value;
use futures_util::StreamExt;
use std::time::Duration;

/// Validates the value against the schema's column type
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
                                return Err(format!(
                                    "String is too short, min length is {}",
                                    min
                                )
                                .into());
                            }
                        }

                        if let Some(max) = max_length {
                            if s.len() > *max {
                                return Err(format!(
                                    "String is too long, max length is {}",
                                    max
                                )
                                .into());
                            }
                        }

                        if !allowed_values.is_empty() && !allowed_values.contains(&s.as_str()) {
                            return Err("String is not in the allowed values".into());
                        }
                    }
                }
                InnerColumnType::Timestamp {} => {
                    if !matches!(v, Value::String(_)) {
                        return Err(format!("Expected Timestamp, got {}", v).into());
                    }

                    if perform_schema_checks {
                        let s = match v {
                            Value::String(s) => s,
                            _ => unreachable!(),
                        };

                        if chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                            .is_err()
                        {
                            return Err("Invalid timestamp format".into());
                        }
                    }
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
                        return Err(
                            format!("Expected a channel id (string), got {}", v).into()
                        );
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
                        return Err(
                            format!("Expected an emoji id (string), got {}", v).into()
                        );
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
                        return Err(
                            format!("Expected a message id (string), got {}", v).into()
                        );
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

fn _getcols(setting: &ConfigOption) -> Vec<String> {
    let mut cols = vec![];

    for col in &setting.columns {
        cols.push(col.id.to_string());
    }

    cols
}

async fn _parse_row(
    setting: &ConfigOption,
    row: &sqlx::postgres::PgRow,
    state: &mut State,
    ctx: &serenity::all::Context,
) -> Result<(), crate::Error> {
    for (i, col) in setting.columns.iter().enumerate() {
        // Fetch and validate the value itv
        let val = Value::from_sqlx(row, i)?;
        _validate_value(&val, &col.column_type, col.nullable, false)
            .map_err(|e| format!("Error validating value for column {}: {}", col.id, e))?;

        // Insert the value into the map
        state.state.insert(col.id.to_string(), val);

        let actions = col
            .pre_checks
            .get(&OperationType::View)
            .unwrap_or(&col.default_pre_checks);

        crate::silverpelt::settings::action_executor::execute_actions(state, actions, ctx).await?;
    }

    Ok(())
}

fn _create_reply<'a>(
    setting: &ConfigOption,
    values: &'a [State],
    index: usize,
) -> poise::CreateReply<'a> {
    fn create_action_row<'a>(index: usize, total: usize) -> serenity::all::CreateActionRow<'a> {
        serenity::all::CreateActionRow::Buttons(vec![
            serenity::all::CreateButton::new("previous")
                .style(serenity::all::ButtonStyle::Primary)
                .label("Previous")
                .disabled(index == 0),
            serenity::all::CreateButton::new("next")
                .style(serenity::all::ButtonStyle::Primary)
                .label("Next")
                .disabled(index >= total - 1),
            serenity::all::CreateButton::new("first")
                .style(serenity::all::ButtonStyle::Primary)
                .label("First")
                .disabled(false),
            serenity::all::CreateButton::new("close")
                .style(serenity::all::ButtonStyle::Danger)
                .label("Close")
                .disabled(false),
        ])
    }

    let mut embed = serenity::all::CreateEmbed::default();

    embed = embed.title(format!(
        "{} ({} of {})",
        setting.name,
        index + 1,
        values.len()
    ));

    for (key, value) in values[index].state.iter() {
        embed = embed.field(key, value.to_string(), true);
    }

    poise::CreateReply::new()
        .embed(embed)
        .components(vec![create_action_row(index, values.len())])
}

// Common settings viewer for poise, sends an embed, all that stuff
pub async fn settings_viewer(
    ctx: &crate::Context<'_>,
    setting: &ConfigOption,
) -> Result<(), crate::Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command must be run in a server".into());
    };

    let cols = _getcols(setting);

    let data = ctx.data();
    let serenity_ctx = ctx.serenity_context();

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
    .fetch_all(&data.pool)
    .await?;

    if row.is_empty() {
        return Err(format!(
            "Whoa there! There seems to be no {}",
            setting.name.to_lowercase()
        )
        .into());
    }

    let mut values: Vec<State> = Vec::new();

    for row in row {
        let mut state = State::new();
        _parse_row(setting, &row, &mut state, serenity_ctx).await?;
        values.push(state);
    }

    let mut index = 0;

    let reply = _create_reply(setting, &values, index);

    let msg = ctx.send(reply).await?.into_message().await?;

    let collector = msg
        .await_component_interactions(ctx.serenity_context().shard.clone())
        .author_id(ctx.author().id)
        .timeout(Duration::from_secs(180));

    let mut collect_stream = collector.stream();

    while let Some(item) = collect_stream.next().await {
        let item_id = item.data.custom_id.as_str();

        match item_id {
            "previous" => {
                index = index.saturating_sub(1);
            }
            "next" => {
                index = usize::min(index + 1, values.len() - 1);
            }
            "first" => {
                index = 0;
            }
            "close" => {
                item.defer(&serenity_ctx.http).await?;
                item.delete_response(&serenity_ctx.http).await?;
                break;
            }
            _ => {}
        }

        item.defer(&serenity_ctx.http).await?;

        let reply = _create_reply(setting, &values, index);

        item.edit_response(
            &serenity_ctx.http,
            reply.to_slash_initial_response_edit(serenity::all::EditInteractionResponse::default()),
        )
        .await?;
    }

    Ok(())
}

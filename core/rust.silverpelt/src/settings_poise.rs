use futures_util::StreamExt;
use module_settings::cfg::{settings_create, settings_delete, settings_update, settings_view};
use module_settings::state::State;
use module_settings::types::{
    ColumnType, ConfigOption, InnerColumnType, InnerColumnTypeStringKind, OperationType,
};
use splashcore_rs::value::Value;
use std::time::Duration;

fn _get_display_value(column_type: &ColumnType, value: &Value, state: &State) -> String {
    match column_type {
        ColumnType::Scalar { column_type } => match column_type {
            InnerColumnType::String { kind, .. } => match kind {
                InnerColumnTypeStringKind::Channel { .. } => format!("<#{}>", value),
                InnerColumnTypeStringKind::Role => format!("<@&{}>", value),
                InnerColumnTypeStringKind::User => format!("<@{}>", value),
                _ => value.to_string(),
            },
            InnerColumnType::BitFlag { values } => {
                let v = match value {
                    Value::Integer(v) => *v,
                    Value::Float(v) => *v as i64,
                    Value::String(v) => {
                        if let Ok(v) = v.parse::<i64>() {
                            v
                        } else {
                            return v.to_string();
                        }
                    }
                    _ => return value.to_string(),
                };

                let mut result = Vec::new();
                for (name, flag) in values.iter() {
                    if v & *flag == *flag {
                        result.push(format!("`{}` ({})", name, flag));
                    }
                }
                result.join(", ")
            }
            _ => value.to_string(),
        },
        ColumnType::Array { inner } => {
            // Then the value must also be an array, check that or fallback to scalar _get_display_value
            match value {
                Value::List(values) => values
                    .iter()
                    .map(|v| _get_display_value(&ColumnType::new_scalar(inner.clone()), v, state))
                    .collect::<Vec<String>>()
                    .join(", "),
                _ => _get_display_value(&ColumnType::new_scalar(inner.clone()), value, state),
            }
        }
        ColumnType::Dynamic { clauses } => {
            for clause in clauses {
                let _value = state.template_to_string(clause.field);

                if _value == clause.value {
                    return _get_display_value(&clause.column_type, value, state);
                }
            }

            value.to_string()
        }
    }
}

/// Common settings viewer for poise, sends an embed, all that stuff
pub async fn settings_viewer(
    ctx: &crate::Context<'_>,
    setting: &ConfigOption,
    fields: indexmap::IndexMap<String, Value>, // The filters to apply
) -> Result<(), crate::Error> {
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
            if key.starts_with("__") {
                continue; // Skip internal variables
            }

            // Find the key in the schema
            let column = setting.columns.iter().find(|c| c.id == key);

            let display_key = if let Some(column) = column {
                column.name.to_string()
            } else {
                key.clone()
            };

            let display_value = if let Some(column) = column {
                _get_display_value(&column.column_type, value, &values[index])
            } else {
                value.to_string()
            };

            if display_value.len() > 1024 {
                // Discord embed fields have a 1024 character limit
                // Split the value into multiple fields
                let mut len = 0;

                while display_value.len() >= len {
                    // Take the next 1024 characters
                    let next: String = display_value.chars().skip(len).take(1024).collect();
                    let next_len = next.len();
                    embed = embed.field(display_key.clone(), next, true);
                    len += next_len;
                }

                continue;
            }

            embed = embed.field(display_key, display_value, true);
        }

        poise::CreateReply::new()
            .embed(embed)
            .components(vec![create_action_row(index, values.len())])
    }

    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command must be run in a server".into());
    };

    let Some(operation_specific) = setting.operations.get(&OperationType::View) else {
        return Err("Unsupported operation (View) for setting".into());
    };

    let data = ctx.data();
    let cache_http = botox::cache::CacheHttpImpl::from_ctx(ctx.serenity_context());

    {
        let perm_res = crate::cmd::check_command(
            &data.silverpelt_cache,
            operation_specific.corresponding_command,
            guild_id,
            ctx.author().id,
            &data.pool,
            &cache_http,
            &Some(*ctx),
            crate::cmd::CheckCommandOptions {
                ignore_module_disabled: true,
                ..Default::default()
            },
        )
        .await;

        let is_ok = perm_res.is_ok();

        if !is_ok {
            ctx.send(
                poise::CreateReply::new().embed(
                    serenity::all::CreateEmbed::new()
                        .color(serenity::all::Color::RED)
                        .title("You don't have permission to update this setting?")
                        .description(perm_res.to_markdown())
                        .field("Code", format!("`{}`", perm_res.code()), false),
                ),
            )
            .await?;

            return Ok(());
        }
    }

    let values = settings_view(
        setting,
        &data.settings_data(cache_http),
        guild_id,
        ctx.author().id,
        fields,
    )
    .await
    .map_err(|e| format!("Error fetching settings: {}", e))?;

    if values.is_empty() {
        ctx.say(format!("No settings found for `{}`", setting.name))
            .await?;
        return Ok(());
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
                item.defer(ctx.http()).await?;
                item.delete_response(ctx.http()).await?;
                break;
            }
            _ => {}
        }

        item.defer(ctx.http()).await?;

        let reply = _create_reply(setting, &values, index);

        item.edit_response(
            ctx.http(),
            reply.to_slash_initial_response_edit(serenity::all::EditInteractionResponse::default()),
        )
        .await?;
    }

    Ok(())
}

/// Common settings creator for poise, sends an embed, all that stuff
pub async fn settings_creator(
    ctx: &crate::Context<'_>,
    setting: &ConfigOption,
    fields: indexmap::IndexMap<String, Value>,
) -> Result<(), crate::Error> {
    fn _create_reply<'a>(setting: &ConfigOption, value: &State) -> poise::CreateReply<'a> {
        let mut embed = serenity::all::CreateEmbed::default();

        embed = embed.title(format!("Created {}", setting.name));

        for (key, v) in value.state.iter() {
            if key.starts_with("__") {
                continue; // Skip internal variables
            }

            // Find the key in the schema
            let column = setting.columns.iter().find(|c| c.id == key);

            let display_key = if let Some(column) = column {
                column.name.to_string()
            } else {
                key.clone()
            };

            let display_value = if let Some(column) = column {
                _get_display_value(&column.column_type, v, value)
            } else {
                v.to_string()
            };

            if display_value.len() > 1024 {
                // Discord embed fields have a 1024 character limit
                // Split the value into multiple fields
                let mut len = 0;

                while display_value.len() >= len {
                    // Take the next 1024 characters
                    let next: String = display_value.chars().skip(len).take(1024).collect();
                    let next_len = next.len();
                    embed = embed.field(display_key.clone(), next, true);
                    len += next_len;
                }

                continue;
            }

            embed = embed.field(display_key, display_value, true);
        }

        poise::CreateReply::new().embed(embed)
    }

    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command must be run in a server".into());
    };

    let Some(operation_specific) = setting.operations.get(&OperationType::Create) else {
        return Err("Unsupported operation (Create) for setting".into());
    };

    let data = ctx.data();
    let cache_http = botox::cache::CacheHttpImpl::from_ctx(ctx.serenity_context());

    {
        let perm_res = crate::cmd::check_command(
            &data.silverpelt_cache,
            operation_specific.corresponding_command,
            guild_id,
            ctx.author().id,
            &data.pool,
            &cache_http,
            &Some(*ctx),
            crate::cmd::CheckCommandOptions {
                ignore_module_disabled: true,
                ..Default::default()
            },
        )
        .await;

        let is_ok = perm_res.is_ok();

        if !is_ok {
            ctx.send(
                poise::CreateReply::new().embed(
                    serenity::all::CreateEmbed::new()
                        .color(serenity::all::Color::RED)
                        .title("You don't have permission to update this setting?")
                        .description(perm_res.to_markdown())
                        .field("Code", format!("`{}`", perm_res.code()), false),
                ),
            )
            .await?;

            return Ok(());
        }
    }

    let mut value = settings_create(
        setting,
        &data.settings_data(cache_http),
        guild_id,
        ctx.author().id,
        fields,
    )
    .await
    .map_err(|e| format!("Error creating new setting: {}", e))?;

    value.state.insert(
        "key".to_string(),
        value.template_to_string(setting.title_template),
    );

    let reply = _create_reply(setting, &value);

    ctx.send(reply).await?;

    Ok(())
}

/// Common settings updater for poise, sends an embed, all that stuff
pub async fn settings_updater(
    ctx: &crate::Context<'_>,
    setting: &ConfigOption,
    fields: indexmap::IndexMap<String, Value>,
) -> Result<(), crate::Error> {
    fn _create_reply<'a>(setting: &ConfigOption, value: &State) -> poise::CreateReply<'a> {
        let mut embed = serenity::all::CreateEmbed::default();

        embed = embed.title(format!("Updated {}", setting.name));

        for (key, v) in value.state.iter() {
            if key.starts_with("__") {
                continue; // Skip internal variables
            }

            // Find the key in the schema
            let column = setting.columns.iter().find(|c| c.id == key);

            let display_key = if let Some(column) = column {
                column.name.to_string()
            } else {
                key.clone()
            };

            let display_value = if let Some(column) = column {
                _get_display_value(&column.column_type, v, value)
            } else {
                v.to_string()
            };

            if display_value.len() > 1024 {
                // Discord embed fields have a 1024 character limit
                // Split the value into multiple fields
                let mut len = 0;

                while display_value.len() >= len {
                    // Take the next 1024 characters
                    let next: String = display_value.chars().skip(len).take(1024).collect();
                    let next_len = next.len();
                    embed = embed.field(display_key.clone(), next, true);
                    len += next_len;
                }

                continue;
            }

            embed = embed.field(display_key, display_value, true);
        }

        poise::CreateReply::new().embed(embed)
    }

    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command must be run in a server".into());
    };

    let Some(operation_specific) = setting.operations.get(&OperationType::Update) else {
        return Err("Unsupported operation (Update) for setting".into());
    };

    let data = ctx.data();
    let cache_http = botox::cache::CacheHttpImpl::from_ctx(ctx.serenity_context());

    {
        let perm_res = crate::cmd::check_command(
            &data.silverpelt_cache,
            operation_specific.corresponding_command,
            guild_id,
            ctx.author().id,
            &data.pool,
            &cache_http,
            &Some(*ctx),
            crate::cmd::CheckCommandOptions {
                ignore_module_disabled: true,
                ..Default::default()
            },
        )
        .await;

        let is_ok = perm_res.is_ok();

        if !is_ok {
            ctx.send(
                poise::CreateReply::new().embed(
                    serenity::all::CreateEmbed::new()
                        .color(serenity::all::Color::RED)
                        .title("You don't have permission to update this setting?")
                        .description(perm_res.to_markdown())
                        .field("Code", format!("`{}`", perm_res.code()), false),
                ),
            )
            .await?;

            return Ok(());
        }
    }

    let mut value = settings_update(
        setting,
        &data.settings_data(cache_http),
        guild_id,
        ctx.author().id,
        fields,
    )
    .await
    .map_err(|e| format!("Error updating setting: {}", e))?;

    value.state.insert(
        "key".to_string(),
        value.template_to_string(setting.title_template),
    );

    let reply = _create_reply(setting, &value);

    ctx.send(reply).await?;

    Ok(())
}

/// Common settings deleter for poise, sends an embed, all that stuff
pub async fn settings_deleter(
    ctx: &crate::Context<'_>,
    setting: &ConfigOption,
    pkey: Value,
) -> Result<(), crate::Error> {
    fn _create_reply<'a>(setting: &ConfigOption, value: &State) -> poise::CreateReply<'a> {
        let mut embed = serenity::all::CreateEmbed::default();

        embed = embed
            .title(format!("Deleted {}", setting.name))
            .color(serenity::all::Color::RED);

        for (key, v) in value.state.iter() {
            if key.starts_with("__") {
                continue; // Skip internal variables
            }

            // Find the key in the schema
            let column = setting.columns.iter().find(|c| c.id == key);

            let display_key = if let Some(column) = column {
                column.name.to_string()
            } else {
                key.clone()
            };

            let display_value = if let Some(column) = column {
                _get_display_value(&column.column_type, v, value)
            } else {
                v.to_string()
            };

            if display_value.len() > 1024 {
                // Discord embed fields have a 1024 character limit
                // Split the value into multiple fields
                let mut len = 0;

                while display_value.len() >= len {
                    // Take the next 1024 characters
                    let next: String = display_value.chars().skip(len).take(1024).collect();
                    let next_len = next.len();
                    embed = embed.field(display_key.clone(), next, true);
                    len += next_len;
                }

                continue;
            }

            embed = embed.field(display_key, display_value, true);
        }

        poise::CreateReply::new().embed(embed)
    }

    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command must be run in a server".into());
    };

    let Some(operation_specific) = setting.operations.get(&OperationType::Delete) else {
        return Err("Unsupported operation (Delete) for setting".into());
    };

    let data = ctx.data();
    let cache_http = botox::cache::CacheHttpImpl::from_ctx(ctx.serenity_context());

    {
        let perm_res = crate::cmd::check_command(
            &data.silverpelt_cache,
            operation_specific.corresponding_command,
            guild_id,
            ctx.author().id,
            &data.pool,
            &cache_http,
            &Some(*ctx),
            crate::cmd::CheckCommandOptions {
                ignore_module_disabled: true,
                ..Default::default()
            },
        )
        .await;

        let is_ok = perm_res.is_ok();

        if !is_ok {
            ctx.send(
                poise::CreateReply::new().embed(
                    serenity::all::CreateEmbed::new()
                        .color(serenity::all::Color::RED)
                        .title("You don't have permission to update this setting?")
                        .description(perm_res.to_markdown())
                        .field("Code", format!("`{}`", perm_res.code()), false),
                ),
            )
            .await?;

            return Ok(());
        }
    }

    let mut value = settings_delete(
        setting,
        &data.settings_data(cache_http),
        guild_id,
        ctx.author().id,
        pkey,
    )
    .await
    .map_err(|e| format!("Error deleting setting: {}", e))?;

    value.state.insert(
        "key".to_string(),
        value.template_to_string(setting.title_template),
    );

    let reply = _create_reply(setting, &value);

    ctx.send(reply).await?;

    Ok(())
}

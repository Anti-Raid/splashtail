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
        count: i64,
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

        embed = embed.title(format!("{} ({} of {})", setting.name, index + 1, count));

        for (key, value) in values[index].state.iter() {
            if key.starts_with("__") {
                continue; // Skip internal variables
            }

            // Find a __{key}_botDisplayIgnore field, skip if it exists
            if values[index]
                .state
                .contains_key(&format!("__{}_botDisplayIgnore", key))
            {
                continue;
            }

            // Find the key in the schema
            let column = setting.columns.iter().find(|c| c.id == key);

            let display_key = if let Some(column) = column {
                column.name.to_string()
            } else {
                key.clone()
            };

            let mut display_value = if let Some(column) = column {
                _get_display_value(&column.column_type, value, &values[index])
            } else {
                value.to_string()
            };

            if display_value.len() > 1024 {
                display_value = format!("{}...", &display_value[..1021]);
            }

            embed = embed.field(display_key, display_value, true);
        }

        poise::CreateReply::new()
            .embed(embed)
            .components(vec![create_action_row(
                index,
                count.try_into().unwrap_or_default(),
            )])
    }

    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command must be run in a server".into());
    };

    if setting.operations.get(&OperationType::View).is_none() {
        return Err("Unsupported operation (View) for setting".into());
    };

    let data = ctx.data();
    let cache_http = botox::cache::CacheHttpImpl::from_ctx(ctx.serenity_context());

    {
        let perm_res = crate::cmd::check_command(
            &data.silverpelt_cache,
            &setting.get_corresponding_command(OperationType::View),
            guild_id,
            ctx.author().id,
            &data.pool,
            &cache_http,
            &data.reqwest,
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

    let mut values = settings_view(
        setting,
        &data.settings_data(ctx.serenity_context().clone()),
        guild_id,
        ctx.author().id,
        fields.clone(),
    )
    .await
    .map_err(|e| format!("Error fetching settings: {}", e))?;

    if values.is_empty() {
        ctx.say(format!("No settings found for `{}`", setting.name))
            .await?;
        return Ok(());
    }

    let total_count = match settings_view(
        setting,
        &data.settings_data(ctx.serenity_context().clone()),
        guild_id,
        ctx.author().id,
        {
            let mut fields = fields.clone();
            fields.insert("__count".to_string(), Value::Boolean(true));
            fields
        },
    )
    .await
    .map_err(|e| format!("Error fetching settings count: {}", e))?
    .first()
    .ok_or("No count found")?
    .state
    .get("count")
    {
        Some(Value::Integer(v)) => *v,
        _ => 0,
    };

    let total_count_usize: usize = total_count.try_into()?;

    let mut index = 0;

    let reply = _create_reply(setting, &values, index, total_count);

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
                index = usize::min(index + 1, total_count_usize - 1);
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

        if index > total_count_usize {
            index = total_count_usize - 1;
        }

        // Check if we need to fetch more values
        if index >= values.len() {
            // Fetch the next page
            let next = settings_view(
                setting,
                &data.settings_data(ctx.serenity_context().clone()),
                guild_id,
                ctx.author().id,
                {
                    let mut fields = fields.clone();
                    fields.insert("__offset".to_string(), Value::Integer(index as i64));
                    fields.insert("__limit".to_string(), Value::Integer(setting.max_return));
                    fields
                },
            )
            .await
            .map_err(|e| format!("Error fetching settings: {}", e))?;

            values.extend(next);
        }

        let reply = _create_reply(setting, &values, index, total_count);

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

            // Find a __{key}_botDisplayIgnore field, skip if it exists
            if value
                .state
                .contains_key(&format!("__{}_botDisplayIgnore", key))
            {
                continue;
            }

            // Find the key in the schema
            let column = setting.columns.iter().find(|c| c.id == key);

            let display_key = if let Some(column) = column {
                column.name.to_string()
            } else {
                key.clone()
            };

            let mut display_value = if let Some(column) = column {
                _get_display_value(&column.column_type, v, value)
            } else {
                v.to_string()
            };

            if display_value.len() > 1024 {
                display_value = format!("{}...", &display_value[..1021]);
            }

            embed = embed.field(display_key, display_value, true);
        }

        poise::CreateReply::new().embed(embed)
    }

    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command must be run in a server".into());
    };

    if setting.operations.get(&OperationType::Create).is_none() {
        return Err("Unsupported operation (Create) for setting".into());
    };

    let data = ctx.data();
    let cache_http = botox::cache::CacheHttpImpl::from_ctx(ctx.serenity_context());

    {
        let perm_res = crate::cmd::check_command(
            &data.silverpelt_cache,
            &setting.get_corresponding_command(OperationType::Create),
            guild_id,
            ctx.author().id,
            &data.pool,
            &cache_http,
            &data.reqwest,
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

    // Send message that we are creating the setting
    let mut msg = ctx
        .send(
            poise::CreateReply::new().embed(
                serenity::all::CreateEmbed::new()
                    .title(format!("Creating {}", setting.name))
                    .description(":hourglass_flowing_sand: Please wait..."),
            ),
        )
        .await?
        .into_message()
        .await?;

    let mut value = settings_create(
        setting,
        &data.settings_data(ctx.serenity_context().clone()),
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

    msg.edit(ctx, reply.to_prefix_edit(serenity::all::EditMessage::new()))
        .await?;

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

            // Find a __{key}_botDisplayIgnore field, skip if it exists
            if value
                .state
                .contains_key(&format!("__{}_botDisplayIgnore", key))
            {
                continue;
            }

            // Find the key in the schema
            let column = setting.columns.iter().find(|c| c.id == key);

            let display_key = if let Some(column) = column {
                column.name.to_string()
            } else {
                key.clone()
            };

            let mut display_value = if let Some(column) = column {
                _get_display_value(&column.column_type, v, value)
            } else {
                v.to_string()
            };

            if display_value.len() > 1024 {
                display_value = format!("{}...", &display_value[..1021]);
            }

            embed = embed.field(display_key, display_value, true);
        }

        poise::CreateReply::new().embed(embed)
    }

    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command must be run in a server".into());
    };

    if setting.operations.get(&OperationType::Update).is_none() {
        return Err("Unsupported operation (Update) for setting".into());
    };

    let data = ctx.data();
    let cache_http = botox::cache::CacheHttpImpl::from_ctx(ctx.serenity_context());

    {
        let perm_res = crate::cmd::check_command(
            &data.silverpelt_cache,
            &setting.get_corresponding_command(OperationType::Update),
            guild_id,
            ctx.author().id,
            &data.pool,
            &cache_http,
            &data.reqwest,
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
        &data.settings_data(ctx.serenity_context().clone()),
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

            // Find a __{key}_botDisplayIgnore field, skip if it exists
            if value
                .state
                .contains_key(&format!("__{}_botDisplayIgnore", key))
            {
                continue;
            }

            // Find the key in the schema
            let column = setting.columns.iter().find(|c| c.id == key);

            let display_key = if let Some(column) = column {
                column.name.to_string()
            } else {
                key.clone()
            };

            let mut display_value = if let Some(column) = column {
                _get_display_value(&column.column_type, v, value)
            } else {
                v.to_string()
            };

            if display_value.len() > 1024 {
                display_value = format!("{}...", &display_value[..1021]);
            }

            embed = embed.field(display_key, display_value, true);
        }

        poise::CreateReply::new().embed(embed)
    }

    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command must be run in a server".into());
    };

    if setting.operations.get(&OperationType::Delete).is_none() {
        return Err("Unsupported operation (Delete) for setting".into());
    }

    let data = ctx.data();
    let cache_http = botox::cache::CacheHttpImpl::from_ctx(ctx.serenity_context());

    {
        let perm_res = crate::cmd::check_command(
            &data.silverpelt_cache,
            &setting.get_corresponding_command(OperationType::Delete),
            guild_id,
            ctx.author().id,
            &data.pool,
            &cache_http,
            &data.reqwest,
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

    // Send message that we are creating the setting
    let mut msg = ctx
        .send(
            poise::CreateReply::new().embed(
                serenity::all::CreateEmbed::new()
                    .title(format!("Deleting {}", setting.name))
                    .description(":hourglass_flowing_sand: Please wait..."),
            ),
        )
        .await?
        .into_message()
        .await?;

    let mut value = settings_delete(
        setting,
        &data.settings_data(ctx.serenity_context().clone()),
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

    msg.edit(ctx, reply.to_prefix_edit(serenity::all::EditMessage::new()))
        .await?;

    Ok(())
}

#[allow(dead_code)]
pub async fn standard_autocomplete<'a>(
    ctx: super::Context<'_>,
    setting: &ConfigOption,
    partial: &'a str,
) -> Vec<serenity::all::AutocompleteChoice<'a>> {
    // Fetch first 20 objects
    let data = ctx.data();

    let guild_id = ctx.guild_id();

    if guild_id.is_none() {
        return Vec::new();
    }

    let guild_id = guild_id.unwrap();

    let limit = std::cmp::max(20, setting.max_return);

    let objs = match settings_view(
        setting,
        &data.settings_data(ctx.serenity_context().clone()),
        guild_id,
        ctx.author().id,
        indexmap::indexmap! {
            "__limit".to_string() => Value::Integer(limit),
        },
    )
    .await
    {
        Ok(objs) => objs,
        Err(_) => return Vec::new(),
    };

    let mut choices = Vec::new();

    for obj in objs {
        let disp_name = obj.template_to_string(setting.title_template);

        let Some(primary_key) = obj.state.get(setting.primary_key) else {
            continue;
        };

        let disp_name = disp_name.to_string();
        let primary_key = primary_key.to_string();

        if disp_name.starts_with(partial) || primary_key.starts_with(partial) {
            choices.push(serenity::all::AutocompleteChoice::new(
                disp_name,
                primary_key,
            ));
            continue;
        }
    }

    choices
}

#[inline(always)]
pub async fn bitflag_autocomplete<'a>(
    ctx: super::Context<'_>,
    values: &indexmap::IndexMap<String, i64>,
    partial: &'a str,
) -> Vec<serenity::all::AutocompleteChoice<'a>> {
    // Fetch all bitflags available
    let guild_id = ctx.guild_id();

    if guild_id.is_none() {
        return Vec::new();
    }

    let current_choices = partial
        .split(';')
        .map(|x| x.to_string())
        .collect::<Vec<String>>();

    let mut choices = Vec::with_capacity(std::cmp::max(values.len(), 25));

    for (label, _) in values {
        // We can abuse autocomplete to emulate a bitflag like setup
        if choices.len() > 25 {
            break;
        }

        if current_choices.contains(label) {
            continue;
        }

        let partial = partial.trim().trim_matches(';');

        if partial.is_empty() {
            choices.push(serenity::all::AutocompleteChoice::new(
                label.clone(),
                label.clone(),
            ));
            continue;
        }

        choices.push(serenity::all::AutocompleteChoice::new(
            format!("{};{}", partial, label),
            format!("{};{}", partial, label),
        ));
    }

    choices
}

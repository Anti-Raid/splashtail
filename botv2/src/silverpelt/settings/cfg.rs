use super::config_opts::{ConfigOption, OperationType};
use super::state::State;
use super::value::Value;
use futures_util::StreamExt;
use std::time::Duration;

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
        // Fetch and validate the value itself
        let val = Value::from_sqlx(row, i)?;
        val.validate_value(&col.column_type, col.nullable, false)
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

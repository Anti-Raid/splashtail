use chrono::TimeZone;
use silverpelt::Context;
use silverpelt::Error;
use splashcore_rs::value::Value;

#[poise::command(
    prefix_command,
    subcommands(
        "register",
        "can_use_bot_list",
        "can_use_bot_add",
        "can_use_bot_update",
        "can_use_bot_delete",
        "inspector__fake_bots_list",
        "inspector__fake_bots_add",
        "inspector__fake_bots_update",
        "inspector__fake_bots_delete",
        "last_task_expiry_view",
        "last_task_expiry_create",
        "last_task_expiry_update",
        "last_task_expiry_delete"
    )
)]
pub async fn sudo(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(prefix_command)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

#[poise::command(prefix_command)]
pub async fn can_use_bot_list(ctx: Context<'_>) -> Result<(), silverpelt::Error> {
    silverpelt::settings_poise::settings_viewer(
        &ctx,
        &super::settings::CAN_USE_BOT,
        indexmap::IndexMap::new(),
    )
    .await
}

#[poise::command(prefix_command)]
pub async fn can_use_bot_add(
    ctx: Context<'_>,
    #[description = "ID of the entity"] id: String,
    #[description = "Type of the entity. Either user or guild"] r#type: String,
    #[description = "Name of the entity"] name: String,
) -> Result<(), silverpelt::Error> {
    silverpelt::settings_poise::settings_creator(
        &ctx,
        &super::settings::CAN_USE_BOT,
        indexmap::indexmap! {
            "id".to_string() => Value::String(id),
            "type".to_string() => Value::String(r#type),
            "name".to_string() => Value::String(name),
        },
    )
    .await
}

#[poise::command(prefix_command)]
pub async fn can_use_bot_update(
    ctx: Context<'_>,
    #[description = "ID of the entity"] id: String,
    #[description = "Type of the entity. Either user or guild"] r#type: String,
    #[description = "Name of the entity"] name: String,
) -> Result<(), silverpelt::Error> {
    silverpelt::settings_poise::settings_updater(
        &ctx,
        &super::settings::CAN_USE_BOT,
        indexmap::indexmap! {
            "id".to_string() => Value::String(id),
            "type".to_string() => Value::String(r#type),
            "name".to_string() => Value::String(name),
        },
    )
    .await
}

#[poise::command(prefix_command)]
pub async fn can_use_bot_delete(
    ctx: Context<'_>,
    #[description = "ID of the entity"] id: String,
) -> Result<(), silverpelt::Error> {
    silverpelt::settings_poise::settings_deleter(
        &ctx,
        &super::settings::CAN_USE_BOT,
        Value::String(id),
    )
    .await
}

#[poise::command(prefix_command)]
pub async fn inspector__fake_bots_list(ctx: Context<'_>) -> Result<(), silverpelt::Error> {
    silverpelt::settings_poise::settings_viewer(
        &ctx,
        &super::settings::INSPECTOR_FAKE_BOTS,
        indexmap::IndexMap::new(),
    )
    .await
}

#[poise::command(prefix_command)]
pub async fn inspector__fake_bots_add(
    ctx: Context<'_>,
    #[description = "ID of the bot"] id: String,
    #[description = "Name of the bot"] name: String,
    #[description = "Official bot IDs, comma-seperated"] official_ids: String,
) -> Result<(), silverpelt::Error> {
    silverpelt::settings_poise::settings_creator(
        &ctx,
        &super::settings::INSPECTOR_FAKE_BOTS,
        indexmap::indexmap! {
            "bot_id".to_string() => Value::String(id),
            "name".to_string() => Value::String(name),
            "official_bot_ids".to_string() => {
                let official_ids = official_ids.split(',').map(|x| Value::String(x.trim().to_string())).collect::<Vec<Value>>();
                Value::List(official_ids)
            },
        },
    )
    .await
}

#[poise::command(prefix_command)]
pub async fn inspector__fake_bots_update(
    ctx: Context<'_>,
    #[description = "ID of the bot"] id: String,
    #[description = "Name of the bot"] name: String,
    #[description = "Official bot IDs, comma-seperated"] official_ids: String,
) -> Result<(), silverpelt::Error> {
    silverpelt::settings_poise::settings_updater(
        &ctx,
        &super::settings::INSPECTOR_FAKE_BOTS,
        indexmap::indexmap! {
            "bot_id".to_string() => Value::String(id),
            "name".to_string() => Value::String(name),
            "official_bot_ids".to_string() => {
                let official_ids = official_ids.split(',').map(|x| Value::String(x.trim().to_string())).collect::<Vec<Value>>();
                Value::List(official_ids)
            },
        },
    )
    .await
}

#[poise::command(prefix_command)]
pub async fn inspector__fake_bots_delete(
    ctx: Context<'_>,
    #[description = "ID of the bot"] id: String,
) -> Result<(), silverpelt::Error> {
    silverpelt::settings_poise::settings_deleter(
        &ctx,
        &super::settings::INSPECTOR_FAKE_BOTS,
        Value::String(id),
    )
    .await
}

#[poise::command(prefix_command)]
pub async fn last_task_expiry_view(ctx: Context<'_>) -> Result<(), silverpelt::Error> {
    silverpelt::settings_poise::settings_viewer(
        &ctx,
        &super::settings::LAST_TASK_EXPIRY,
        indexmap::IndexMap::new(),
    )
    .await
}

#[poise::command(prefix_command)]
pub async fn last_task_expiry_create(
    ctx: Context<'_>,
    #[description = "Task Name (afk etc)"] task: String,
    #[description = "Created At"] created_at: i64,
) -> Result<(), silverpelt::Error> {
    silverpelt::settings_poise::settings_creator(
        &ctx,
        &super::settings::LAST_TASK_EXPIRY,
        indexmap::indexmap! {
            "task".to_string() => Value::String(task),
            "created_at".to_string() => {
                // Parse the date
                let created_at_utc = match chrono::Utc.timestamp_opt(created_at, 0) {
                    chrono::offset::LocalResult::Single(created_at_utc) => created_at_utc,
                    _ => return Err(format!("Invalid timestamp: {}", created_at).into()),
                };

                Value::TimestampTz(created_at_utc)
            },
        },
    )
    .await
}

#[poise::command(prefix_command)]
pub async fn last_task_expiry_update(
    ctx: Context<'_>,
    #[description = "ID"] id: String,
    #[description = "Task Name (afk etc)"] task: String,
    #[description = "Created At"] created_at: i64,
) -> Result<(), silverpelt::Error> {
    silverpelt::settings_poise::settings_updater(
        &ctx,
        &super::settings::LAST_TASK_EXPIRY,
        indexmap::indexmap! {
            "id".to_string() => Value::String(id),
            "task".to_string() => Value::String(task),
            "created_at".to_string() => {
                // Parse the date
                let created_at_utc = match chrono::Utc.timestamp_opt(created_at, 0) {
                    chrono::offset::LocalResult::Single(created_at_utc) => created_at_utc,
                    _ => return Err(format!("Invalid timestamp: {}", created_at).into()),
                };

                Value::TimestampTz(created_at_utc)
            },
        },
    )
    .await
}

#[poise::command(prefix_command)]
pub async fn last_task_expiry_delete(
    ctx: Context<'_>,
    #[description = "ID"] id: String,
) -> Result<(), silverpelt::Error> {
    silverpelt::settings_poise::settings_deleter(
        &ctx,
        &super::settings::LAST_TASK_EXPIRY,
        Value::String(id),
    )
    .await
}

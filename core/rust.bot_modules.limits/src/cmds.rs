use silverpelt::Context;
use silverpelt::Error;
use splashcore_rs::value::Value;

pub async fn limits_autocomplete<'a>(
    ctx: Context<'_>,
    partial: &'a str,
) -> Vec<serenity::all::AutocompleteChoice<'a>> {
    silverpelt::settings_poise::standard_autocomplete(ctx, &super::settings::GUILD_LIMITS, partial)
        .await
}

/// Limits base command
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    subcommands("limits_view", "limits_add", "limits_update", "limits_remove")
)]
pub async fn limits(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// View the limits setup for this server
#[poise::command(prefix_command, slash_command, guild_only, rename = "view")]
pub async fn limits_view(ctx: Context<'_>) -> Result<(), Error> {
    silverpelt::settings_poise::settings_viewer(
        &ctx,
        &super::settings::GUILD_LIMITS,
        indexmap::IndexMap::new(),
    )
    .await
}

/// Add a limit
#[poise::command(prefix_command, slash_command, guild_only, rename = "add")]
pub async fn limits_add(
    ctx: Context<'_>,
    #[description = "The name of the limit"] limit_name: String,
    #[description = "The type of limit to impose on moderators"]
    limit_type: super::core::LimitTypesChoices,
    #[description = "The amount of times the limit can be hit"] limit_per: i32,
    #[description = "The time interval infractions are counted in"] limit_time: i64,
    #[description = "The time unit for the time interval [seconds/minutes/hours/days]"]
    limit_time_unit: splashcore_rs::utils::Unit,
    #[description = "The number of stings to give on hitting the limit"] stings: i32,
) -> Result<(), Error> {
    silverpelt::settings_poise::settings_creator(
        &ctx,
        &super::settings::GUILD_LIMITS,
        indexmap::indexmap! {
            "limit_name".to_string() => Value::String(limit_name),
            "limit_type".to_string() => Value::String(limit_type.resolve().to_string()),
            "limit_per".to_string() => Value::Integer(limit_per.into()),
            "limit_time".to_string() => Value::Interval(chrono::Duration::seconds(limit_time * limit_time_unit.to_seconds_i64())),
            "stings".to_string() => Value::Integer(stings.into()),
        },
    )
    .await
}

/// Update an existing limit
#[poise::command(prefix_command, slash_command, guild_only, rename = "update")]
#[allow(clippy::too_many_arguments)]
pub async fn limits_update(
    ctx: Context<'_>,
    #[description = "The ID of the limit"] limit_id: String,
    #[description = "The name of the limit"] limit_name: String,
    #[description = "The type of limit to impose on moderators"]
    limit_type: super::core::LimitTypesChoices,
    #[description = "The amount of times the limit can be hit"] limit_per: i32,
    #[description = "The time interval infractions are counted in"] limit_time: i64,
    #[description = "The time unit for the time interval [seconds/minutes/hours/days]"]
    limit_time_unit: splashcore_rs::utils::Unit,
    #[description = "The number of stings to give on hitting the limit"] stings: i32,
) -> Result<(), Error> {
    silverpelt::settings_poise::settings_updater(
        &ctx,
        &super::settings::GUILD_LIMITS,
        indexmap::indexmap! {
            "limit_id".to_string() => Value::String(limit_id),
            "limit_name".to_string() => Value::String(limit_name),
            "limit_type".to_string() => Value::String(limit_type.resolve().to_string()),
            "limit_per".to_string() => Value::Integer(limit_per.into()),
            "limit_time".to_string() => Value::Interval(chrono::Duration::seconds(limit_time * limit_time_unit.to_seconds_i64())),
            "stings".to_string() => Value::Integer(stings.into()),
        },
    )
    .await
}

/// Remove a limit from the server
#[poise::command(prefix_command, slash_command, guild_only, rename = "remove")]
pub async fn limits_remove(
    ctx: Context<'_>,
    #[description = "The limit id to remove"]
    #[autocomplete = "limits_autocomplete"]
    limit_id: String,
) -> Result<(), Error> {
    silverpelt::settings_poise::settings_deleter(
        &ctx,
        &super::settings::GUILD_LIMITS,
        Value::String(limit_id),
    )
    .await
}

/// Limits User stings base command
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    subcommands("limits_user_stings_view", "limits_user_stings_remove",)
)]
pub async fn limits_user_stings(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// View all user stings due to limits
#[poise::command(prefix_command, slash_command, guild_only, rename = "view")]
pub async fn limits_user_stings_view(
    ctx: Context<'_>,
    #[description = "The user id to filter by"] user_id: Option<serenity::all::UserId>,
) -> Result<(), Error> {
    silverpelt::settings_poise::settings_viewer(&ctx, &super::settings::USER_STINGS, {
        let mut map = indexmap::IndexMap::new();
        if let Some(user_id) = user_id {
            map.insert("user_id".to_string(), Value::String(user_id.to_string()));
        }
        map
    })
    .await
}

/// Remove a user sting caused by a limit
#[poise::command(prefix_command, slash_command, guild_only, rename = "remove")]
pub async fn limits_user_stings_remove(
    ctx: Context<'_>,
    #[description = "The limit user sting ID to remove"] limit_user_sting_id: String,
) -> Result<(), Error> {
    silverpelt::settings_poise::settings_deleter(
        &ctx,
        &super::settings::USER_STINGS,
        Value::String(limit_user_sting_id),
    )
    .await
}

/// Limit user actions base command
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    subcommands("limit_user_actions_view", "limit_user_actions_remove",)
)]
pub async fn limit_user_actions(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// View all user actions recorded
#[poise::command(prefix_command, slash_command, guild_only, rename = "view")]
pub async fn limit_user_actions_view(
    ctx: Context<'_>,
    #[description = "The user id to filter by"] user_id: Option<serenity::all::UserId>,
) -> Result<(), Error> {
    silverpelt::settings_poise::settings_viewer(&ctx, &super::settings::USER_ACTIONS, {
        let mut map = indexmap::IndexMap::new();
        if let Some(user_id) = user_id {
            map.insert("user_id".to_string(), Value::String(user_id.to_string()));
        }
        map
    })
    .await
}

/// Remove a user action by ID
#[poise::command(prefix_command, slash_command, guild_only, rename = "remove")]
pub async fn limit_user_actions_remove(
    ctx: Context<'_>,
    #[description = "The user action ID to remove"] user_action_id: String,
) -> Result<(), Error> {
    silverpelt::settings_poise::settings_deleter(
        &ctx,
        &super::settings::USER_ACTIONS,
        Value::String(user_action_id),
    )
    .await
}

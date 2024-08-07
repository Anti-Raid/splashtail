use base_data::Error;
use silverpelt::Context;
use splashcore_rs::value::Value;

/// AFK base command
#[poise::command(
    prefix_command,
    slash_command,
    guild_cooldown = 10,
    subcommands("afk_list", "afk_create", "afk_update", "afk_delete",)
)]
pub async fn afk(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// View AFKs of a user, defaults to author
#[poise::command(prefix_command, slash_command, guild_cooldown = 10, rename = "list")]
pub async fn afk_list(
    ctx: Context<'_>,
    #[description = "User to view AFKs for"] user: Option<serenity::all::UserId>,
) -> Result<(), Error> {
    silverpelt::settings_poise::settings_viewer(
        &crate::SILVERPELT_CACHE,
        &ctx,
        &super::settings::AFKS,
        indexmap::indexmap! {
            "user_id".to_string() => Value::String(user.unwrap_or(ctx.author().id).to_string()),
        },
    )
    .await
}

/// Creates a new AFK
#[poise::command(prefix_command, slash_command, guild_cooldown = 10, rename = "create")]
pub async fn afk_create(
    ctx: Context<'_>,
    #[description = "The reason for being AFK"] reason: String,
    #[description = "The time interval to be AFK for"] time: i64,
    #[description = "The time unit for the time interval [seconds/minutes/hours/days]"]
    time_unit: splashcore_rs::utils::Unit,
) -> Result<(), Error> {
    silverpelt::settings_poise::settings_creator(
        &crate::SILVERPELT_CACHE,
        &ctx,
        &super::settings::AFKS,
        indexmap::indexmap! {
            "reason".to_string() => Value::String(reason),
            "expires_at".to_string() => {
                let duration = chrono::Duration::seconds(time * time_unit.to_seconds_i64());
                Value::TimestampTz(chrono::Utc::now() + duration)
            }
        },
    )
    .await
}

/// Updates an existing AFK
#[poise::command(prefix_command, slash_command, guild_cooldown = 10, rename = "update")]
pub async fn afk_update(
    ctx: Context<'_>,
    #[description = "The ID of the AFK"] id: String,
    #[description = "The reason for being AFK"] reason: String,
    #[description = "The time interval to be AFK for"] time: i64,
    #[description = "The time unit for the time interval [seconds/minutes/hours/days]"]
    time_unit: splashcore_rs::utils::Unit,
) -> Result<(), Error> {
    silverpelt::settings_poise::settings_updater(
        &crate::SILVERPELT_CACHE,
        &ctx,
        &super::settings::AFKS,
        indexmap::indexmap! {
            "id".to_string() => Value::String(id),
            "reason".to_string() => Value::String(reason),
            "expires_at".to_string() => {
                let duration = chrono::Duration::seconds(time * time_unit.to_seconds_i64());
                Value::TimestampTz(chrono::Utc::now() + duration)
            }
        },
    )
    .await
}

/// Deletes an existing AFK
#[poise::command(prefix_command, slash_command, guild_cooldown = 10, rename = "delete")]
pub async fn afk_delete(
    ctx: Context<'_>,
    #[description = "The ID of the AFK"] id: String,
) -> Result<(), Error> {
    silverpelt::settings_poise::settings_deleter(
        &crate::SILVERPELT_CACHE,
        &ctx,
        &super::settings::AFKS,
        Value::String(id),
    )
    .await
}

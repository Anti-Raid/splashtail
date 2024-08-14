use serenity::all::Role;
use silverpelt::Context;
use silverpelt::Error;
use splashcore_rs::value::Value;

#[poise::command(
    prefix_command,
    slash_command,
    user_cooldown = 1,
    guild_cooldown = 1,
    subcommands(
        "guildroles_list",
        "guildroles_add",
        "guildroles_edit",
        "guildroles_remove"
    )
)]
pub async fn guildroles(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Lists all roles with the setup permission and index
#[poise::command(
    prefix_command,
    slash_command,
    user_cooldown = 1,
    guild_cooldown = 1,
    rename = "list"
)]
pub async fn guildroles_list(ctx: Context<'_>) -> Result<(), Error> {
    silverpelt::settings_poise::settings_viewer(
        &ctx,
        &super::settings::GUILD_ROLES,
        indexmap::IndexMap::new(),
    )
    .await
}

/// Adds a new role with specific permissions
#[poise::command(
    prefix_command,
    slash_command,
    user_cooldown = 1,
    guild_cooldown = 1,
    rename = "add"
)]
pub async fn guildroles_add(
    ctx: Context<'_>,
    #[description = "The role to add"] role: Role,
    #[description = "The permissions to set, separated by commas"] perms: String,
    #[description = "The index of the role"] index: Option<i32>,
) -> Result<(), Error> {
    silverpelt::settings_poise::settings_creator(
        &ctx,
        &super::settings::GUILD_ROLES,
        indexmap::indexmap! {
            "role_id".to_string() => Value::String(role.id.to_string()),
            "perms".to_string() => Value::String(perms),
            "index".to_string() => index.map(|x| Value::Integer(x as i64)).unwrap_or(Value::None),
        },
    )
    .await
}

/// Edits an existing roles' permissions
#[poise::command(
    prefix_command,
    slash_command,
    user_cooldown = 1,
    guild_cooldown = 1,
    rename = "edit"
)]
pub async fn guildroles_edit(
    ctx: Context<'_>,
    #[description = "The role to edit"] role: Role,
    #[description = "The permissions to set, separated by commas"] perms: String,
    #[description = "The index of the role"] index: Option<i32>,
) -> Result<(), Error> {
    silverpelt::settings_poise::settings_updater(
        &ctx,
        &super::settings::GUILD_ROLES,
        indexmap::indexmap! {
            "role_id".to_string() => Value::String(role.id.to_string()),
            "perms".to_string() => Value::String(perms),
            "index".to_string() => index.map(|x| Value::Integer(x as i64)).unwrap_or(Value::None),
        },
    )
    .await
}

/// Edits an existing roles' permissions
#[poise::command(
    prefix_command,
    slash_command,
    user_cooldown = 1,
    guild_cooldown = 1,
    rename = "remove"
)]
pub async fn guildroles_remove(
    ctx: Context<'_>,
    #[description = "The role to remove"] role: Role,
) -> Result<(), Error> {
    silverpelt::settings_poise::settings_deleter(
        &ctx,
        &super::settings::GUILD_ROLES,
        Value::String(role.id.to_string()),
    )
    .await
}

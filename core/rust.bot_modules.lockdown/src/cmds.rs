use silverpelt::Context;
use silverpelt::Error;
use splashcore_rs::utils::{parse_numeric_list, REPLACE_ROLE};
use splashcore_rs::value::Value;

async fn lockdown_autocomplete<'a>(
    ctx: Context<'_>,
    partial: &'a str,
) -> Vec<serenity::all::AutocompleteChoice<'a>> {
    silverpelt::settings_poise::standard_autocomplete(ctx, &super::settings::LOCKDOWNS, partial).await
}

/// Configure the common lockdown settings for this server
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    subcommands(
        "lockdown_settings_view",
        "lockdown_settings_create",
        "lockdown_settings_update",
        "lockdown_settings_delete"
    )
)]
pub async fn lockdown_settings(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// View the lockdown settings currently configured
#[poise::command(prefix_command, slash_command, guild_only, rename = "view")]
async fn lockdown_settings_view(ctx: Context<'_>) -> Result<(), Error> {
    silverpelt::settings_poise::settings_viewer(
        &ctx,
        &super::settings::LOCKDOWN_SETTINGS,
        indexmap::IndexMap::new(),
    )
    .await
}

/// Create new lockdown settings currently configured
#[poise::command(prefix_command, slash_command, guild_only, rename = "create")]
async fn lockdown_settings_create(
    ctx: Context<'_>,
    #[description = "The member roles to apply lockdown to"] member_roles: Option<String>,
    #[description = "Require correct server layout to ensure complete lockdown"]
    require_correct_layout: Option<bool>,
) -> Result<(), Error> {
    silverpelt::settings_poise::settings_creator(
        &ctx,
        &super::settings::LOCKDOWN_SETTINGS,
        indexmap::indexmap! {
            "guild_id".to_string() => {
                let Some(guild_id) = ctx.guild_id() else {
                    return Err("This command must be run in a server".into());
                };

                Value::String(guild_id.to_string())
            },
            "member_roles".to_string() => {
                if let Some(member_roles) = member_roles {
                    let member_roles = parse_numeric_list::<serenity::all::RoleId>(
                        &member_roles,
                        &REPLACE_ROLE
                    )?;
    
                    Value::List(member_roles.into_iter().map(|r| Value::String(r.to_string())).collect())
                } else {
                    Value::List(Vec::new())
                }
            },
            "require_correct_layout".to_string() => Value::Boolean(require_correct_layout.unwrap_or_default()),
        },
    )
    .await
}

/// Updates an existing server lockdown configuration
#[poise::command(prefix_command, slash_command, guild_only, rename = "update")]
async fn lockdown_settings_update(
    ctx: Context<'_>,
    #[description = "The member roles to apply lockdown to"] member_roles: Option<String>,
    #[description = "Require correct server layout to ensure complete lockdown"]
    require_correct_layout: Option<bool>,
) -> Result<(), Error> {
    silverpelt::settings_poise::settings_updater(
        &ctx,
        &super::settings::LOCKDOWN_SETTINGS,
        indexmap::indexmap! {
            "guild_id".to_string() => {
                let Some(guild_id) = ctx.guild_id() else {
                    return Err("This command must be run in a server".into());
                };

                Value::String(guild_id.to_string())
            },
            "member_roles".to_string() => {
                if let Some(member_roles) = member_roles {
                    let member_roles = parse_numeric_list::<serenity::all::RoleId>(
                        &member_roles,
                        &REPLACE_ROLE
                    )?;
    
                    Value::List(member_roles.into_iter().map(|r| Value::String(r.to_string())).collect())
                } else {
                    Value::List(Vec::new())
                }
            },
            "require_correct_layout".to_string() => Value::Boolean(require_correct_layout.unwrap_or_default()),
        },
    )
    .await
}

/// Deletes a server lockdown configuration
#[poise::command(prefix_command, slash_command, guild_only, rename = "delete")]
async fn lockdown_settings_delete(
    ctx: Context<'_>,
) -> Result<(), Error> {
    silverpelt::settings_poise::settings_deleter(
        &ctx,
        &super::settings::LOCKDOWN_SETTINGS,
        {
            let Some(guild_id) = ctx.guild_id() else {
                return Err("This command must be run in a server".into());
            };

            Value::String(guild_id.to_string())
        }
    )
    .await
}

/// Configure lockdowns
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    subcommands(
        "lockdown_list",
        "lockdown_lock",
        "lockdown_unlock",
    )
)]
pub async fn lockdown(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// List all current lockdowns
#[poise::command(prefix_command, slash_command, guild_only, rename = "list")]
async fn lockdown_list(ctx: Context<'_>) -> Result<(), Error> {
    silverpelt::settings_poise::settings_viewer(
        &ctx,
        &super::settings::LOCKDOWNS,
        indexmap::IndexMap::new(),
    )
    .await
}

/// Start a quick server lockdown
#[poise::command(prefix_command, slash_command, guild_only, rename = "lock")]
async fn lockdown_lock(
    ctx: Context<'_>,
    #[description = "The type for the lockdown"] r#type: String,
    #[description = "The reason for the lockdown"] reason: String,
) -> Result<(), Error> {
    silverpelt::settings_poise::settings_creator(
        &ctx,
        &super::settings::LOCKDOWNS,
        indexmap::indexmap! {
            "guild_id".to_string() => {
                let Some(guild_id) = ctx.guild_id() else {
                    return Err("This command must be run in a server".into());
                };

                Value::String(guild_id.to_string())
            },
            "type".to_string() => Value::String(r#type),
            "reason".to_string() => Value::String(reason),
        },
    )
    .await
}

/// Unlock/revert a lockdown
#[poise::command(prefix_command, slash_command, guild_only, rename = "unlock")]
async fn lockdown_unlock(
    ctx: Context<'_>,
    #[description = "The lockdown to unlock"] 
    #[autocomplete = "lockdown_autocomplete"]
    id: String,
) -> Result<(), Error> {
    silverpelt::settings_poise::settings_deleter(
        &ctx,
        &super::settings::LOCKDOWNS,
        Value::String(id)
    )
    .await
}

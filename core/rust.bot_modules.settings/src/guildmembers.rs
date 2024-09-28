use serenity::all::UserId;
use silverpelt::Context;
use silverpelt::Error;
use splashcore_rs::value::Value;

#[poise::command(
    prefix_command,
    slash_command,
    user_cooldown = 1,
    guild_cooldown = 1,
    subcommands(
        "guildmembers_list",
        "guildmembers_add",
        "guildmembers_edit",
        "guildmembers_remove"
    )
)]
pub async fn guildmembers(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Lists all members registered with AntiRaid
#[poise::command(
    prefix_command,
    slash_command,
    user_cooldown = 1,
    guild_cooldown = 1,
    rename = "list"
)]
pub async fn guildmembers_list(ctx: Context<'_>) -> Result<(), Error> {
    silverpelt::settings_poise::settings_viewer(
        &ctx,
        &super::settings::GUILD_MEMBERS,
        indexmap::IndexMap::new(),
    )
    .await
}

/// Adds a new member
#[poise::command(
    prefix_command,
    slash_command,
    user_cooldown = 1,
    guild_cooldown = 1,
    rename = "add"
)]
pub async fn guildmembers_add(
    ctx: Context<'_>,
    #[description = "The user to add"] user_id: UserId,
    #[description = "The permissions overrides, separated by commas"] perm_overrides: Option<String>,
) -> Result<(), Error> {
    silverpelt::settings_poise::settings_creator(
        &ctx,
        &super::settings::GUILD_MEMBERS,
        indexmap::indexmap! {
            "user_id".to_string() => Value::String(user_id.to_string()),
            "perm_overrides".to_string() => {
                match perm_overrides {
                    None => Value::List(vec![]),
                    Some(x) => {
                        let split = x.split(',').map(|x| x.trim().to_string()).collect::<Vec<String>>();

                        Value::List(split.into_iter().map(|x| Value::String(x)).collect())        
                    },
                }
            },
        },
    )
    .await
}

/// Edits an existing member' permissions
#[poise::command(
    prefix_command,
    slash_command,
    user_cooldown = 1,
    guild_cooldown = 1,
    rename = "edit"
)]
pub async fn guildmembers_edit(
    ctx: Context<'_>,
    #[description = "The user to edit"] user_id: UserId,
    #[description = "The permissions overrides, separated by commas"] perm_overrides: Option<String>,
) -> Result<(), Error> {
    silverpelt::settings_poise::settings_updater(
        &ctx,
        &super::settings::GUILD_MEMBERS,
        indexmap::indexmap! {
            "user_id".to_string() => Value::String(user_id.to_string()),
            "perm_overrides".to_string() => {
                match perm_overrides {
                    None => Value::List(vec![]),
                    Some(x) => {
                        let split = x.split(',').map(|x| x.trim().to_string()).collect::<Vec<String>>();

                        Value::List(split.into_iter().map(|x| Value::String(x)).collect())        
                    },
                }
            },
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
pub async fn guildmembers_remove(
    ctx: Context<'_>,
    #[description = "The user to remove"] user_id: UserId,
) -> Result<(), Error> {
    silverpelt::settings_poise::settings_deleter(
        &ctx,
        &super::settings::GUILD_MEMBERS,
        Value::String(user_id.to_string()),
    )
    .await
}

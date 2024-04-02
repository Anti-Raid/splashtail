use bothelpers::cache::CacheHttpImpl;
use crate::{Error, Context};

use crate::silverpelt::silverpelt_cache::SILVERPELT_CACHE;

/// Settings related to commands
#[poise::command(
    prefix_command,
    slash_command,
    user_cooldown = 1,
    guild_cooldown = 1,
    subcommands("commands_check", "commands_enable", "commands_disable")
)]
pub async fn commands(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Checks if a command is usable
#[poise::command(
    prefix_command,
    slash_command,
    user_cooldown = 1,
    guild_cooldown = 1,
    rename = "check"
)]
pub async fn commands_check(
    ctx: Context<'_>,
    #[description = "The command to check"]
    command: String,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command must be run in a guild".into());
    };

    let base_command = command.split_whitespace().next().unwrap();

    // Find associated module
    /*let Some(module) = SILVERPELT_CACHE.command_id_module_map.get(base_command) else {
        return Err("Command not found".into());
    };*/

    // Check if the user has permission to use the command
    let cache_http = &CacheHttpImpl::from_ctx(ctx.serenity_context());
    let perm_res = crate::silverpelt::cmd::check_command(
        base_command,
        &command,
        guild_id,
        ctx.author().id,
        &ctx.data().pool,
        cache_http,
        &Some(ctx),
        crate::silverpelt::cmd::CheckCommandOptions {
            ignore_command_disabled: true,
            ..Default::default()
        },
    ).await;

    if !perm_res.is_ok() {
        return Err(format!("You do NOT have permission to use this command?\n{}", perm_res.to_markdown()).into());
    }

    Ok(())
}

/// Enables a module. Note that globally disabled modules cannot be used even if enabled
#[poise::command(
    prefix_command,
    slash_command,
    user_cooldown = 1,
    guild_cooldown = 1,
    rename = "enable"
)]
pub async fn commands_enable(
    ctx: Context<'_>,
    #[description = "The command to enable"]
    command: String,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command must be run in a guild".into());
    };

    let base_command = command.split_whitespace().next().unwrap();

    // Find associated module
    /*let Some(module) = SILVERPELT_CACHE.command_id_module_map.get(base_command) else {
        return Err("Command not found".into());
    };*/

    // Check if the user has permission to use the command
    let cache_http = &CacheHttpImpl::from_ctx(ctx.serenity_context());
    let perm_res = crate::silverpelt::cmd::check_command(
        base_command,
        &command,
        guild_id,
        ctx.author().id,
        &ctx.data().pool,
        cache_http,
        &Some(ctx),
        crate::silverpelt::cmd::CheckCommandOptions {
            ignore_command_disabled: true,
            ..Default::default()
        },
    ).await;

    if !perm_res.is_ok() {
        return Err(format!("You can only modify commands that you have permission to use?\n{}", perm_res.to_markdown()).into());
    }

    // Check if command is already enabled
    let mut tx = ctx.data().pool.begin().await?;

    let disabled = sqlx::query!(
        "SELECT disabled FROM guild_command_configurations WHERE guild_id = $1 AND command = $2",
        guild_id.to_string(),
        command
    )
    .fetch_optional(&mut *tx)
    .await?;

    if let Some(disabled) = disabled {
        // We have a module, now check
        if disabled.disabled.is_some() && !disabled.disabled.unwrap_or_default() {
            return Err("Command is already enabled".into());
        }

        sqlx::query!(
            "UPDATE guild_command_configurations SET disabled = false WHERE guild_id = $1 AND command = $2",
            guild_id.to_string(),
            command
        )
        .execute(&mut *tx)
        .await?;
    } else {
        // No module, create it
        sqlx::query!(
            "INSERT INTO guild_command_configurations (guild_id, command, disabled) VALUES ($1, $2, false)",
            guild_id.to_string(),
            command
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    tokio::spawn(async move {
        if let Err(err) = SILVERPELT_CACHE
            .command_permission_cache
            .invalidate_entries_if(move |k, _| k.0 == guild_id)
        {
            log::error!(
                "Failed to invalidate command permission cache for guild {}: {}",
                guild_id,
                err
            );
        } else {
            log::info!("Invalidated cache for guild {}", guild_id);
        }
    });

    Ok(())
}

/// Enables a module. Note that globally disabled modules cannot be used even if enabled
#[poise::command(
    prefix_command,
    slash_command,
    user_cooldown = 1,
    guild_cooldown = 1,
    rename = "disable"
)]
pub async fn commands_disable(
    ctx: Context<'_>,
    #[description = "The command to disable"]
    command: String,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command must be run in a guild".into());
    };

    let base_command = command.split_whitespace().next().unwrap();

    // Find associated module
    /*let Some(module) = SILVERPELT_CACHE.command_id_module_map.get(base_command) else {
        return Err("Command not found".into());
    };*/

    // Check if the user has permission to use the command
    let cache_http = &CacheHttpImpl::from_ctx(ctx.serenity_context());
    let perm_res = crate::silverpelt::cmd::check_command(
        base_command,
        &command,
        guild_id,
        ctx.author().id,
        &ctx.data().pool,
        cache_http,
        &Some(ctx),
        crate::silverpelt::cmd::CheckCommandOptions {
            ignore_command_disabled: true,
            ..Default::default()
        },
    ).await;

    if !perm_res.is_ok() {
        return Err(format!("You can only modify commands that you have permission to use?\n{}", perm_res.to_markdown()).into());
    }

    // Check if command is already enabled
    let mut tx = ctx.data().pool.begin().await?;

    let disabled = sqlx::query!(
        "SELECT disabled FROM guild_command_configurations WHERE guild_id = $1 AND command = $2",
        guild_id.to_string(),
        command
    )
    .fetch_optional(&mut *tx)
    .await?;

    if let Some(disabled) = disabled {
        // We have a command, now check
        if disabled.disabled.is_some() && disabled.disabled.unwrap_or_default() {
            return Err("Command is already disabled".into());
        }

        sqlx::query!(
            "UPDATE guild_command_configurations SET disabled = true WHERE guild_id = $1 AND command = $2",
            guild_id.to_string(),
            command
        )
        .execute(&mut *tx)
        .await?;
    } else {
        // No module, create it
        sqlx::query!(
            "INSERT INTO guild_command_configurations (guild_id, command, disabled) VALUES ($1, $2, true)",
            guild_id.to_string(),
            command
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    tokio::spawn(async move {
        if let Err(err) = SILVERPELT_CACHE
            .command_permission_cache
            .invalidate_entries_if(move |k, _| k.0 == guild_id)
        {
            log::error!(
                "Failed to invalidate command permission cache for guild {}: {}",
                guild_id,
                err
            );
        } else {
            log::info!("Invalidated cache for guild {}", guild_id);
        }
    });

    Ok(())
}
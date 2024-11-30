use serenity::all::{AutocompleteChoice, CreateAutocompleteResponse};
use silverpelt::Context;
use silverpelt::Error;

async fn module_list_autocomplete<'a>(
    ctx: Context<'_>,
    partial: &'a str,
) -> CreateAutocompleteResponse<'a> {
    let data = ctx.data();
    let mut ac = Vec::new();

    for refs in data.silverpelt_cache.module_cache.iter() {
        let module = refs.value();
        if module
            .name()
            .to_lowercase()
            .contains(&partial.to_lowercase())
            || module.id().to_lowercase().contains(&partial.to_lowercase())
        {
            ac.push(AutocompleteChoice::new(module.name(), module.id()));
        }
    }

    CreateAutocompleteResponse::new().set_choices(ac)
}

#[poise::command(
    slash_command,
    user_cooldown = 1,
    guild_cooldown = 1,
    subcommands("modules_list", "modules_enable", "modules_disable",)
)]
pub async fn modules(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Lists all module configurations currently setup
#[poise::command(slash_command, user_cooldown = 1, guild_cooldown = 1, rename = "list")]
pub async fn modules_list(ctx: Context<'_>) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command must be run in a guild".into());
    };

    let data = ctx.data();

    let module_configs = sqlx::query!(
        "SELECT module, disabled FROM guild_module_configurations WHERE guild_id = $1",
        guild_id.to_string()
    )
    .fetch_all(&data.pool)
    .await?;

    let mut msg = "**Module Configurations**\n\n".to_string();

    let mut done_modules = Vec::new();
    for module_config in module_configs {
        let Some(module) = data
            .silverpelt_cache
            .module_cache
            .get(&module_config.module)
        else {
            continue;
        };

        let module_id = module_config.module;

        if let Some(disabled) = module_config.disabled {
            msg.push_str(&format!(
                "**{}**: {} [module id = {}]\n",
                module.name(),
                if disabled { "Disabled" } else { "Enabled" },
                module_id
            ));
        } else {
            msg.push_str(&format!(
                "**{}**: {} [default] [module id = {}]\n",
                module.name(),
                if module.is_default_enabled() {
                    "Enabled"
                } else {
                    "Disabled"
                },
                module_id
            ));
        }

        done_modules.push(module_id);
    }

    for refs in data.silverpelt_cache.module_cache.iter() {
        let module = refs.value();
        if done_modules.contains(&module.id().to_string()) {
            continue;
        }

        msg.push_str(&format!(
            "**{}**: {} [default, config not modified] [module id = {}]\n",
            module.name(),
            if module.is_default_enabled() {
                "Enabled"
            } else {
                "Disabled"
            },
            module.id()
        ));
    }

    ctx.say(msg).await?;

    Ok(())
}

/// Enables a module. Note that globally disabled modules cannot be used even if enabled
#[poise::command(
    slash_command,
    user_cooldown = 1,
    guild_cooldown = 1,
    rename = "enable"
)]
pub async fn modules_enable(
    ctx: Context<'_>,
    #[description = "The module to enable"]
    #[autocomplete = "module_list_autocomplete"]
    module: String,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command must be run in a guild".into());
    };

    let data = ctx.data();

    // Check that the module exists
    let Some(module_data) = data.silverpelt_cache.module_cache.get(&module) else {
        return Err(format!(
            "The module you are trying to disable ({}) does not exist",
            module
        )
        .into());
    };

    if !module_data.toggleable() {
        return Err("This module cannot be enabled/disabled".into());
    }

    // Check for a module_configuration in db
    // If it doesn't exist, create it
    let mut tx = data.pool.begin().await?;

    let disabled = sqlx::query!(
        "SELECT disabled FROM guild_module_configurations WHERE guild_id = $1 AND module = $2 FOR UPDATE",
        guild_id.to_string(),
        module
    )
    .fetch_optional(&mut *tx)
    .await?;

    if let Some(disabled) = disabled {
        // We have a module, now check
        if disabled.disabled.is_some() && !disabled.disabled.unwrap_or_default() {
            return Err("Module is already enabled".into());
        }

        sqlx::query!(
            "UPDATE guild_module_configurations SET disabled = false WHERE guild_id = $1 AND module = $2",
            guild_id.to_string(),
            module
        )
        .execute(&mut *tx)
        .await?;
    } else {
        // No module, create it
        sqlx::query!(
            "INSERT INTO guild_module_configurations (guild_id, module, disabled) VALUES ($1, $2, false)",
            guild_id.to_string(),
            module
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    data.silverpelt_cache
        .module_enabled_cache
        .invalidate(&(guild_id, module))
        .await;

    ctx.say("Module enabled successfully!").await?;

    Ok(())
}

/// Disables a module. Note that certain modules may not be disablable
#[poise::command(
    slash_command,
    user_cooldown = 1,
    guild_cooldown = 1,
    rename = "disable"
)]
pub async fn modules_disable(
    ctx: Context<'_>,
    #[description = "The module to disable"]
    #[autocomplete = "module_list_autocomplete"]
    module: String,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command must be run in a guild".into());
    };

    let data = ctx.data();

    // Check that the module exists
    let Some(module_data) = data.silverpelt_cache.module_cache.get(&module) else {
        return Err(format!(
            "The module you are trying to disable ({}) does not exist",
            module
        )
        .into());
    };

    if !module_data.toggleable() {
        return Err("This module cannot be enabled/disabled".into());
    }

    // Check for a module_configuration in db
    // If it doesn't exist, create it
    let mut tx = data.pool.begin().await?;

    let disabled = sqlx::query!(
        "SELECT disabled FROM guild_module_configurations WHERE guild_id = $1 AND module = $2 FOR UPDATE",
        guild_id.to_string(),
        module
    )
    .fetch_optional(&mut *tx)
    .await?;

    if let Some(disabled) = disabled {
        // We have a module, now check
        if disabled.disabled.is_some() && disabled.disabled.unwrap_or_default() {
            return Err("Module is already disabled".into());
        }

        sqlx::query!(
            "UPDATE guild_module_configurations SET disabled = true WHERE guild_id = $1 AND module = $2",
            guild_id.to_string(),
            module
        )
        .execute(&mut *tx)
        .await?;
    } else {
        // No module, create it
        sqlx::query!(
            "INSERT INTO guild_module_configurations (guild_id, module, disabled) VALUES ($1, $2, true)",
            guild_id.to_string(),
            module
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    data.silverpelt_cache
        .module_enabled_cache
        .invalidate(&(guild_id, module))
        .await;

    ctx.say("Module disabled successfully!").await?;

    Ok(())
}

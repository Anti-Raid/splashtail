use silverpelt::Context;
use silverpelt::Error;
use splashcore_rs::value::Value;
use std::sync::LazyLock;

// To avoid recreating the dehoist options every time, use a lazy lock
static DEHOIST_OPTIONS: LazyLock<indexmap::IndexMap<String, i64>> = LazyLock::new(|| {
    super::types::DehoistOptions::all()
        .into_iter()
        .map(|x| (x.to_string(), x.bits() as i64))
        .collect()
});

static GUILD_PROTECTION_OPTIONS: LazyLock<indexmap::IndexMap<String, i64>> = LazyLock::new(|| {
    super::types::GuildProtectionOptions::all()
        .into_iter()
        .map(|x| (x.to_string(), x.bits() as i64))
        .collect()
});

static FAKE_BOT_DETECTION_OPTIONS: LazyLock<indexmap::IndexMap<String, i64>> =
    LazyLock::new(|| {
        super::types::FakeBotDetectionOptions::all()
            .into_iter()
            .map(|x| (x.to_string(), x.bits() as i64))
            .collect()
    });

#[inline]
pub fn convert_bitflags_string_to_value(
    values: &indexmap::IndexMap<String, i64>,
    input: Option<String>,
) -> Value {
    match input {
        Some(input) => {
            let mut bitflags = 0;

            for value in input.split(';') {
                if let Some(value) = values.get(value) {
                    bitflags |= *value;
                }
            }

            Value::Integer(bitflags)
        }
        None => Value::None,
    }
}

pub async fn hoist_detection_autocomplete<'a>(
    ctx: Context<'_>,
    partial: &'a str,
) -> Vec<serenity::all::AutocompleteChoice<'a>> {
    silverpelt::settings_poise::bitflag_autocomplete(ctx, &DEHOIST_OPTIONS, partial).await
}

pub async fn guild_protection_autocomplete<'a>(
    ctx: Context<'_>,
    partial: &'a str,
) -> Vec<serenity::all::AutocompleteChoice<'a>> {
    silverpelt::settings_poise::bitflag_autocomplete(ctx, &GUILD_PROTECTION_OPTIONS, partial).await
}

pub async fn fake_bot_detection_autocomplete<'a>(
    ctx: Context<'_>,
    partial: &'a str,
) -> Vec<serenity::all::AutocompleteChoice<'a>> {
    silverpelt::settings_poise::bitflag_autocomplete(ctx, &FAKE_BOT_DETECTION_OPTIONS, partial)
        .await
}

pub fn number_to_value(number: Option<i64>, default: Option<i64>) -> Value {
    if let Some(number) = number {
        Value::Integer(number)
    } else if let Some(default) = default {
        Value::Integer(default)
    } else {
        Value::None
    }
}

/// Inspector global options command
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = "5",
    subcommands(
        "inspector_global_list",
        "inspector_global_setup",
        "inspector_global_update",
        "inspector_global_delete"
    )
)]
pub async fn inspector_global(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// List inspector global settings
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = "5",
    rename = "list"
)]
pub async fn inspector_global_list(ctx: Context<'_>) -> Result<(), Error> {
    silverpelt::settings_poise::settings_viewer(
        &ctx,
        &super::settings::INSPECTOR_GLOBAL_OPTIONS,
        indexmap::IndexMap::new(),
    )
    .await
}

/// Setup inspector global settings (initially)
#[allow(clippy::too_many_arguments)]
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = "5",
    rename = "setup"
)]
pub async fn inspector_global_setup(
    ctx: Context<'_>,
    #[description = "Minimum Account Age"] minimum_account_age: Option<i64>,
    #[description = "Maximum Account Age"] maximum_account_age: Option<i64>,
    #[description = "Number of seconds to keep stings for. Defaults to 0"] sting_retention: Option<
        i64,
    >,
    #[description = "Hoist Detections Options"]
    #[autocomplete = "hoist_detection_autocomplete"]
    hoist_detection: Option<String>,
    #[description = "Guild Protection Options"]
    #[autocomplete = "guild_protection_autocomplete"]
    guild_protection: Option<String>,
    #[description = "Fake Bot Detection Options"]
    #[autocomplete = "fake_bot_detection_autocomplete"]
    fake_bot_detection: Option<String>,
) -> Result<(), Error> {
    silverpelt::settings_poise::settings_creator(
        &ctx,
        &super::settings::INSPECTOR_GLOBAL_OPTIONS,
        indexmap::indexmap! {
            "guild_id".to_string() => {
                if let Some(guild_id) = ctx.guild_id() {
                    Value::String(guild_id.to_string())
                } else {
                    return Err("Guild ID not found".into());
                }
            },
            "minimum_account_age".to_string() => number_to_value(minimum_account_age, None),
            "maximum_account_age".to_string() => number_to_value(maximum_account_age, None),
            "sting_retention".to_string() => number_to_value(sting_retention, Some(60)),
            "hoist_detection".to_string() => convert_bitflags_string_to_value(&DEHOIST_OPTIONS, hoist_detection),
            "guild_protection".to_string() => convert_bitflags_string_to_value(&GUILD_PROTECTION_OPTIONS, guild_protection),
            "fake_bot_detection".to_string() => convert_bitflags_string_to_value(&FAKE_BOT_DETECTION_OPTIONS, fake_bot_detection),
        },
    )
    .await
}

/// Update inspector global settings
#[allow(clippy::too_many_arguments)]
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = "5",
    rename = "update"
)]
pub async fn inspector_global_update(
    ctx: Context<'_>,
    #[description = "Minimum Account Age"] minimum_account_age: Option<i64>,
    #[description = "Maximum Account Age"] maximum_account_age: Option<i64>,
    #[description = "Number of seconds to keep stings for. Defaults to 0"] sting_retention: Option<
        i64,
    >,
    #[description = "Hoist Detections Options"]
    #[autocomplete = "hoist_detection_autocomplete"]
    hoist_detection: Option<String>,
    #[description = "Guild Protection Options"]
    #[autocomplete = "guild_protection_autocomplete"]
    guild_protection: Option<String>,
    #[description = "Fake Bot Detection Options"]
    #[autocomplete = "fake_bot_detection_autocomplete"]
    fake_bot_detection: Option<String>,
) -> Result<(), Error> {
    silverpelt::settings_poise::settings_updater(
        &ctx,
        &super::settings::INSPECTOR_GLOBAL_OPTIONS,
        indexmap::indexmap! {
            "guild_id".to_string() => {
                if let Some(guild_id) = ctx.guild_id() {
                    Value::String(guild_id.to_string())
                } else {
                    return Err("Guild ID not found".into());
                }
            },
            "minimum_account_age".to_string() => number_to_value(minimum_account_age, None),
            "maximum_account_age".to_string() => number_to_value(maximum_account_age, None),
            "sting_retention".to_string() => number_to_value(sting_retention, Some(60)),
            "hoist_detection".to_string() => convert_bitflags_string_to_value(&DEHOIST_OPTIONS, hoist_detection),
            "guild_protection".to_string() => convert_bitflags_string_to_value(&GUILD_PROTECTION_OPTIONS, guild_protection),
            "fake_bot_detection".to_string() => convert_bitflags_string_to_value(&FAKE_BOT_DETECTION_OPTIONS, fake_bot_detection),
        },
    )
    .await
}

/// Delete inspector global settings
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = "5",
    rename = "delete"
)]
pub async fn inspector_global_delete(ctx: Context<'_>) -> Result<(), Error> {
    silverpelt::settings_poise::settings_deleter(
        &ctx,
        &super::settings::INSPECTOR_GLOBAL_OPTIONS,
        {
            if let Some(guild_id) = ctx.guild_id() {
                Value::String(guild_id.to_string())
            } else {
                return Err("Guild ID not found".into());
            }
        },
    )
    .await
}

pub async fn inspector_specific_id_autocomplete<'a>(
    ctx: Context<'_>,
    partial: &'a str,
) -> Vec<serenity::all::AutocompleteChoice<'a>> {
    let Some(guild_id) = ctx.guild_id() else {
        return Vec::new();
    };

    let data = ctx.data();

    let Ok(configs) = super::cache::get_specific_configs(&data.pool, guild_id).await else {
        log::error!("Failed to get specific configs");
        return Vec::new();
    };

    let mut choices = Vec::new();

    for config in configs {
        if config.id.to_string().starts_with(partial) {
            choices.push(serenity::all::AutocompleteChoice::new(format!("ID: {}", config.id), config.id.to_string()));
        }
    }

    choices
}

/// Inspector specific options base command
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = "5",
    subcommands(
        "inspector_specific_list",
        "inspector_specific_create",
        "inspector_specific_update",
        "inspector_specific_delete"
    )
)]
pub async fn inspector_specific(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// List inspector specific settings
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = "5",
    rename = "list"
)]
pub async fn inspector_specific_list(ctx: Context<'_>) -> Result<(), Error> {
    silverpelt::settings_poise::settings_viewer(
        &ctx,
        &super::settings::INSPECTOR_SPECIFIC_OPTIONS,
        indexmap::IndexMap::new(),
    )
    .await
}

/// Create inspector specific settings
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = "5",
    rename = "create"
)]
pub async fn inspector_specific_create(
    ctx: Context<'_>,
    #[description = "Number of stings to give when an invite is sent, None means disabled"]
    anti_invite: Option<i64>,
    #[description = "Number of stings to give when an everyone ping is sent, 0 or none means disabled"]
    anti_everyone: Option<i64>,
    #[description = "Number of seconds to keep stings for. Defaults to 0"] sting_retention: Option<
        i64,
    >,
    #[description = "Modifiers to set, comma seperated"] modifiers: Option<String>,
) -> Result<(), Error> {
    silverpelt::settings_poise::settings_creator(
        &ctx,
        &super::settings::INSPECTOR_SPECIFIC_OPTIONS,
        indexmap::indexmap! {
            "guild_id".to_string() => {
                if let Some(guild_id) = ctx.guild_id() {
                    Value::String(guild_id.to_string())
                } else {
                    return Err("Guild ID not found".into());
                }
            },
            "anti_invite".to_string() => number_to_value(anti_invite, None),
            "anti_everyone".to_string() => number_to_value(anti_everyone, None),
            "sting_retention".to_string() => number_to_value(sting_retention, Some(60)),
            "modifier".to_string() => {
                let modifiers = splashcore_rs::utils::split_input_to_string(&modifiers.unwrap_or_default(), ",");
                let mut modifiers_value = Vec::new();

                for modifier in modifiers.iter() {
                    match splashcore_rs::modifier::Modifier::from_repr(modifier)? {
                        splashcore_rs::modifier::Modifier::Role(_) => return Err("Role modifiers are not allowed".into()),
                        _ => {}
                    }

                    modifiers_value.push(Value::String(modifier.to_string())); 
                }

                Value::List(modifiers_value)
            }
        },
    )
    .await
}

/// Update inspector specific settings
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = "5",
    rename = "update"
)]
pub async fn inspector_specific_update(
    ctx: Context<'_>,
    #[description = "Which specific settings to update"] 
    #[autocomplete = "inspector_specific_id_autocomplete"]
    id: String,
    #[description = "Number of stings to give when an invite is sent, None means disabled"]
    anti_invite: Option<i64>,
    #[description = "Number of stings to give when an everyone ping is sent, 0 or none means disabled"]
    anti_everyone: Option<i64>,
    #[description = "Number of seconds to keep stings for. Defaults to 0"] sting_retention: Option<
        i64,
    >,
    #[description = "Modifiers to set, comma seperated"] modifiers: Option<String>,
) -> Result<(), Error> {
    silverpelt::settings_poise::settings_updater(
        &ctx,
        &super::settings::INSPECTOR_SPECIFIC_OPTIONS,
        indexmap::indexmap! {
            "id".to_string() => Value::String(id),
            "guild_id".to_string() => {
                if let Some(guild_id) = ctx.guild_id() {
                    Value::String(guild_id.to_string())
                } else {
                    return Err("Guild ID not found".into());
                }
            },
            "anti_invite".to_string() => number_to_value(anti_invite, None),
            "anti_everyone".to_string() => number_to_value(anti_everyone, None),
            "sting_retention".to_string() => number_to_value(sting_retention, Some(60)),
            "modifier".to_string() => {
                let modifiers = splashcore_rs::utils::split_input_to_string(&modifiers.unwrap_or_default(), ",");
                let mut modifiers_value = Vec::new();

                for modifier in modifiers.iter() {
                    match splashcore_rs::modifier::Modifier::from_repr(modifier)? {
                        splashcore_rs::modifier::Modifier::Role(_) => return Err("Role modifiers are not allowed".into()),
                        _ => {}
                    }

                    modifiers_value.push(Value::String(modifier.to_string())); 
                }

                Value::List(modifiers_value)
            }
        },
    )
    .await
}

/// Delete inspector specific settings
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = "5",
    rename = "delete"
)]
pub async fn inspector_specific_delete(
    ctx: Context<'_>,
    #[description = "Which specific settings to delete"] 
    #[autocomplete = "inspector_specific_id_autocomplete"]
    id: String,
) -> Result<(), Error> {
    silverpelt::settings_poise::settings_deleter(
        &ctx,
        &super::settings::INSPECTOR_SPECIFIC_OPTIONS,
        Value::String(id),
    )
    .await
}

use base_data::Error;
use silverpelt::Context;
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
pub async fn bitflag_autocomplete<'a>(
    ctx: Context<'_>,
    values: &indexmap::IndexMap<String, i64>,
    partial: &'a str,
) -> Vec<serenity::all::AutocompleteChoice<'a>> {
    // Fetch all bitflags available
    let guild_id = ctx.guild_id();

    if guild_id.is_none() {
        return Vec::new();
    }

    let current_choices = partial
        .split(';')
        .map(|x| x.to_string())
        .collect::<Vec<String>>();

    let mut choices = Vec::with_capacity(std::cmp::max(values.len(), 25));

    for (label, _) in values {
        // We can abuse autocomplete to emulate a bitflag like setup
        if choices.len() > 25 {
            break;
        }

        if current_choices.contains(label) {
            continue;
        }

        let partial = partial.trim().trim_matches(';');

        if partial.is_empty() {
            choices.push(serenity::all::AutocompleteChoice::new(
                label.clone(),
                label.clone(),
            ));
            continue;
        }

        choices.push(serenity::all::AutocompleteChoice::new(
            format!("{};{}", partial, label),
            format!("{};{}", partial, label),
        ));
    }

    choices
}

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
    bitflag_autocomplete(ctx, &DEHOIST_OPTIONS, partial).await
}

pub async fn guild_protection_autocomplete<'a>(
    ctx: Context<'_>,
    partial: &'a str,
) -> Vec<serenity::all::AutocompleteChoice<'a>> {
    bitflag_autocomplete(ctx, &GUILD_PROTECTION_OPTIONS, partial).await
}

pub async fn fake_bot_detection_autocomplete<'a>(
    ctx: Context<'_>,
    partial: &'a str,
) -> Vec<serenity::all::AutocompleteChoice<'a>> {
    bitflag_autocomplete(ctx, &FAKE_BOT_DETECTION_OPTIONS, partial).await
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

/// Inspector base command
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = "5",
    subcommands(
        "inspector_list",
        "inspector_setup",
        "inspector_update",
        "inspector_disable"
    )
)]
pub async fn inspector(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// List inspector settings
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = "5",
    rename = "list"
)]
pub async fn inspector_list(ctx: Context<'_>) -> Result<(), Error> {
    silverpelt::settings_poise::settings_viewer(
        &crate::SILVERPELT_CACHE,
        &ctx,
        &super::settings::INSPECTOR_OPTIONS,
        indexmap::IndexMap::new(),
    )
    .await
}

/// Setup inspector
#[allow(clippy::too_many_arguments)]
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = "5",
    rename = "setup"
)]
pub async fn inspector_setup(
    ctx: Context<'_>,
    #[description = "Minimum Account Age"] minimum_account_age: Option<i64>,
    #[description = "Maximum Account Age"] maximum_account_age: Option<i64>,
    #[description = "Number of stings to give when an invite is sent, None means disabled"]
    anti_invite: Option<i64>,
    #[description = "Number of stings to give when an everyone ping is sent, 0 or none means disabled"]
    anti_everyone: Option<i64>,
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
        &crate::SILVERPELT_CACHE,
        &ctx,
        &super::settings::INSPECTOR_OPTIONS,
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
            "anti_invite".to_string() => number_to_value(anti_invite, None),
            "anti_everyone".to_string() => number_to_value(anti_everyone, None),
            "sting_retention".to_string() => number_to_value(sting_retention, Some(60)),
            "hoist_detection".to_string() => convert_bitflags_string_to_value(&DEHOIST_OPTIONS, hoist_detection),
            "guild_protection".to_string() => convert_bitflags_string_to_value(&GUILD_PROTECTION_OPTIONS, guild_protection),
            "fake_bot_detection".to_string() => convert_bitflags_string_to_value(&FAKE_BOT_DETECTION_OPTIONS, fake_bot_detection),
        },
    )
    .await
}

/// Update inspector settings
#[allow(clippy::too_many_arguments)]
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = "5",
    rename = "update"
)]
pub async fn inspector_update(
    ctx: Context<'_>,
    #[description = "Minimum Account Age"] minimum_account_age: Option<i64>,
    #[description = "Maximum Account Age"] maximum_account_age: Option<i64>,
    #[description = "Number of stings to give when an invite is sent, None means disabled"]
    anti_invite: Option<i64>,
    #[description = "Number of stings to give when an everyone ping is sent, 0 or none means disabled"]
    anti_everyone: Option<i64>,
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
        &crate::SILVERPELT_CACHE,
        &ctx,
        &super::settings::INSPECTOR_OPTIONS,
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
            "anti_invite".to_string() => number_to_value(anti_invite, None),
            "anti_everyone".to_string() => number_to_value(anti_everyone, None),
            "sting_retention".to_string() => number_to_value(sting_retention, Some(60)),
            "hoist_detection".to_string() => convert_bitflags_string_to_value(&DEHOIST_OPTIONS, hoist_detection),
            "guild_protection".to_string() => convert_bitflags_string_to_value(&GUILD_PROTECTION_OPTIONS, guild_protection),
            "fake_bot_detection".to_string() => convert_bitflags_string_to_value(&FAKE_BOT_DETECTION_OPTIONS, fake_bot_detection),
        },
    )
    .await
}

/// List inspector settings
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = "5",
    rename = "disable"
)]
pub async fn inspector_disable(ctx: Context<'_>) -> Result<(), Error> {
    silverpelt::settings_poise::settings_deleter(
        &crate::SILVERPELT_CACHE,
        &ctx,
        &super::settings::INSPECTOR_OPTIONS,
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

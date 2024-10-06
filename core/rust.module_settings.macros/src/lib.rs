use module_settings::types::{ColumnType, InnerColumnType, InnerColumnTypeStringKind};
use serenity::futures::FutureExt;

// HACK: Work around poise requiring function pointers
const INJECT_LOCALE: &str = "ru";
static COLUMN_CACHE: std::sync::LazyLock<dashmap::DashMap<u64, module_settings::types::Column>> =
    std::sync::LazyLock::new(dashmap::DashMap::new);

pub fn create_poise_commands_from_setting(
    module_id: &str,
    config_opt: &module_settings::types::ConfigOption,
) -> poise::Command<silverpelt::data::Data, silverpelt::Error> {
    let mut cmd = poise::Command::default();

    // Set base info
    cmd.name = config_opt.id.to_string();
    cmd.qualified_name = config_opt.id.to_string();
    cmd.description = Some(config_opt.description.to_string());
    cmd.guild_only = true;
    cmd.subcommand_required = true;
    cmd.category = Some(module_id.to_string());

    // Create base command
    async fn base_command(
        ctx: poise::Context<'_, silverpelt::data::Data, silverpelt::Error>,
    ) -> Result<(), poise::FrameworkError<'_, silverpelt::data::Data, silverpelt::Error>> {
        match ctx
            .send(
                poise::CreateReply::new()
                    .content(format!(
                        "This is the base command for `{}`",
                        ctx.command().name
                    ))
                    .ephemeral(true),
            )
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => return Err(poise::FrameworkError::new_command(ctx, Box::new(e))),
        }
    }

    cmd.prefix_action = Some(|p_ctx| {
        let ctx = poise::Context::Prefix(p_ctx);

        base_command(ctx).boxed()
    });

    cmd.slash_action = Some(|app_ctx| {
        let ctx = poise::Context::Application(app_ctx);

        base_command(ctx).boxed()
    });

    // Create subcommands
    for (operation_type, operation_specific) in config_opt.operations.iter() {
        let cmd_args = create_command_args_for_operation_type(config_opt, *operation_type);
    }

    cmd
}

fn get_choices_from_config_opt(column: &module_settings::types::Column) -> Vec<String> {
    // Get the choices from the column_type. Note that only string scalar columns can have choices
    match column.column_type {
        ColumnType::Scalar { ref column_type } => {
            match column_type {
                InnerColumnType::String { allowed_values, .. } => {
                    if allowed_values.is_empty() {
                        Vec::new()
                    } else {
                        allowed_values
                            .iter()
                            .map(|v| v.to_string())
                            .collect::<Vec<String>>()
                    }
                }
                _ => Vec::new(), // No other channel type can contain a scalar
            }
        }
        _ => Vec::new(),
    }
}

fn get_create_command_option_from_column_type<'a>(
    column: &module_settings::types::Column,
    cco: serenity::all::CreateCommandOption<'a>,
) -> serenity::all::CreateCommandOption<'a> {
    match column.column_type {
        ColumnType::Scalar { ref column_type } => {
            match column_type {
                InnerColumnType::Integer {} => cco.kind(serenity::all::CommandOptionType::Integer),
                InnerColumnType::Float {} => cco.kind(serenity::all::CommandOptionType::Number),
                InnerColumnType::Boolean {} => cco.kind(serenity::all::CommandOptionType::Boolean),
                InnerColumnType::String { kind, .. } => match kind {
                    InnerColumnTypeStringKind::Channel { .. } => {
                        cco.kind(serenity::all::CommandOptionType::Channel)
                    }
                    InnerColumnTypeStringKind::User { .. } => {
                        cco.kind(serenity::all::CommandOptionType::User)
                    }
                    InnerColumnTypeStringKind::Role { .. } => {
                        cco.kind(serenity::all::CommandOptionType::Role)
                    }
                    // Fallback to string
                    _ => cco.kind(serenity::all::CommandOptionType::String),
                },
                // Fallback to string
                _ => cco.kind(serenity::all::CommandOptionType::String),
            }
        }
        // Other types are not supported yet, fallback to string
        _ => cco.kind(serenity::all::CommandOptionType::String),
    }
}

fn create_command_args_for_operation_type(
    config_opt: &module_settings::types::ConfigOption,
    operation_type: module_settings::types::OperationType,
) -> Vec<poise::CommandParameter<silverpelt::data::Data, silverpelt::Error>> {
    let mut args = vec![];

    for column in config_opt.columns.iter() {
        // Check if we should ignore this column
        if column.ignored_for.contains(&operation_type) {
            continue;
        }

        // HACK: Bypass us not having full context by just serializing a 'pointer' to it in name_localization.
        // and storing as a global variable
        let mut ptr = 0;

        for i in 0..u64::MAX {
            if !COLUMN_CACHE.contains_key(&i) {
                ptr = i;
                break;
            }
        }

        // Store the column in the cache
        COLUMN_CACHE.insert(ptr, column.clone());

        let mut name_localizations = std::collections::HashMap::new();
        name_localizations.insert(INJECT_LOCALE.to_string(), ptr.to_string());

        // Create the new command parameter
        let new_command_param = poise::CommandParameter {
            name: column.id.to_string(),
            name_localizations,
            description_localizations: std::collections::HashMap::new(),
            description: Some(column.description.to_string()),
            required: !column.nullable,
            channel_types: {
                match column.column_type {
                    ColumnType::Scalar { ref column_type } => {
                        match column_type {
                            InnerColumnType::String { kind, .. } => match kind {
                                InnerColumnTypeStringKind::Channel { allowed_types, .. } => {
                                    Some(allowed_types.clone())
                                }
                                _ => None, // No other string kind contains a scalar
                            },
                            _ => None, // No other channel type contains a scalar
                        }
                    }
                    _ => None,
                }
            },
            choices: {
                let choices = get_choices_from_config_opt(column);

                choices
                    .into_iter()
                    .map(|v| poise::CommandParameterChoice {
                        name: v.into(),
                        localizations: std::collections::HashMap::new(),
                        __non_exhaustive: (),
                    })
                    .collect()
            },
            type_setter: {
                Some(|cco| {
                    #[allow(dead_code)]
                    #[derive(serde::Deserialize)]
                    struct RequiredCommandData {
                        name_localizations: std::collections::HashMap<String, String>,
                    }

                    let json_data = serde_json::to_value(&cco).unwrap();
                    let required_data: RequiredCommandData =
                        serde_json::from_value(json_data).unwrap();

                    let col_ptr = required_data.name_localizations.get(INJECT_LOCALE).unwrap();

                    let col_ptr = col_ptr.parse::<u64>().unwrap();

                    let column = COLUMN_CACHE.get(&col_ptr).unwrap();

                    let cco = get_create_command_option_from_column_type(column.value(), cco);
                    cco.name_localized(INJECT_LOCALE, column.id.to_string())
                })
            },
            autocomplete_callback: None,
            __non_exhaustive: (),
        };

        println!("{:#?}", new_command_param.create_as_slash_command_option());

        args.push(new_command_param);
    }

    args
}

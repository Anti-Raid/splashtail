use module_settings::types::{
    ColumnType, InnerColumnType, InnerColumnTypeStringKind, OperationType,
};
use serenity::futures::FutureExt;

/// String Error wrapper
struct StringErr(String);

impl std::fmt::Display for StringErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Debug for StringErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for StringErr {}

// HACK: Work around poise requiring function pointers
const INJECT_LOCALE: &str = "ru";
static COLUMN_CACHE: std::sync::LazyLock<dashmap::DashMap<u64, module_settings::types::Column>> =
    std::sync::LazyLock::new(dashmap::DashMap::new);

/// Base command callback used for the root command
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

/// In order to provide state to the subcommand callback, we need to wrap it in a struct and then pass it through custom_data
struct SubcommandCallbackWrapper {
    config_option: module_settings::types::ConfigOption,
    operation_type: OperationType,
}

/// Subcommand callback
/// Base command callback used for the root command
async fn subcommand_command(
    ctx: poise::Context<'_, silverpelt::data::Data, silverpelt::Error>,
) -> Result<(), poise::FrameworkError<'_, silverpelt::data::Data, silverpelt::Error>> {
    let Some(cwctx) = ctx
        .command()
        .custom_data
        .downcast_ref::<SubcommandCallbackWrapper>()
    else {
        return Err(poise::FrameworkError::new_command(
            ctx,
            Box::new(StringErr(
                "Failed to downcast custom_data to ConfigOption".to_string(),
            ))
            .into(),
        ));
    };

    // View is a special case, we just need to call settings viewer
    if cwctx.operation_type == OperationType::View {
        return silverpelt::settings_poise::settings_viewer(
            &ctx,
            &cwctx.config_option,
            indexmap::IndexMap::new(), // TODO: Add filtering in the future
        )
        .await
        .map_err(|e| poise::FrameworkError::new_command(ctx, Box::new(StringErr(e.to_string()))));
    }

    ctx.say(format!(
        "In subcommand for {} [{}]",
        cwctx.config_option.id, cwctx.operation_type
    ))
    .await
    .map_err(|e| poise::FrameworkError::new_command(ctx, Box::new(e)))?;

    Ok(()) // TODO
}

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

    cmd.prefix_action = Some(|p_ctx| {
        let ctx = poise::Context::Prefix(p_ctx);

        base_command(ctx).boxed()
    });

    cmd.slash_action = Some(|app_ctx| {
        let ctx = poise::Context::Application(app_ctx);

        base_command(ctx).boxed()
    });

    // Create subcommands
    cmd.subcommands
        .extend(create_poise_subcommands_from_setting(module_id, config_opt));

    cmd
}

pub fn create_poise_subcommands_from_setting(
    module_id: &str,
    config_opt: &module_settings::types::ConfigOption,
) -> Vec<poise::Command<silverpelt::data::Data, silverpelt::Error>> {
    let mut sub_cmds = Vec::new();
    // Create subcommands
    for (operation_type, operation_specific) in config_opt.operations.iter() {
        let mut sub_cmd = poise::Command::default();

        sub_cmd.name = operation_specific
            .corresponding_command
            .split(" ")
            .last()
            .unwrap()
            .to_string();
        sub_cmd.qualified_name = sub_cmd.name.clone();
        sub_cmd.parameters = create_command_args_for_operation_type(config_opt, *operation_type);

        match operation_type {
            OperationType::View => {
                sub_cmd.description = Some(format!("View {}", config_opt.id));
            }
            OperationType::Create => {
                sub_cmd.description = Some(format!("Create {}", config_opt.id));
            }
            OperationType::Update => {
                sub_cmd.description = Some(format!("Update {}", config_opt.id));
            }
            OperationType::Delete => {
                sub_cmd.description = Some(format!("Delete {}", config_opt.id));
            }
        };
        sub_cmd.guild_only = true;
        sub_cmd.subcommand_required = false;
        sub_cmd.category = Some(module_id.to_string());
        sub_cmd.custom_data = Box::new(SubcommandCallbackWrapper {
            config_option: config_opt.clone(),
            operation_type: *operation_type,
        }); // Store the config_opt in the command

        sub_cmd.prefix_action = Some(|p_ctx| {
            let ctx = poise::Context::Prefix(p_ctx);

            subcommand_command(ctx).boxed()
        });

        sub_cmd.slash_action = Some(|app_ctx| {
            let ctx = poise::Context::Application(app_ctx);

            subcommand_command(ctx).boxed()
        });

        // Add to command list
        sub_cmds.push(sub_cmd);
    }

    sub_cmds
}

/// Get the choices from the column_type. Note that only string scalar columns can have choices
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

/// Given a column and a CreateCommandOption, set the kind of the CreateCommandOption based on the column type
fn set_create_command_option_from_column_type<'a>(
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

    if operation_type == OperationType::View {
        return args; // View doesnt need any arguments
    }

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
        let mut new_command_param = poise::CommandParameter {
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

                    let cco = set_create_command_option_from_column_type(column.value(), cco);
                    cco.name_localized(INJECT_LOCALE, column.id.to_string())
                })
            },
            autocomplete_callback: None,
            __non_exhaustive: (),
        };

        // Autocomplete for bitflag and other basic input types
        match column.column_type {
            ColumnType::Scalar { ref column_type } => {
                match column_type {
                    InnerColumnType::BitFlag { .. } => {
                        new_command_param.autocomplete_callback = Some(|ctx, partial| {
                            let cwctx = ctx
                                .command()
                                .custom_data
                                .downcast_ref::<SubcommandCallbackWrapper>()
                                .unwrap();

                            // Get column ID from interaction
                            let aco = ctx.interaction.data.autocomplete().unwrap();
                            let column_id = aco.name;

                            async move {
                                // Get column from cwtx
                                let Some(column) = cwctx
                                    .config_option
                                    .columns
                                    .iter()
                                    .find(|c| c.id == column_id)
                                else {
                                    return Err(
                                        poise::SlashArgError::new_command_structure_mismatch(
                                            "Column not found",
                                        ),
                                    );
                                };

                                // Get the values from the column
                                let values = match column.column_type {
                                    ColumnType::Scalar { ref column_type } => match column_type {
                                        InnerColumnType::BitFlag { ref values } => values,
                                        _ => unreachable!(),
                                    },
                                    _ => unreachable!(),
                                };

                                let resp = silverpelt::settings_poise::bitflag_autocomplete(
                                    poise::Context::Application(ctx),
                                    values,
                                    partial,
                                )
                                .await;

                                Ok(serenity::all::CreateAutocompleteResponse::new()
                                    .set_choices(resp))
                            }
                            .boxed()
                        });
                    }
                    _ => {} // No other inner types have autocomplete (yet)
                }
            }
            _ => {} // No other types have autocomplete (yet)
        }

        args.push(new_command_param);
    }

    args
}

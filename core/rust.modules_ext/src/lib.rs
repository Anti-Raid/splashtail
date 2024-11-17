use module_settings::types::OperationType;
use silverpelt::module::{CommandObj, Module};
use silverpelt::types::CommandExtendedData;

/// Base command for a virtual settings command
#[poise::command(slash_command)]
async fn config_opt_base_cmd(_ctx: silverpelt::Context<'_>) -> Result<(), silverpelt::Error> {
    Ok(())
}

fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

pub fn create_full_command_list<T: Module + ?Sized>(module: &T) -> Vec<CommandObj> {
    #[poise::command(slash_command, rename = "")]
    pub async fn base_cmd(_ctx: silverpelt::Context<'_>) -> Result<(), silverpelt::Error> {
        Ok(())
    }

    let mut commands = module.raw_commands();

    // Add in the settings related commands as virtual commands to allow configuring permissions while not listing in the bot
    for config_opt in module.config_options() {
        let mut created_cmd = config_opt_base_cmd();
        created_cmd.name = config_opt.id.to_string().into();
        created_cmd.qualified_name = config_opt.id.to_string().into();

        for (operation_type, _) in config_opt.operations.iter() {
            let mut subcmd = config_opt_base_cmd();
            subcmd.name = operation_type.corresponding_command_suffix().into();
            subcmd.qualified_name = operation_type.corresponding_command_suffix().into();
            subcmd.description = {
                match operation_type {
                    OperationType::View => Some(format!("View {}", config_opt.id).into()),
                    OperationType::Create => Some(format!("Create {}", config_opt.id).into()),
                    OperationType::Update => Some(format!("Update {}", config_opt.id).into()),
                    OperationType::Delete => Some(format!("Delete {}", config_opt.id).into()),
                }
            };
            created_cmd.subcommands.push(subcmd);
        }

        let mut extended_data = indexmap::IndexMap::new();

        // Add base command to extended data
        let mut command_extended_data =
            CommandExtendedData::kittycat_or_admin(module.id(), config_opt.id);

        command_extended_data.virtual_command = true; // Ensure its virtual

        extended_data.insert("", command_extended_data);

        for sub in created_cmd.subcommands.iter() {
            let mut command_extended_data =
                CommandExtendedData::kittycat_or_admin(module.id(), config_opt.id);

            command_extended_data.virtual_command = true; // Ensure its virtual

            extended_data.insert(
                string_to_static_str(sub.name.to_string()),
                command_extended_data,
            );
        }

        commands.push((created_cmd, extended_data));
    }

    commands
}

mod settings;

#[poise::command(prefix_command)]
pub async fn sudo_register(ctx: silverpelt::Context<'_>) -> Result<(), silverpelt::Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

pub struct Module;

impl silverpelt::module::Module for Module {
    fn id(&self) -> &'static str {
        "root"
    }

    fn name(&self) -> &'static str {
        "Root/Staff-Only Commands"
    }

    fn description(&self) -> &'static str {
        "Commands that are only available to staff members. Publicly viewable for transparency."
    }

    fn toggleable(&self) -> bool {
        false
    }

    fn commands_toggleable(&self) -> bool {
        false
    }

    fn is_default_enabled(&self) -> bool {
        true
    }

    fn root_module(&self) -> bool {
        true
    }

    fn raw_commands(&self) -> Vec<silverpelt::module::CommandObj> {
        vec![(sudo_register(), indexmap::indexmap! {})]
    }

    fn config_options(&self) -> Vec<module_settings::types::ConfigOption> {
        vec![
            (*settings::INSPECTOR_FAKE_BOTS).clone(),
            (*settings::LAST_TASK_EXPIRY).clone(),
        ]
    }
}

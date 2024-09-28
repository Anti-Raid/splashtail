pub mod tasks;

use futures_util::future::FutureExt;

pub struct Module;

impl silverpelt::module::Module for Module {
    fn id(&self) -> &'static str {
        "temporary_punishments"
    }

    fn name(&self) -> &'static str {
        "Temporary Punishments"
    }

    fn description(&self) -> &'static str {
        "Customizable setting and management of temporary punishments (tempbans/temp role removals). Most servers will not need to customize this"
    }

    fn is_default_enabled(&self) -> bool {
        true
    }

    fn raw_commands(&self) -> Vec<silverpelt::module::CommandObj> {
        vec![]
    }

    fn background_tasks(&self) -> Vec<silverpelt::BackgroundTask> {
        vec![(
            botox::taskman::Task {
                name: "Temporary Punishment Task",
                description: "Handle expired punishments",
                duration: std::time::Duration::from_secs(60),
                enabled: true,
                run: Box::new(move |ctx| tasks::temporary_punishment_task(ctx).boxed()),
            },
            |_ctx| (true, "Temporary Punishment Task is enabled".to_string()),
        )]
    }
}

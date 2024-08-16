pub mod tasks;

use futures_util::future::FutureExt;

pub fn module() -> silverpelt::Module {
    silverpelt::Module {
        id: "temporary_punishments",
        name: "Temporary Punishments",
        description: "Customizable setting and management of temporary punishments (tempbans/temp role removals). Most servers will not need to customize this",
        toggleable: true,
        commands_toggleable: true,
        virtual_module: false,
        web_hidden: false,
        is_default_enabled: true,
        on_startup: vec![],
        background_tasks: vec![(
            botox::taskman::Task {
                name: "Temporary Punishment Task",
                description: "Handle expired punishments",
                duration: std::time::Duration::from_secs(60),
                enabled: true,
                run: Box::new(move |ctx| tasks::temporary_punishment_task(ctx).boxed()),
            },
            |_ctx| (true, "Temporary Punishment Task is enabled".to_string()),
        )],
        ..Default::default()
    }
}

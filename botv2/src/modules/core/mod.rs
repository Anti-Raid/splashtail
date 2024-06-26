mod help;
mod ping;
mod sandwich_status_task;
mod stats;

use futures_util::FutureExt;

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "core",
        name: "Core Commands",
        description: "Core commands for the bot",
        toggleable: false,
        commands_toggleable: false,
        virtual_module: false,
        web_hidden: false,
        is_default_enabled: true,
        commands: vec![
            (help::help(), crate::silverpelt::CommandExtendedData::none()),
            (
                help::simplehelp(),
                crate::silverpelt::CommandExtendedData::none(),
            ),
            (
                stats::stats(),
                crate::silverpelt::CommandExtendedData::none(),
            ),
            (ping::ping(), crate::silverpelt::CommandExtendedData::none()),
        ],
        background_tasks: vec![botox::taskman::Task {
            name: "Sandwich Status Task",
            description: "Checks the status of the sandwich http server",
            duration: std::time::Duration::from_secs(30),
            enabled: crate::config::CONFIG.meta.sandwich_http_api.is_some(),
            run: Box::new(move |ctx| sandwich_status_task::sandwich_status_task(ctx).boxed()),
        }],
        ..Default::default()
    }
}

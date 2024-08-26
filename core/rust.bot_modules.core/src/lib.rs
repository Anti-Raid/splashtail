mod help;
mod ping;
mod sandwich_status_task;
mod stats;
mod whois;

use futures_util::future::FutureExt;

pub fn module() -> silverpelt::Module {
    silverpelt::Module {
        id: "core",
        name: "Core Commands",
        description: "Core commands for the bot",
        toggleable: false,
        commands_toggleable: false,
        virtual_module: false,
        web_hidden: false,
        is_default_enabled: true,
        commands: vec![
            (
                help::help(),
                silverpelt::types::CommandExtendedData::none_map(),
            ),
            (
                help::simplehelp(),
                silverpelt::types::CommandExtendedData::none_map(),
            ),
            (
                stats::stats(),
                silverpelt::types::CommandExtendedData::none_map(),
            ),
            (
                ping::ping(),
                silverpelt::types::CommandExtendedData::none_map(),
            ),
            (
                whois::whois(),
                silverpelt::types::CommandExtendedData::none_map(),
            )
        ],
        background_tasks: vec![(
            botox::taskman::Task {
                name: "Sandwich Status Task",
                description: "Checks the status of the sandwich http server",
                duration: std::time::Duration::from_secs(30),
                enabled: config::CONFIG.meta.sandwich_http_api.is_some(),
                run: Box::new(move |ctx| sandwich_status_task::sandwich_status_task(ctx).boxed()),
            },
            |_ctx| {
                (
                    config::CONFIG.meta.sandwich_http_api.is_some(),
                    "Sandwich HTTP API is enabled".to_string(),
                )
            },
        )],
        ..Default::default()
    }
}

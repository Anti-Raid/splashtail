mod help;
mod ping;
mod sandwich_status_task;
mod stats;
mod whois;

use futures_util::future::FutureExt;

pub struct Module;

#[async_trait::async_trait]
impl silverpelt::module::Module for Module {
    fn id(&self) -> &'static str {
        "core"
    }

    fn name(&self) -> &'static str {
        "Core Commands"
    }

    fn description(&self) -> &'static str {
        "Core commands for the bot"
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

    fn raw_commands(&self) -> Vec<silverpelt::module::CommandObj> {
        vec![
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
            ),
        ]
    }

    fn background_tasks(&self) -> Vec<silverpelt::BackgroundTask> {
        vec![(
            botox::taskman::Task {
                name: "Sandwich Status Task",
                description: "Checks the status of the sandwich http server",
                duration: std::time::Duration::from_secs(30),
                enabled: true,
                run: Box::new(move |ctx| sandwich_status_task::sandwich_status_task(ctx).boxed()),
            },
            |_ctx| (true, "Sandwich HTTP API is enabled".to_string()),
        )]
    }
}

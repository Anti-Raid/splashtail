mod commands;
mod events;
mod help;
mod modules;
mod ping;
mod settings;
mod stats;
mod tasks;
mod web;
mod whois;

use futures_util::future::FutureExt;
use indexmap::indexmap;

pub struct Module;

#[async_trait::async_trait]
impl silverpelt::module::Module for Module {
    fn id(&self) -> &'static str {
        "core"
    }

    fn name(&self) -> &'static str {
        "Core"
    }

    fn description(&self) -> &'static str {
        "Core module handling pretty much all core functionality"
    }

    fn toggleable(&self) -> bool {
        false
    }

    fn commands_toggleable(&self) -> bool {
        true
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
            (
                modules::modules(),
                indexmap! {
                    "" => silverpelt::types::CommandExtendedData::kittycat_or_admin("modules", "*"),
                    "list" => silverpelt::types::CommandExtendedData::kittycat_or_admin("modules", "list"),
                    "enable" => silverpelt::types::CommandExtendedData::kittycat_or_admin("modules", "enable"),
                    "disable" => silverpelt::types::CommandExtendedData::kittycat_or_admin("modules", "disable"),
                },
            ),
            (
                commands::commands(),
                indexmap! {
                    "check" => silverpelt::types::CommandExtendedData::kittycat_or_admin("commands", "check"),
                    "enable" => silverpelt::types::CommandExtendedData::kittycat_or_admin("commands", "enable"),
                    "disable" => silverpelt::types::CommandExtendedData::kittycat_or_admin("commands", "disable"),
                    "modperms" => silverpelt::types::CommandExtendedData::kittycat_or_admin("commands", "modperms"),
                },
            ),
            (
                web::web(),
                indexmap! {
                    "use" => silverpelt::types::CommandExtendedData {
                        virtual_command: true,
                        ..silverpelt::types::CommandExtendedData::kittycat_or_admin("web", "use")
                    },
                },
            ),
        ]
    }

    fn background_tasks(&self) -> Vec<silverpelt::BackgroundTask> {
        vec![
            (
                botox::taskman::Task {
                    name: "Sandwich Status Task",
                    description: "Checks the status of the sandwich http server",
                    duration: std::time::Duration::from_secs(30),
                    enabled: true,
                    run: Box::new(move |ctx| tasks::sandwich_status_task(ctx).boxed()),
                },
                |_ctx| (true, "Sandwich HTTP API is enabled".to_string()),
            ),
            (
                botox::taskman::Task {
                    name: "Punishment Expiry Task",
                    description: "Check for and dispatch events for expired punishments",
                    duration: std::time::Duration::from_secs(30),
                    enabled: true,
                    run: Box::new(move |ctx| tasks::punishment_expiry_task(ctx).boxed()),
                },
                |_ctx| (true, "Punishment Expiry Task is enabled".to_string()),
            ),
            (
                botox::taskman::Task {
                    name: "Stings Expiry Task",
                    description: "Check for and dispatch events for expired stings",
                    duration: std::time::Duration::from_secs(20),
                    enabled: true,
                    run: Box::new(move |ctx| tasks::stings_expiry_task(ctx).boxed()),
                },
                |_ctx| (true, "Stings Expiry Task is enabled".to_string()),
            ),
        ]
    }

    fn config_options(&self) -> Vec<ar_settings::types::Setting> {
        vec![
            (*settings::GUILD_ROLES).clone(),
            (*settings::GUILD_MEMBERS).clone(),
            (*settings::GUILD_TEMPLATES).clone(),
            (*settings::GUILD_TEMPLATES_KV).clone(),
            (*settings::GUILD_TEMPLATE_SHOP).clone(),
            (*settings::GUILD_TEMPLATE_SHOP_PUBLIC_LIST).clone(),
        ]
    }

    fn event_listeners(&self) -> Option<Box<dyn silverpelt::module::ModuleEventListeners>> {
        Some(Box::new(EventHandler))
    }
}

struct EventHandler;

#[async_trait::async_trait]
impl silverpelt::module::ModuleEventListeners for EventHandler {
    async fn event_handler(
        &self,
        ectx: &silverpelt::ar_event::EventHandlerContext,
    ) -> Result<(), silverpelt::Error> {
        events::event_listener(ectx).await
    }

    fn event_handler_filter(&self, _event: &silverpelt::ar_event::AntiraidEvent) -> bool {
        true // All events should be sent
    }
}

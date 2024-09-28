mod cmds;
mod events;
mod settings;
mod tasks;

use futures_util::future::FutureExt;

pub struct Module;

impl silverpelt::module::Module for Module {
    fn id(&self) -> &'static str {
        "afk"
    }

    fn name(&self) -> &'static str {
        "AFK"
    }

    fn description(&self) -> &'static str {
        "Exactly what it says. Away from keyboard related commands"
    }

    fn raw_commands(&self) -> Vec<silverpelt::module::CommandObj> {
        vec![(
            cmds::afk(),
            indexmap::indexmap! {
                "list" => silverpelt::types::CommandExtendedData::none(),
                "create" => silverpelt::types::CommandExtendedData::none(),
                "update" => silverpelt::types::CommandExtendedData::none(),
                "delete" => silverpelt::types::CommandExtendedData::none(),
            },
        )]
    }

    fn background_tasks(&self) -> Vec<silverpelt::BackgroundTask> {
        vec![(
            botox::taskman::Task {
                name: "AFK Expiration Task",
                description: "Handle expired AFKs",
                duration: std::time::Duration::from_secs(300),
                enabled: true,
                run: Box::new(move |ctx| tasks::afk_task(ctx).boxed()),
            },
            |ctx| {
                (
                    ctx.shard_id == serenity::all::ShardId(0),
                    "AFK expiration only runs on shard 0".to_string(),
                )
            },
        )]
    }

    fn event_listeners(&self) -> Option<Box<dyn silverpelt::module::ModuleEventListeners>> {
        Some(Box::new(EventHandler))
    }

    fn config_options(&self) -> Vec<module_settings::types::ConfigOption> {
        vec![(*settings::AFKS).clone()]
    }
}

struct EventHandler;

#[async_trait::async_trait]
impl silverpelt::module::ModuleEventListeners for EventHandler {
    async fn event_handler(
        &self,
        ectx: &silverpelt::EventHandlerContext,
    ) -> Result<(), silverpelt::Error> {
        events::event_listener(ectx).await
    }
}

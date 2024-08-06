mod cmds;
mod events;
mod settings;
mod tasks;

use futures_util::future::FutureExt;

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "afk",
        name: "AFK",
        description: "Exactly what it says. Away from keyboard related commands",
        toggleable: true,
        commands_toggleable: true,
        virtual_module: false,
        web_hidden: false,
        is_default_enabled: false,
        commands: vec![(
            cmds::afk(),
            indexmap::indexmap! {
                "list" => crate::silverpelt::CommandExtendedData::none(),
                "create" => crate::silverpelt::CommandExtendedData::none(),
                "update" => crate::silverpelt::CommandExtendedData::none(),
                "delete" => crate::silverpelt::CommandExtendedData::none(),
            },
        )],
        on_startup: vec![],
        background_tasks: vec![(
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
        )],
        event_handlers: vec![Box::new(move |ectx| events::event_listener(ectx).boxed())],
        config_options: vec![(*settings::AFKS).clone()],
        ..Default::default()
    }
}

mod help;
mod ping;
mod stats;
mod update_status_task;

use futures_util::FutureExt;

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "core",
        name: "Core Commands",
        description: "Core commands for the bot",
        toggleable: false,
        commands_configurable: false,
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
        background_tasks: vec![
            crate::silverpelt::taskcat::Task {
                name: "Update Bot Status".to_string(),
                description: "Updates the bot status every 15 minutes".to_string(),
                duration: std::time::Duration::from_secs(60 * 5),
                enabled: true,
                run: Box::new(
                    move |_pool, _ch, ctx| {
                        update_status_task::update_status(ctx).boxed()
                    }
                )

            }
        ],
        ..Default::default()
    }
}

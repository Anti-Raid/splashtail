mod help;
mod ping;
mod stats;

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "core",
        name: "Core Commands",
        description: "Core commands for the bot",
        toggleable: false,
        commands_configurable: false,
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
        event_handlers: vec![], // Core has no event listeners
    }
}

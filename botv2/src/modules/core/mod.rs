mod help;
mod stats;
mod ping;

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "core",
        name: "Core Commands",
        commands: vec![
            (help::help(), crate::silverpelt::CommandExtendedData::none()),
            (help::simplehelp(), crate::silverpelt::CommandExtendedData::none()),
            (stats::stats(), crate::silverpelt::CommandExtendedData::none()),
            (ping::ping(), crate::silverpelt::CommandExtendedData::none()),
        ],
        event_handlers: vec![], // Core has no event listeners
    }
}
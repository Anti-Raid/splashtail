mod cmds;

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "root",
        name: "Root/Staff-Only Commands",
        description: "Commands that are only available to staff members.",
        configurable: false,
        commands_configurable: false,
        web_hidden: true,
        is_default_enabled: true,
        // These commands do not follow the typical permission system anyways
        commands: vec![
            (cmds::register(), crate::silverpelt::CommandExtendedData::none()),
        ],
        event_handlers: vec![], // Root has no event listeners
    }
}

mod cmds;

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "root",
        name: "Root/Staff-Only Commands",
        // These commands do not follow the typical permission system anyways
        commands: vec![
            (cmds::register(), crate::silverpelt::CommandExtendedData::none()),
        ],
        event_handlers: vec![], // Root has no event listeners
    }
}

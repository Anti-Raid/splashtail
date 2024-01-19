mod cmds;

pub fn module() -> super::Module {
    super::Module {
        id: "root",
        name: "Root/Staff-Only Commands",
        // These commands do not follow the typical permission system anyways
        commands: vec![
            (cmds::register(), super::CommandExtendedData::none()),
        ],
        event_handlers: vec![], // Root has no event listeners
    }
}

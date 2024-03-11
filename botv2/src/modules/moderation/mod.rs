mod cmd;

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "moderation",
        name: "Moderation",
        description: "Basic customizable moderation plugin for your server.",
        configurable: true,
        commands_configurable: true,
        web_hidden: false,
        is_default_enabled: true,
        commands: vec![],
        event_handlers: vec![],
    }
}

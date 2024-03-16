pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "server_member_backups",
        name: "Server Member Backups",
        description: "Backups members on your server to allow for them to be restored in the event of a raid, nuke or other mass member deletions.",
        toggleable: true,
        commands_configurable: true,
        virtual_module: false,
        web_hidden: false,
        is_default_enabled: false,
        commands: vec![],
        event_handlers: vec![],
    }
}

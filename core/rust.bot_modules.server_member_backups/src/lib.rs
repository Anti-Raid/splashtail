pub fn module() -> silverpelt::Module {
    silverpelt::Module {
        id: "server_member_backups",
        name: "Server Member Backups",
        description: "Backups members on your server to allow for them to be restored in the event of a raid, nuke or other mass member deletions.",
        toggleable: true,
        commands_toggleable: true,
        virtual_module: false,
        web_hidden: true, // Not yet ready for release yet
        is_default_enabled: false,
        commands: vec![],
        ..Default::default()
    }
}

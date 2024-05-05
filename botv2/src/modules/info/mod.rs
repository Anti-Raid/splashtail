use futures_util::FutureExt;

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "info",
        name: "Info Commands",
        description: "Get information about the server/guilds etc. Useful for diagnostics",
        toggleable: false,
        commands_configurable: false,
        virtual_module: false,
        web_hidden: false,
        is_default_enabled: true,
        commands: vec![],
        ..Default::default()
    }
}

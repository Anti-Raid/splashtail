pub mod whois;

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "info",
        name: "Info Commands",
        description: "Get information about the server/guilds etc. Useful for diagnostics",
        toggleable: true,
        commands_toggleable: true,
        virtual_module: false,
        web_hidden: false,
        is_default_enabled: true,
        commands: vec![(
            whois::whois(),
            crate::silverpelt::CommandExtendedData::none_map(),
        )],
        ..Default::default()
    }
}

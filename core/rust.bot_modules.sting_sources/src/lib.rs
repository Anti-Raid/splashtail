pub mod settings;

pub fn module() -> silverpelt::Module {
    silverpelt::Module {
        id: "sting_sources",
        name: "Sting Sources",
        description: "Customizable management of sting sources",
        toggleable: true,
        commands_toggleable: true,
        virtual_module: false,
        web_hidden: true, // Not yet ready for release yet
        is_default_enabled: true,
        commands: vec![],
        on_startup: vec![],
        ..Default::default()
    }
}

pub mod sting_source;
pub mod core;

use futures::future::FutureExt;

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "punishments",
        name: "Punishments",
        description: "Customizable setting of punishments based on number of stings.",
        toggleable: true,
        commands_configurable: true,
        virtual_module: false,
        web_hidden: false,
        is_default_enabled: false,
        commands: vec![],
        on_startup: vec![],
        ..Default::default()
    }
}

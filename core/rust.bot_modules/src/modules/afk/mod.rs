mod cmds;
mod events;
mod settings;

use futures_util::future::FutureExt;

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "afk",
        name: "AFK",
        description: "Exactly what it says. Away from keyboard related commands",
        toggleable: true,
        commands_toggleable: true,
        virtual_module: false,
        web_hidden: false,
        is_default_enabled: false,
        commands: vec![(
            cmds::afk(),
            indexmap::indexmap! {
                "list" => crate::silverpelt::CommandExtendedData::none(),
                "create" => crate::silverpelt::CommandExtendedData::none(),
                "update" => crate::silverpelt::CommandExtendedData::none(),
                "delete" => crate::silverpelt::CommandExtendedData::none(),
            },
        )],
        on_startup: vec![],
        event_handlers: vec![Box::new(move |ectx| events::event_listener(ectx).boxed())],
        config_options: vec![(*settings::AFKS).clone()],
        ..Default::default()
    }
}

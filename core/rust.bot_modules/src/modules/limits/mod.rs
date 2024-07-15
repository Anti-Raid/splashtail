mod autocompletes;
mod cmds;
mod core;
mod events;
mod handler;
mod settings;

use futures_util::FutureExt;
use indexmap::indexmap;

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "limits",
        name: "Limits",
        description:
            "Experimental server ratelimiting module. Not yet suitable for production use. Should be combined with anti-nuke bots for best efficacy",
        toggleable: true,
        commands_toggleable: true,
        virtual_module: false,
        web_hidden: false,
        is_default_enabled: false,
        commands: vec![
            (
                cmds::limits(),
                indexmap! {
                    "view" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("limits", "view"),
                    "add" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("limits", "add"),
                    "update" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("limits", "update"),
                    "remove" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("limits", "remove"),
                    "hit" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("limits", "hit"),
                },
            ),
            (
                cmds::limitactions(),
                indexmap! {
                    "view" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("limits", "limitactions_view"),
                }
            ),
        ],
        event_handlers: vec![Box::new(move |ectx| {
            events::event_listener(ectx).boxed()
        })],
        on_startup: vec![Box::new(move |data| {
            core::register_punishment_sting_source(data).boxed()
        })],
        config_options: vec![(*settings::GUILD_LIMITS).clone()],
        ..Default::default()
    }
}

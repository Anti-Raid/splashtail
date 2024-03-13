mod autocompletes;
mod cmds;
mod core;
mod events;
mod handler;

use indexmap::indexmap;

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "limits",
        name: "Limits",
        description:
            "Experimental server ratelimiting module. Not yet suitable for production use. Should be combined with anti-nuke bots for best efficacy",
        configurable: true,
        commands_configurable: true,
        web_hidden: false,
        is_default_enabled: false,
        commands: vec![
            (
                cmds::limits(),
                indexmap! {
                    "add" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("limits", "add"),
                    "view" => crate::silverpelt::CommandExtendedData::kittycat_or_admin("limits", "view"),
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
        event_handlers: vec![Box::new(move |ctx, fe, ectx| {
            Box::pin(async move { events::event_listener(ctx, fe, ectx).await })
        })],
    }
}

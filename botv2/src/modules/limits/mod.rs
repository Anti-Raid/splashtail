mod core;
mod handler;
mod events;
mod cmds;
mod autocompletes;

use indexmap::indexmap;

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "limits",
        name: "Limits",
        description: "Experimental server ratelimiting module. Not yet suitable for production use.",
        configurable: true,
        commands_configurable: true,
        web_hidden: false,
        is_default_enabled: false,
        commands: vec![
            (cmds::limits(), indexmap! {
                "add" => crate::silverpelt::CommandExtendedData::kittycat_simple("limits", "add"),
                "view" => crate::silverpelt::CommandExtendedData::kittycat_simple("limits", "view"),
                "remove" => crate::silverpelt::CommandExtendedData::kittycat_simple("limits", "remove"),
                "hit" => crate::silverpelt::CommandExtendedData::kittycat_simple("limits", "hit"),
            }),
            (cmds::limitactions(), crate::silverpelt::CommandExtendedData::none()),
        ],
        event_handlers: vec![
            Box::new(
                move |ctx, fe| {
                    Box::pin(async move {
                        events::event_listener(ctx, fe).await
                    })
                }
            )
        ]
    }
}
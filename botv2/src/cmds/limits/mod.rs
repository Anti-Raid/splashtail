mod cmds;
mod autocompletes;

use indexmap::indexmap;

pub fn module() -> super::Module {
    super::Module {
        id: "limits",
        name: "Limits",
        commands: vec![
            (cmds::limits(), indexmap! {
                "add" => super::CommandExtendedData::kittycat_simple("limits", "add"),
                "view" => super::CommandExtendedData::kittycat_simple("limits", "view"),
                "remove" => super::CommandExtendedData::kittycat_simple("limits", "remove"),
                "hit" => super::CommandExtendedData::kittycat_simple("limits", "hit"),
            }),
            (cmds::limitactions(), super::CommandExtendedData::none()),
        ],
        event_handlers: vec![
            Box::new(
                move |ctx, fe| {
                    Box::pin(async move {
                        crate::modules::limits::events::event_listener(ctx, fe).await
                    })
                }
            )
        ]
    }
}
mod autocompletes;
mod cmds;
mod core;
mod events;
mod handler;
mod settings;

use futures_util::future::FutureExt;
use indexmap::indexmap;

pub fn module() -> silverpelt::Module {
    silverpelt::Module {
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
                    "view" => silverpelt::types::CommandExtendedData::kittycat_or_admin("limits", "view"),
                    "add" => silverpelt::types::CommandExtendedData::kittycat_or_admin("limits", "add"),
                    "update" => silverpelt::types::CommandExtendedData::kittycat_or_admin("limits", "update"),
                    "remove" => silverpelt::types::CommandExtendedData::kittycat_or_admin("limits", "remove"),
                },
            ),
            (
                cmds::past_hit_limits(),
                indexmap! {
                    "view" => silverpelt::types::CommandExtendedData::kittycat_or_admin("past_hit_limits", "view"),
                    "remove" => silverpelt::types::CommandExtendedData::kittycat_or_admin("past_hit_limits", "remove"),
                }
            ),
            (
                cmds::limit_user_actions(),
                indexmap! {
                    "view" => silverpelt::types::CommandExtendedData::kittycat_or_admin("limit_user_actions", "view"),
                    "remove" => silverpelt::types::CommandExtendedData::kittycat_or_admin("limit_user_actions", "remove"),
                }
            )
        ],
        event_handlers: vec![Box::new(move |ectx| {
            events::event_listener(ectx).boxed()
        })],
        sting_sources: vec![
            std::sync::Arc::new(core::LimitsUserActionsStingSource)
        ],
        config_options: vec![
            (*settings::PAST_HIT_LIMITS).clone(),
            (*settings::USER_ACTIONS).clone(),
            (*settings::GUILD_LIMITS).clone(),
        ],
        ..Default::default()
    }
}

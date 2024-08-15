pub mod cache; // Used by root module
mod cmd;
mod core;
mod dehoist;
pub mod events; // Events is a public interface
mod guildprotect;
mod settings;
pub mod types;

use futures_util::future::FutureExt;

pub fn module() -> silverpelt::Module {
    silverpelt::Module {
        id: "inspector",
        name: "Inspector",
        description:
            "Provides basic anti-spam options (currently only anti-invite and anti-everyone pings)",
        toggleable: true,
        commands_toggleable: true,
        virtual_module: false,
        web_hidden: false,
        is_default_enabled: false,
        commands: vec![(
            cmd::inspector(),
            indexmap::indexmap! {
                "list" => silverpelt::types::CommandExtendedData::kittycat_or_admin("inspector", "list"),
                "setup" => silverpelt::types::CommandExtendedData::kittycat_or_admin("inspector", "setup"),
                "update" => silverpelt::types::CommandExtendedData::kittycat_or_admin("inspector", "setup"),
                "disable" => silverpelt::types::CommandExtendedData::kittycat_or_admin("inspector", "setup"),
            },
        )],
        event_handlers: vec![Box::new(move |ectx| events::event_listener(ectx).boxed())],
        on_startup: vec![
            Box::new(move |data| cache::setup_cache_initial(&data.pool).boxed()),
            Box::new(move |data| cache::setup_am_toggle(data).boxed()),
            Box::new(move |data| cache::setup_fake_bots_cache(&data.pool).boxed()),
            Box::new(move |data| core::register_punishment_sting_source(data).boxed()),
        ],
        s3_paths: vec!["inspector/guild_icons/{guild_id}".to_string()],
        config_options: vec![(*settings::INSPECTOR_OPTIONS).clone()],
        ..Default::default()
    }
}

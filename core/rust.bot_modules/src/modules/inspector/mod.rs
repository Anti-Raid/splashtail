mod cache;
mod core;
mod dehoist;
pub mod events; // Events is a public interface
mod guildprotect;
pub mod types;

use futures::future::FutureExt;

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "inspector",
        name: "Inspector",
        description:
            "Provides basic anti-spam options (currently only anti-invite and anti-everyone pings)",
        toggleable: true,
        commands_toggleable: true,
        virtual_module: false,
        web_hidden: false,
        is_default_enabled: false,
        commands: vec![],
        event_handlers: vec![Box::new(move |ectx| events::event_listener(ectx).boxed())],
        on_startup: vec![
            Box::new(move |data| cache::setup_cache_initial(&data.pool).boxed()),
            Box::new(move |data| cache::setup_am_toggle(data).boxed()),
            Box::new(move |data| cache::setup_fake_bots(data).boxed()),
            Box::new(move |data| core::register_punishment_sting_source(data).boxed()),
        ],
        on_first_ready: vec![Box::new(move |ctx, data| {
            guildprotect::save_all_guilds_initial(ctx, data).boxed()
        })],
        s3_paths: vec!["inspector/guild_icons/{guild_id}".to_string()],
        ..Default::default()
    }
}

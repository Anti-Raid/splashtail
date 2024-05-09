mod cache;
mod core;
pub mod events; // Events is a public interface

use futures::future::FutureExt;

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "inspector",
        name: "Inspector",
        description:
            "Provides basic anti-spam options (currently only anti-invite and anti-everyone pings)",
        toggleable: true,
        commands_configurable: true,
        virtual_module: false,
        web_hidden: false,
        is_default_enabled: false,
        commands: vec![],
        event_handlers: vec![Box::new(move |ectx| events::event_listener(ectx).boxed())],
        on_startup: vec![
            Box::new(move |data| cache::setup_cache_initial(&data.pool).boxed()),
            Box::new(move |data| cache::setup_am_toggle(&data.pool).boxed()),
            Box::new(move |data| cache::setup_fake_bots(data).boxed()),
            Box::new(move |data| core::register_punishment_sting_source(data).boxed()),
        ],
        ..Default::default()
    }
}
pub mod events; // Events is a public interface
mod cache;

use futures::future::FutureExt;

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "basic_antispam",
        name: "Basic Antispam",
        description: "Provides basic anti-spam options (currently only anti-invite and anti-everyone pings)",
        toggleable: true,
        commands_configurable: true,
        virtual_module: false,
        web_hidden: false,
        is_default_enabled: false,
        commands: vec![],
        event_handlers: vec![Box::new(move |ctx, fe, ectx| {
            events::event_listener(ctx, fe, ectx).boxed()
        })],
        on_startup: vec![
            Box::new(move |data| {
                cache::setup_cache_initial(&data.pool).boxed()
            }),
            Box::new(move |data| {
                cache::setup_am_toggle(&data.pool).boxed()
            }),
        ],
        ..Default::default()
    }
}

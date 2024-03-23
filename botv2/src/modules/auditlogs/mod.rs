mod events;

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "auditlogs",
        name: "Audit Logs",
        description: "Customizable and comprehensive audit logging module supporting 72+ discord events",
        toggleable: false,
        commands_configurable: false,
        virtual_module: false,
        web_hidden: false,
        is_default_enabled: false,
        commands: vec![], // No commands
        event_handlers: vec![Box::new(move |ctx, fe, ectx| {
            Box::pin(async move { events::event_listener(ctx, fe, ectx).await })
        })],
    }
}

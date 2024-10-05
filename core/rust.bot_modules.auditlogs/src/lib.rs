mod am_toggles;
mod cache;
mod checks;
mod cmds;
pub mod events;
mod settings;

use indexmap::indexmap;
use permissions::types::{PermissionCheck, PermissionChecks};
use silverpelt::types::CommandExtendedData;

pub struct Module;

impl silverpelt::module::Module for Module {
    fn id(&self) -> &'static str {
        "auditlogs"
    }

    fn name(&self) -> &'static str {
        "Audit Logs"
    }

    fn description(&self) -> &'static str {
        "Customizable and comprehensive audit logging module supporting 72+ discord events"
    }

    fn raw_commands(&self) -> Vec<silverpelt::module::CommandObj> {
        vec![(
            cmds::auditlogs(),
            indexmap! {
                "list_sinks" => CommandExtendedData {
                    default_perms: PermissionChecks::Simple {
                        checks: vec![
                            PermissionCheck {
                                kittycat_perms: vec!["auditlogs.list_sinks".to_string(), "auditlogs.list".to_string()],
                                native_perms: vec![],
                                inner_and: false,
                                outer_and: false,
                            },
                            PermissionCheck {
                                kittycat_perms: vec![],
                                native_perms: vec![serenity::model::permissions::Permissions::VIEW_AUDIT_LOG, serenity::model::permissions::Permissions::MANAGE_GUILD],
                                inner_and: true,
                                outer_and: false,
                            }
                        ],
                    },
                    ..Default::default()
                },
                "add_channel" => CommandExtendedData {
                    default_perms: PermissionChecks::Simple {
                        checks: vec![
                            PermissionCheck {
                                kittycat_perms: vec!["auditlogs.add_sink".to_string()],
                                native_perms: vec![],
                                inner_and: false,
                                outer_and: false,
                            },
                            PermissionCheck {
                                kittycat_perms: vec![],
                                native_perms: vec![serenity::model::permissions::Permissions::VIEW_AUDIT_LOG, serenity::model::permissions::Permissions::MANAGE_GUILD],
                                inner_and: true,
                                outer_and: false,
                            }
                        ],
                    },
                    ..Default::default()
                },
                "add_sink" => CommandExtendedData {
                    default_perms: PermissionChecks::Simple {
                        checks: vec![
                            PermissionCheck {
                                kittycat_perms: vec!["auditlogs.add_sink".to_string()],
                                native_perms: vec![],
                                inner_and: false,
                                outer_and: false,
                            },
                            PermissionCheck {
                                kittycat_perms: vec![],
                                native_perms: vec![serenity::model::permissions::Permissions::VIEW_AUDIT_LOG, serenity::model::permissions::Permissions::MANAGE_GUILD],
                                inner_and: true,
                                outer_and: false,
                            }
                        ],
                    },
                    ..Default::default()
                },
                "add_discordhook" => CommandExtendedData {
                    default_perms: PermissionChecks::Simple {
                        checks: vec![
                            PermissionCheck {
                                kittycat_perms: vec!["auditlogs.add_sink".to_string()],
                                native_perms: vec![],
                                inner_and: false,
                                outer_and: false,
                            },
                            PermissionCheck {
                                kittycat_perms: vec![],
                                native_perms: vec![serenity::model::permissions::Permissions::VIEW_AUDIT_LOG, serenity::model::permissions::Permissions::MANAGE_GUILD],
                                inner_and: true,
                                outer_and: false,
                            }
                        ],
                    },
                    ..Default::default()
                },
                "edit_sink" => CommandExtendedData {
                    default_perms: PermissionChecks::Simple {
                        checks: vec![
                            PermissionCheck {
                                kittycat_perms: vec!["auditlogs.edit_sink".to_string()],
                                native_perms: vec![],
                                inner_and: false,
                                outer_and: false,
                            },
                            PermissionCheck {
                                kittycat_perms: vec![],
                                native_perms: vec![serenity::model::permissions::Permissions::VIEW_AUDIT_LOG, serenity::model::permissions::Permissions::MANAGE_GUILD],
                                inner_and: true,
                                outer_and: false,
                            }
                        ],
                    },
                    ..Default::default()
                },
                "remove_sink" => CommandExtendedData {
                    default_perms: PermissionChecks::Simple {
                        checks: vec![
                            PermissionCheck {
                                kittycat_perms: vec!["auditlogs.remove_sink".to_string()],
                                native_perms: vec![],
                                inner_and: false,
                                outer_and: false,
                            },
                            PermissionCheck {
                                kittycat_perms: vec![],
                                native_perms: vec![serenity::model::permissions::Permissions::VIEW_AUDIT_LOG, serenity::model::permissions::Permissions::MANAGE_GUILD],
                                inner_and: true,
                                outer_and: false,
                            }
                        ],
                    },
                    ..Default::default()
                },
            },
        )]
    }

    fn event_listeners(&self) -> Option<Box<dyn silverpelt::module::ModuleEventListeners>> {
        Some(Box::new(EventHandler))
    }

    fn config_options(&self) -> Vec<module_settings::types::ConfigOption> {
        vec![(*settings::SINK).clone()]
    }
}

struct EventHandler;

#[async_trait::async_trait]
impl silverpelt::module::ModuleEventListeners for EventHandler {
    async fn on_startup(&self, data: &silverpelt::data::Data) -> Result<(), silverpelt::Error> {
        am_toggles::setup(data).await
    }

    async fn event_handler(
        &self,
        ectx: &silverpelt::ar_event::EventHandlerContext,
    ) -> Result<(), silverpelt::Error> {
        events::event_listener(ectx).await
    }

    fn event_handler_filter(&self, event: &silverpelt::ar_event::AntiraidEvent) -> bool {
        match event {
            silverpelt::ar_event::AntiraidEvent::Discord(_) => true,
            silverpelt::ar_event::AntiraidEvent::Custom(ref ce) => {
                ce.target() == std_events::auditlog::AUDITLOG_TARGET_ID
            }
            silverpelt::ar_event::AntiraidEvent::StingCreate(_) => true,
            _ => false,
        }
    }
}

mod am_toggles;
mod cmds;
mod core;
pub mod events; // Events is a public interface

use crate::silverpelt::config_opts::{
    Column, ColumnAction, ColumnSuggestion, ColumnType, ConfigOption, OperationSpecific,
    OperationType, OptionType
};
use futures_util::FutureExt;
use indexmap::indexmap;

pub fn module() -> crate::silverpelt::Module {
    crate::silverpelt::Module {
        id: "auditlogs",
        name: "Audit Logs",
        description:
            "Customizable and comprehensive audit logging module supporting 72+ discord events",
        toggleable: true,
        commands_configurable: true,
        virtual_module: false,
        web_hidden: false,
        is_default_enabled: false,
        commands: vec![(
            cmds::auditlogs(),
            indexmap! {
                "list_sinks" => crate::silverpelt::CommandExtendedData {
                    default_perms: crate::silverpelt::PermissionChecks {
                        checks: vec![
                            crate::silverpelt::PermissionCheck {
                                kittycat_perms: vec!["auditlogs.list_sinks".to_string(), "auditlogs.list".to_string()],
                                native_perms: vec![],
                                inner_and: false,
                                outer_and: false,
                            },
                            crate::silverpelt::PermissionCheck {
                                kittycat_perms: vec![],
                                native_perms: vec![serenity::model::permissions::Permissions::VIEW_AUDIT_LOG, serenity::model::permissions::Permissions::MANAGE_GUILD],
                                inner_and: true,
                                outer_and: false,
                            }
                        ],
                        checks_needed: 1,
                    },
                    ..Default::default()
                },
                "add_channel" => crate::silverpelt::CommandExtendedData {
                    default_perms: crate::silverpelt::PermissionChecks {
                        checks: vec![
                            crate::silverpelt::PermissionCheck {
                                kittycat_perms: vec!["auditlogs.add_channel".to_string(), "auditlogs.add".to_string()],
                                native_perms: vec![],
                                inner_and: false,
                                outer_and: false,
                            },
                            crate::silverpelt::PermissionCheck {
                                kittycat_perms: vec![],
                                native_perms: vec![serenity::model::permissions::Permissions::VIEW_AUDIT_LOG, serenity::model::permissions::Permissions::MANAGE_GUILD],
                                inner_and: true,
                                outer_and: false,
                            }
                        ],
                        checks_needed: 1,
                    },
                    ..Default::default()
                },
                "add_discordhook" => crate::silverpelt::CommandExtendedData {
                    default_perms: crate::silverpelt::PermissionChecks {
                        checks: vec![
                            crate::silverpelt::PermissionCheck {
                                kittycat_perms: vec!["auditlogs.add_discordhook".to_string(), "auditlogs.add".to_string()],
                                native_perms: vec![],
                                inner_and: false,
                                outer_and: false,
                            },
                            crate::silverpelt::PermissionCheck {
                                kittycat_perms: vec![],
                                native_perms: vec![serenity::model::permissions::Permissions::VIEW_AUDIT_LOG, serenity::model::permissions::Permissions::MANAGE_GUILD],
                                inner_and: true,
                                outer_and: false,
                            }
                        ],
                        checks_needed: 1,
                    },
                    ..Default::default()
                },
                "remove_sink" => crate::silverpelt::CommandExtendedData {
                    default_perms: crate::silverpelt::PermissionChecks {
                        checks: vec![
                            crate::silverpelt::PermissionCheck {
                                kittycat_perms: vec!["auditlogs.remove_sink".to_string(), "auditlogs.remove".to_string()],
                                native_perms: vec![],
                                inner_and: false,
                                outer_and: false,
                            },
                            crate::silverpelt::PermissionCheck {
                                kittycat_perms: vec![],
                                native_perms: vec![serenity::model::permissions::Permissions::VIEW_AUDIT_LOG, serenity::model::permissions::Permissions::MANAGE_GUILD],
                                inner_and: true,
                                outer_and: false,
                            }
                        ],
                        checks_needed: 1,
                    },
                    ..Default::default()
                },
            },
        )],
        on_startup: vec![Box::new(move |data| am_toggles::setup(data).boxed())],
        event_handlers: vec![Box::new(move |ectx| events::event_listener(ectx).boxed())],
        config_options: vec![
            ConfigOption {
                id: "sinks",
                name: "Audit Log Sinks",
                description: "A sink is a place where audit logs are sent to. This can be a channel or a webhook at this time. More sink types may be added in the future.",
                table: "auditlogs__sinks",
                option_type: OptionType::Multiple,
                guild_id: "guild_id",
                primary_key: "id",
                columns: vec![
                    Column {
                        id: "id",
                        name: "Sink ID",
                        column_type: ColumnType::Uuid {},
                        nullable: false,
                        unique: true,
                        array: false,
                        suggestions: ColumnSuggestion::Dynamic { 
                            table_name: "auditlogs__sinks", 
                            column_name: "id"
                        },
                        readonly: indexmap::indexmap! {},
                        pre_checks: indexmap::indexmap! {
                            OperationType::Create => vec![
                                ColumnAction::CollectColumnToMap { 
                                    table: "auditlogs__sinks", 
                                    column: "id", 
                                    key: "ids", 
                                    fetch_all: true 
                                },
                                ColumnAction::ExecLuaScript { 
                                    script: "return #data.ids < 10",
                                    on_success: vec![],
                                    on_failure: vec![
                                        ColumnAction::Error { 
                                            message: "You have reached the maximum number of sinks allowed. Please remove a sink before adding a new one." 
                                        }
                                    ],
                                },
                                ColumnAction::IpcPerModuleFunction {
                                    module: "auditlogs",
                                    function: "check_all_events",
                                    arguments: indexmap::indexmap! {
                                        "events" => "events"
                                    }
                                }
                            ],
                            OperationType::Update => vec![
                                ColumnAction::IpcPerModuleFunction {
                                    module: "auditlogs",
                                    function: "check_all_events",
                                    arguments: indexmap::indexmap! {
                                        "events" => "events"
                                    }
                                }
                            ]
                        },
                        default_pre_checks: vec![],
                    },
                    Column {
                        id: "type",
                        name: "Sink Type",
                        column_type: ColumnType::String { min_length: None, max_length: None, allowed_values: vec!["channel", "discordhook"] },
                        nullable: false,
                        unique: false,
                        array: false,
                        suggestions: ColumnSuggestion::Static { suggestions: vec!["channel", "discordhook"] },
                        readonly: indexmap::indexmap! {
                            OperationType::Update => true,
                        },
                        pre_checks: indexmap::indexmap! {},
                        default_pre_checks: vec![],
                    },
                    Column {
                        id: "sink",
                        name: "Sink",
                        column_type: ColumnType::String { min_length: None, max_length: None, allowed_values: vec![] },
                        nullable: false,
                        unique: false,
                        array: false,
                        suggestions: ColumnSuggestion::Static { suggestions: vec![] },
                        readonly: indexmap::indexmap! {},
                        pre_checks: indexmap::indexmap! {},
                        default_pre_checks: vec![
                            // If discordhook, must be a webhook
                            ColumnAction::ExecLuaScript {
                                script: r#"
                                    if data.type == "discordhook" then
                                        return data.sink:startswith("https://discord.com/api/webhooks") or
                                            data.sink:startswith("https://discord.com/api/v9/webhooks") or
                                            data.sink:startswith("https://discord.com/api/v10/webhooks")
                                    else
                                        return true -- TODO: Check channels
                                    end
                                "#,
                                on_success: vec![],
                                on_failure: vec![
                                    ColumnAction::Error { message: "Discord webhooks sinks must be a webhook." }
                                ],
                            }
                        ]
                    }
                ],
                operations: indexmap::indexmap! {
                    OperationType::View => OperationSpecific {
                        corresponding_command: "list_sinks",
                        column_ids: vec![],
                        columns_to_set: indexmap::indexmap! {},
                    },
                }
            }
        ],
        ..Default::default()
    }
}

use futures_util::FutureExt;
use module_settings::types::{
    settings_wrap_columns, settings_wrap_precheck, Column, ColumnAction, ColumnSuggestion, ColumnType, ConfigOption, InnerColumnType, InnerColumnTypeStringKind, OperationSpecific, OperationType, SettingsError
};
use once_cell::sync::Lazy;
use splashcore_rs::value::Value;

pub static SINK: Lazy<ConfigOption> = Lazy::new(|| {
    ConfigOption {
        id: "sinks",
        name: "Audit Log Sinks",
        description: "A sink is a place where audit logs are sent to. This can be a channel or a webhook at this time. More sink types may be added in the future.",
        table: "auditlogs__sinks",
        guild_id: "guild_id",
        primary_key: "id",
        max_entries: 10,
        columns: settings_wrap_columns(vec![
            Column {
                id: "id",
                name: "Sink ID",
                description: "The unique identifier for the sink.",
                column_type: ColumnType::new_scalar(InnerColumnType::Uuid {}),
                nullable: false,
                unique: true,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![OperationType::Create],
                secret: None,
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
                default_pre_checks: settings_wrap_precheck(vec![]),
            },
            Column {
                id: "type",
                name: "Sink Type",
                description: "The type of sink. This can be a sink that sends messages to a channel (`channel`) or a discord webhook (`discordhook`).",
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec!["channel", "discordhook"], kind: InnerColumnTypeStringKind::Normal }),
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::Static { suggestions: vec!["channel", "discordhook"] },
                ignored_for: vec![],
                secret: None,
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
                default_pre_checks: settings_wrap_precheck(vec![]),
            },
            Column {
                id: "sink",
                name: "Sink",
                description: "The sink where the logs are sent to. This can be a channel ID (if `channel`) or a discord webhook URL (if `discordhook`).",
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::Normal }),
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: None,
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
                default_pre_checks: settings_wrap_precheck(vec![
                    // Set sink display type
                    ColumnAction::NativeAction {
                        action: Box::new(|_ctx, state| async move {
                            if let Some(Value::String(v)) = state.state.get("type") {
                                if v == "channel" {
                                    state.state.insert("__sink_displaytype".to_string(), Value::String("channel".to_string()));
                                }
                            }
                            Ok(())
                        }.boxed()),
                        on_condition: None
                    },
                    ColumnAction::NativeAction {
                        action: Box::new(|_ctx, state| async move {
                            let Some(Value::String(sink)) = state.state.get("sink") else {
                                return Err(SettingsError::MissingOrInvalidField { 
                                    field: "sink".to_string(),
                                    src: "sink->NativeAction [default_pre_checks]".to_string(),
                                });
                            };
    
                            let Some(Value::String(typ)) = state.state.get("type") else {
                                return Err(SettingsError::MissingOrInvalidField { 
                                    field: "type".to_string(),
                                    src: "sink->NativeAction [default_pre_checks]".to_string(),
                                });
                            };
    
                            if typ == "discordhook" {
                                let sink_url = url::Url::parse(sink)
                                .map_err(|e| SettingsError::SchemaCheckValidationError { 
                                    column: "sink".to_string(),
                                    check: "parse_webhook.parse_sink_to_url".to_string(),
                                    error: e.to_string(),
                                    accepted_range: "Valid Discord webhook URL".to_string()
                                })?;    
    
                                if serenity::utils::parse_webhook(
                                    &sink_url
                                ).is_none() {
                                    return Err(SettingsError::SchemaCheckValidationError { 
                                        column: "sink".to_string(),
                                        check: "parse_webhook.parse".to_string(),
                                        error: "Discord webhook sinks must be a valid webhook URL".to_string(),
                                        accepted_range: "Valid Discord webhook URL".to_string()
                                    });
                                }
                            } else if typ == "channel" {
                                sink.parse::<serenity::all::ChannelId>().map_err(|e| SettingsError::SchemaCheckValidationError {
                                    column: "sink".to_string(),
                                    check: "snowflake_parse".to_string(),
                                    accepted_range: "Valid channel id".to_string(),
                                    error: e.to_string(),
                                })?;
                            } else {
                                return Err(SettingsError::SchemaCheckValidationError { 
                                    column: "type".to_string(),
                                    check: "parse_webhook.parse".to_string(),
                                    error: "Invalid sink type".to_string(),
                                    accepted_range: "Valid Discord webhook URL".to_string()
                                });
                            }
    
                            Ok(())
                        }.boxed()),
                        on_condition: None
                    },
                    // If a channel, execute the check_channel IPC function
                    ColumnAction::IpcPerModuleFunction {
                        module: "auditlogs",
                        function: "check_channel",
                        arguments: indexmap::indexmap! {
                            "channel_id" => "{sink}",
                            "guild_id" => "{__guild_id}"
                        },
                        on_condition: Some(|_acc, state| {
                            let Some(Value::String(typ)) = state.state.get("type") else {
                                return Err(SettingsError::MissingOrInvalidField { 
                                    field: "type".to_string(),
                                    src: "sink->IpcPerModuleFunction [default_pre_checks]".to_string(),
                                });
                            };
    
                            Ok(typ == "channel")
                        })
                    },
                ])
            },
            Column {
                id: "events",
                name: "Events",
                description: "The events that are sent to the sink. If empty, all events are sent. Prefix with R/ for regex (rust style regex) matching.",
                column_type: ColumnType::new_array(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::Normal }),
                nullable: true,
                unique: false,
                suggestions: ColumnSuggestion::Static { suggestions: gwevent::core::event_list().to_vec() },
                ignored_for: vec![],
                secret: None,
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {
                    OperationType::View => vec![]
                }),
                default_pre_checks: settings_wrap_precheck(vec![
                    ColumnAction::IpcPerModuleFunction {
                        module: "auditlogs",
                        function: "check_all_events",
                        arguments: indexmap::indexmap! {
                            "events" => "{events}"
                        },
                        on_condition: None
                    }
                ])
            },
            Column {
                id: "embed_template",
                name: "Template",
                description: "The custom template for the embed. This is a tera template that is executed when an event is sent to the sink. If empty, falls back to default handling",
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::Template }),
                ignored_for: vec![],
                secret: None,
                nullable: true,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
                default_pre_checks: settings_wrap_precheck(vec![])
            },
            module_settings::common_columns::created_at(),
            module_settings::common_columns::created_by(),
            module_settings::common_columns::last_updated_at(),
            module_settings::common_columns::last_updated_by(),
            Column {
                id: "broken",
                name: "Marked as Broken",
                description: "If the sink is marked as broken, it will not be used for sending logs. This can be useful in debugging too!",
                column_type: ColumnType::new_scalar(InnerColumnType::Boolean {}),
                ignored_for: vec![OperationType::Create],
                secret: None,
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
                default_pre_checks: settings_wrap_precheck(vec![])
            },
        ]),
        operations: indexmap::indexmap! {
            OperationType::View => OperationSpecific {
                corresponding_command: "auditlogs list_sinks",
                columns_to_set: indexmap::indexmap! {},
            },
            OperationType::Create => OperationSpecific {
                corresponding_command: "auditlogs add_sink",
                columns_to_set: indexmap::indexmap! {
                    "created_at" => "{__now}",
                    "created_by" => "{__author}",
                    "last_updated_at" => "{__now}",
                    "last_updated_by" => "{__author}",
                },
            },
            OperationType::Update => OperationSpecific {
                corresponding_command: "auditlogs edit_sink",
                columns_to_set: indexmap::indexmap! {
                    "last_updated_at" => "{__now}",
                    "last_updated_by" => "{__author}",
                },
            },
            OperationType::Delete => OperationSpecific {
                corresponding_command: "auditlogs remove_sink",
                columns_to_set: indexmap::indexmap! {},
            },
        }
    }
});
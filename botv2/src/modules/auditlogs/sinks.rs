use futures_util::FutureExt;
use crate::silverpelt::settings::config_opts::{
    Column, ColumnAction, ColumnSuggestion, ColumnType, ConfigOption, InnerColumnType, InnerColumnTypeStringKind, OperationSpecific, OperationType, SettingsError
};
use crate::silverpelt::value::Value;

pub(crate) fn sink() -> ConfigOption {
    ConfigOption {
        id: "sinks",
        name: "Audit Log Sinks",
        description: "A sink is a place where audit logs are sent to. This can be a channel or a webhook at this time. More sink types may be added in the future.",
        table: "auditlogs__sinks",
        guild_id: "guild_id",
        primary_key: "id",
        columns: vec![
            Column {
                id: "id",
                name: "Sink ID",
                column_type: ColumnType::new_scalar(InnerColumnType::Uuid {}),
                nullable: false,
                unique: true,
                suggestions: ColumnSuggestion::Dynamic { 
                    table_name: "auditlogs__sinks", 
                    column_name: "id"
                },
                ignored_for: vec![OperationType::Create],
                pre_checks: indexmap::indexmap! {
                    OperationType::Create => vec![
                        ColumnAction::NativeAction {
                            action: Box::new(|ctx, _state| async move {
                                let ids = sqlx::query!(
                                    "SELECT COUNT(*) FROM auditlogs__sinks WHERE guild_id = $1",
                                    ctx.guild_id.to_string()
                                )
                                .fetch_one(&ctx.pool)
                                .await
                                .map_err(|e| SettingsError::Generic {
                                    message: format!("Failed to fetch sink count: {}", e),
                                    src: "fetch_sinks_count".to_string(),
                                    typ: "internal".to_string(),
                                })?
                                .count
                                .unwrap_or(0);

                                if ids >= 10 {
                                    return Err(SettingsError::MaximumCountReached { 
                                        max: 10,
                                        current: ids,
                                    });
                                }

                                Ok(())
                            }.boxed()),
                            on_condition: None
                        },
                    ],
                },
                default_pre_checks: vec![],
            },
            Column {
                id: "type",
                name: "Sink Type",
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec!["channel", "discordhook"], kind: InnerColumnTypeStringKind::Normal }),
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::Static { suggestions: vec!["channel", "discordhook"] },
                ignored_for: vec![OperationType::Update],
                pre_checks: indexmap::indexmap! {},
                default_pre_checks: vec![],
            },
            Column {
                id: "sink",
                name: "Sink",
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::Normal }),
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                pre_checks: indexmap::indexmap! {
                    OperationType::View => vec![
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
                        }
                    ],
                },
                default_pre_checks: vec![
                    ColumnAction::NativeAction {
                        action: Box::new(|_ctx, state| async move {
                            let Some(Value::String(sink)) = state.state.get("sink") else {
                                return Err(SettingsError::MissingOrInvalidField { 
                                    field: "sink".to_string(),
                                });
                            };

                            let Some(Value::String(typ)) = state.state.get("type") else {
                                return Err(SettingsError::MissingOrInvalidField { 
                                    field: "type".to_string(),
                                });
                            };

                            if typ == "discordhook" {
                                let sink_url = url::Url::parse(sink)
                                .map_err(|e| SettingsError::SchemaCheckValidationError { 
                                    column: "sink".to_string(),
                                    check: "parse_webhook.parse_sink_to_url".to_string(),
                                    error: e.to_string(),
                                    value: serde_json::Value::String(sink.clone()),
                                    accepted_range: "Valid Discord webhook URL".to_string()
                                })?;    

                                if serenity::utils::parse_webhook(
                                    &sink_url
                                ).is_none() {
                                    return Err(SettingsError::SchemaCheckValidationError { 
                                        column: "sink".to_string(),
                                        check: "parse_webhook.parse".to_string(),
                                        error: "Discord webhook sinks must be a valid webhook URL".to_string(),
                                        value: serde_json::Value::String(sink.clone()),
                                        accepted_range: "Valid Discord webhook URL".to_string()
                                    });
                                }
                            } else if typ == "channel" {
                                sink.parse::<serenity::all::ChannelId>().map_err(|e| SettingsError::SchemaCheckValidationError {
                                    column: "sink".to_string(),
                                    check: "snowflake_parse".to_string(),
                                    value: serde_json::Value::String(sink.clone()),
                                    accepted_range: "Valid channel id".to_string(),
                                    error: e.to_string(),
                                })?;
                            } else {
                                return Err(SettingsError::SchemaCheckValidationError { 
                                    column: "type".to_string(),
                                    check: "parse_webhook.parse".to_string(),
                                    error: "Invalid sink type".to_string(),
                                    value: serde_json::Value::String(sink.clone()),
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
                                });
                            };

                            Ok(typ == "channel")
                        })
                    },
                ]
            },
            Column {
                id: "events",
                name: "Events",
                column_type: ColumnType::new_array(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::Normal }),
                nullable: true,
                unique: false,
                suggestions: ColumnSuggestion::Static { suggestions: gwevent::core::event_list().to_vec() },
                ignored_for: vec![],
                pre_checks: indexmap::indexmap! {
                    OperationType::View => vec![]
                },
                default_pre_checks: vec![
                    ColumnAction::IpcPerModuleFunction {
                        module: "auditlogs",
                        function: "check_all_events",
                        arguments: indexmap::indexmap! {
                            "events" => "{events}"
                        },
                        on_condition: None
                    }
                ]
            },
            Column {
                id: "created_at",
                name: "Created At",
                column_type: ColumnType::new_scalar(InnerColumnType::TimestampTz {}),
                nullable: false,
                unique: false,
                ignored_for: vec![OperationType::Create, OperationType::Update],
                suggestions: ColumnSuggestion::None {},
                pre_checks: indexmap::indexmap! {},
                default_pre_checks: vec![]
            },
            Column {
                id: "created_by",
                name: "Created By",
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::User }),
                ignored_for: vec![OperationType::Create, OperationType::Update],
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                pre_checks: indexmap::indexmap! {},
                default_pre_checks: vec![]
            },
            Column {
                id: "last_updated_at",
                name: "Last Updated At",
                column_type: ColumnType::new_scalar(InnerColumnType::TimestampTz {}),
                ignored_for: vec![OperationType::Create, OperationType::Update],
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                pre_checks: indexmap::indexmap! {},
                default_pre_checks: vec![]
            },
            Column {
                id: "last_updated_by",
                name: "Last Updated By",
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::User }),
                ignored_for: vec![OperationType::Create, OperationType::Update],
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                pre_checks: indexmap::indexmap! {},
                default_pre_checks: vec![]
            },
            Column {
                id: "broken",
                name: "Marked as Broken",
                column_type: ColumnType::new_scalar(InnerColumnType::Boolean {}),
                ignored_for: vec![],
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                pre_checks: indexmap::indexmap! {},
                default_pre_checks: vec![]
            },
        ],
        operations: indexmap::indexmap! {
            OperationType::View => OperationSpecific {
                corresponding_command: "list_sinks",
                columns_to_set: indexmap::indexmap! {},
            },
            OperationType::Create => OperationSpecific {
                corresponding_command: "add_sink",
                columns_to_set: indexmap::indexmap! {
                    "created_at" => "{__now}",
                    "created_by" => "{__author}",
                    "last_updated_at" => "{__now}",
                    "last_updated_by" => "{__author}",
                },
            },
            OperationType::Update => OperationSpecific {
                corresponding_command: "add_sink",
                columns_to_set: indexmap::indexmap! {
                    "last_updated_at" => "{__now}",
                    "last_updated_by" => "{__author}",
                },
            },
        }
    }
}
use futures_util::FutureExt;
use crate::silverpelt::settings::config_opts::{
    Column, ColumnAction, ColumnSuggestion, ColumnType, InnerColumnType, ConfigOption, OperationSpecific,
    OperationType
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
                                .await?
                                .count
                                .unwrap_or(0);

                                if ids >= 10 {
                                    return Err("You have reached the maximum number of sinks allowed (10). Please remove a sink before adding a new one.".into());
                                }

                                Ok(())
                            }.boxed())
                        },
                    ],
                },
                default_pre_checks: vec![],
            },
            Column {
                id: "type",
                name: "Sink Type",
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec!["channel", "discordhook"] }),
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
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![] }),
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
                            }.boxed())
                        }
                    ]
                },
                default_pre_checks: vec![
                    ColumnAction::NativeAction {
                        action: Box::new(|_ctx, state| async move {
                            let Some(Value::String(sink)) = state.state.get("sink") else {
                                return Err("Sink must be set.".into());
                            };

                            let Some(Value::String(typ)) = state.state.get("type") else {
                                return Err("Sink type must be set.".into());
                            };

                            if typ == "discordhook" {
                                if serenity::utils::parse_webhook(&sink.parse()?).is_none() {
                                    return Err("Discord webhooks sinks must be a webhook.".into());
                                }
                            } else if typ == "channel" {
                                sink.parse::<serenity::all::ChannelId>().map_err(|e| format!("Invalid channel ID: {}", e))?;
                            } else {
                                return Err("Invalid sink type.".into());
                            }

                            Ok(())
                        }.boxed())
                    },
                ]
            },
            Column {
                id: "events",
                name: "Events",
                column_type: ColumnType::new_array(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![] }),
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
                        }
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
                column_type: ColumnType::new_scalar(InnerColumnType::User {}),
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
                column_type: ColumnType::new_scalar(InnerColumnType::User {}),
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
                    "auditlogs__sinks" => indexmap::indexmap! {
                        "created_at" => "{__now}",
                        "created_by" => "{__author}",
                        "last_updated_at" => "{__now}",
                        "last_updated_by" => "{__author}",
                    },
                },
            },
            OperationType::Update => OperationSpecific {
                corresponding_command: "add_sink",
                columns_to_set: indexmap::indexmap! {
                    "auditlogs__sinks" => indexmap::indexmap! {
                        "last_updated_at" => "{__now}",
                        "last_updated_by" => "{__author}",
                    },
                },
            },
        }
    }
}
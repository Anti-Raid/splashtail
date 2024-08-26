use futures_util::future::FutureExt;
use module_settings::types::{
    settings_wrap_columns, settings_wrap_precheck, settings_wrap_postactions, settings_wrap_datastore, Column, ColumnAction, ColumnSuggestion, ColumnType, ConfigOption, InnerColumnType, InnerColumnTypeStringKind, InnerColumnTypeStringKindTemplateKind, ColumnTypeDynamicClause, OperationSpecific, OperationType, SettingsError
};
use module_settings::data_stores::PostgresDataStore;
use std::sync::LazyLock;
use serenity::all::{ChannelType, Permissions};
use splashcore_rs::value::Value;

pub static SINK: LazyLock<ConfigOption> = LazyLock::new(|| {
    ConfigOption {
        id: "sinks",
        name: "Audit Log Sinks",
        description: "A sink is a place where audit logs are sent to. This can be a channel or a webhook at this time. More sink types may be added in the future.",
        table: "auditlogs__sinks",
        common_filters: indexmap::indexmap! {},
        default_common_filters: indexmap::indexmap! {
            "guild_id" => "{__guild_id}"
        },
        primary_key: "id",
        max_return: 15,
        max_entries: Some(10),
        data_store: settings_wrap_datastore(PostgresDataStore {}),
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
                secret: false,
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
                default_pre_checks: settings_wrap_precheck(vec![]),
            },
            module_settings::common_columns::guild_id("guild_id", "Guild ID", "The Guild ID the sink belongs to"),
            Column {
                id: "type",
                name: "Sink Type",
                description: "The type of sink. This can be a sink that sends messages to a channel (`channel`) or a discord webhook (`discordhook`).",
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec!["channel", "discordhook"], kind: InnerColumnTypeStringKind::Normal }),
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
                default_pre_checks: settings_wrap_precheck(vec![]),
            },
            Column {
                id: "sink",
                name: "Sink",
                description: "The sink where the logs are sent to. This can be a channel ID (if `channel`) or a discord webhook URL (if `discordhook`).",
                column_type: ColumnType::new_dynamic(
                    vec![
                        ColumnTypeDynamicClause {
                            field: "{type}",
                            value: Value::String("discordhook".to_string()),
                            column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::Normal })
                        },
                        ColumnTypeDynamicClause {
                            field: "{type}",
                            value: Value::String("channel".to_string()),
                            column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::Channel {
                                allowed_types: vec![ChannelType::Text, ChannelType::Voice, ChannelType::PublicThread, ChannelType::PrivateThread, ChannelType::News],
                                needed_bot_permissions: Permissions::VIEW_CHANNEL | Permissions::SEND_MESSAGES | Permissions::EMBED_LINKS,
                            } })
                        }
                    ]
                ),
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
                default_pre_checks: settings_wrap_precheck(vec![
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
                secret: false,
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
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::Template { kind: InnerColumnTypeStringKindTemplateKind::Message {
                } } }),
                ignored_for: vec![],
                secret: false,
                nullable: true,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
                default_pre_checks: settings_wrap_precheck(vec![])
            },
            Column {
                id: "send_json_context",
                name: "Send JSON Context",
                description: "Whether to send the JSON context of the event to the sink. This can be useful for seeing exactly what happened to your server.",
                column_type: ColumnType::new_scalar(InnerColumnType::Boolean {}),
                ignored_for: vec![],
                secret: false,
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
                default_pre_checks: settings_wrap_precheck(vec![])
            },
            Column {
                id: "broken",
                name: "Marked as Broken",
                description: "If the sink is marked as broken, it will not be used for sending logs. This can be useful in debugging too!",
                column_type: ColumnType::new_scalar(InnerColumnType::Boolean {}),
                ignored_for: vec![OperationType::Create],
                secret: false,
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
                default_pre_checks: settings_wrap_precheck(vec![])
            },
            module_settings::common_columns::created_at(),
            module_settings::common_columns::created_by(),
            module_settings::common_columns::last_updated_at(),
            module_settings::common_columns::last_updated_by(),
        ]),
        title_template: "{type} {sink} [{id}]",
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
        },
        post_actions: settings_wrap_postactions(vec![ColumnAction::NativeAction {
            action: Box::new(|ctx, _state| {
                async move {
                    super::cache::SINKS_CACHE
                        .invalidate(&ctx.guild_id)
                        .await;
    
                    Ok(())
                }
                .boxed()
            }),
            on_condition: Some(|ctx, _state| Ok(ctx.operation_type != OperationType::View)),
        }]),
    }
});
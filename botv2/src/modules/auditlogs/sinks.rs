use crate::silverpelt::config_opts::{
    Column, ColumnAction, ColumnSuggestion, ColumnType, ConfigOption, OperationSpecific,
    OperationType
};

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
                    ],
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
                pre_checks: indexmap::indexmap! {
                    OperationType::View => vec![]
                },
                default_pre_checks: vec![
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
            },
            Column {
                id: "events",
                name: "Events",
                column_type: ColumnType::String { min_length: None, max_length: None, allowed_values: vec![] },
                nullable: false,
                unique: false,
                array: true,
                suggestions: ColumnSuggestion::Static { suggestions: gwevent::core::event_list().to_vec() },
                readonly: indexmap::indexmap! {},
                pre_checks: indexmap::indexmap! {
                    OperationType::View => vec![]
                },
                default_pre_checks: vec![
                    ColumnAction::IpcPerModuleFunction {
                        module: "auditlogs",
                        function: "check_all_events",
                        arguments: indexmap::indexmap! {
                            "events" => "events"
                        }
                    }
                ]
            },
            Column {
                id: "created_at",
                name: "Created At",
                column_type: ColumnType::Timestamp {},
                nullable: false,
                unique: false,
                array: false,
                suggestions: ColumnSuggestion::Static { suggestions: vec![] },
                readonly: indexmap::indexmap! {
                    OperationType::Create => true,
                    OperationType::Update => true,
                },
                pre_checks: indexmap::indexmap! {},
                default_pre_checks: vec![]
            },
            Column {
                id: "created_by",
                name: "Created By",
                column_type: ColumnType::User {},
                nullable: false,
                unique: false,
                array: false,
                suggestions: ColumnSuggestion::Static { suggestions: vec![] },
                readonly: indexmap::indexmap! {
                    OperationType::Create => true,
                    OperationType::Update => true,
                },
                pre_checks: indexmap::indexmap! {},
                default_pre_checks: vec![]
            },
            Column {
                id: "last_updated_by",
                name: "Last Updated By",
                column_type: ColumnType::User {},
                nullable: false,
                unique: false,
                array: false,
                suggestions: ColumnSuggestion::Static { suggestions: vec![] },
                readonly: indexmap::indexmap! {
                    OperationType::Create => true,
                    OperationType::Update => true,
                },
                pre_checks: indexmap::indexmap! {},
                default_pre_checks: vec![]
            },
            Column {
                id: "broken",
                name: "Marked as Broken",
                column_type: ColumnType::Boolean {},
                nullable: false,
                unique: false,
                array: false,
                suggestions: ColumnSuggestion::Static { suggestions: vec![] },
                readonly: indexmap::indexmap! {},
                pre_checks: indexmap::indexmap! {},
                default_pre_checks: vec![]
            },
        ],
        operations: indexmap::indexmap! {
            OperationType::View => OperationSpecific {
                corresponding_command: "list_sinks",
                column_ids: vec![],
                columns_to_set: indexmap::indexmap! {},
            },
        }
    }
}
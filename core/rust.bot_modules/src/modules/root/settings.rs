use futures_util::FutureExt;
use module_settings::types::{
    Column, ColumnAction, ColumnSuggestion, ColumnType, ConfigOption, InnerColumnType,
    InnerColumnTypeStringKind, OperationSpecific, OperationType, SettingsError,
};
use splashcore_rs::value::Value;

pub(crate) fn maintenance() -> ConfigOption {
    ConfigOption {
        id: "maintenance",
        name: "Maintenance",
        description: "Stores all bot maintenances",
        table: "maintenance",
        guild_id: "published_from",
        primary_key: "id",
        max_entries: 1024,
        columns: vec![
            Column {
                id: "id",
                name: "Maintenance ID",
                column_type: ColumnType::new_scalar(InnerColumnType::Uuid {}),
                nullable: false,
                unique: true,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![OperationType::Create],
                secret: None,
                pre_checks: indexmap::indexmap! {
                    OperationType::Create => vec![
                        // Set sink display type
                        ColumnAction::NativeAction {
                            action: Box::new(|ctx, state| async move {
                                // Set ID
                                let id = botox::crypto::gen_random(128);
                                state.state.insert("id".to_string(), Value::String(id.to_string()));
                                state.bypass_ignore_for.insert("id".to_string());

                                // Also ensure published_from is the official support server
                                if ctx.guild_id != config::CONFIG.servers.main.get() {
                                    return Err(SettingsError::SchemaCheckValidationError {
                                        column: "published_from".to_string(),
                                        check: "maintenance.published_from".to_string(),
                                        error: "Maintenances must be published from the official support server".to_string(),
                                        value: Value::String(ctx.guild_id.to_string()),
                                        accepted_range: config::CONFIG.servers.main.get().to_string()
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
                id: "title",
                name: "Title",
                column_type: ColumnType::new_scalar(InnerColumnType::String {
                    min_length: None,
                    max_length: None,
                    allowed_values: vec![],
                    kind: InnerColumnTypeStringKind::Normal,
                }),
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: None,
                pre_checks: indexmap::indexmap! {},
                default_pre_checks: vec![],
            },
            Column {
                id: "description",
                name: "Description",
                column_type: ColumnType::new_scalar(InnerColumnType::String {
                    min_length: None,
                    max_length: None,
                    allowed_values: vec![],
                    kind: InnerColumnTypeStringKind::Normal,
                }),
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: None,
                pre_checks: indexmap::indexmap! {},
                default_pre_checks: vec![],
            },
            Column {
                id: "entries",
                name: "Entries",
                column_type: ColumnType::new_array(InnerColumnType::String {
                    min_length: None,
                    max_length: None,
                    allowed_values: vec![],
                    kind: InnerColumnTypeStringKind::Normal,
                }),
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: None,
                pre_checks: indexmap::indexmap! {},
                default_pre_checks: vec![],
            },
            module_settings::common_columns::created_at(),
            module_settings::common_columns::created_by(),
            module_settings::common_columns::last_updated_at(),
            module_settings::common_columns::last_updated_by(),
            Column {
                id: "current",
                name: "Currently Running",
                column_type: ColumnType::new_scalar(InnerColumnType::Boolean {}),
                ignored_for: vec![OperationType::Create],
                secret: None,
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                pre_checks: indexmap::indexmap! {},
                default_pre_checks: vec![],
            },
        ],
        operations: indexmap::indexmap! {
            OperationType::View => OperationSpecific {
                corresponding_command: "sudo maintenance_list",
                columns_to_set: indexmap::indexmap! {},
            },
            OperationType::Create => OperationSpecific {
                corresponding_command: "sudo maintenance_create",
                columns_to_set: indexmap::indexmap! {
                    "created_at" => "{__now}",
                    "created_by" => "{__author}",
                    "last_updated_at" => "{__now}",
                    "last_updated_by" => "{__author}",
                },
            },
            OperationType::Update => OperationSpecific {
                corresponding_command: "sudo maintenance_update",
                columns_to_set: indexmap::indexmap! {
                    "last_updated_at" => "{__now}",
                    "last_updated_by" => "{__author}",
                },
            },
            OperationType::Delete => OperationSpecific {
                corresponding_command: "sudo maintenance_delete",
                columns_to_set: indexmap::indexmap! {},
            },
        },
    }
}

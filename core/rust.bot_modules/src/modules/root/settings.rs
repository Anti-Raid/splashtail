use futures_util::FutureExt;
use module_settings::types::{
    settings_wrap_columns, settings_wrap_precheck, Column, ColumnAction, ColumnSuggestion,
    ColumnType, ConfigOption, InnerColumnType, InnerColumnTypeStringKind, OperationSpecific,
    OperationType, SettingsError,
};
use once_cell::sync::Lazy;
use splashcore_rs::value::Value;

pub static MAINTENANCE: Lazy<ConfigOption> = Lazy::new(|| {
    ConfigOption {
        id: "maintenance",
        name: "Maintenance",
        description: "Stores all bot maintenances",
        table: "maintenance",
        guild_id: "published_from",
        primary_key: "id",
        max_entries: 1024,
        columns: settings_wrap_columns(vec![
            Column {
                id: "id",
                name: "Maintenance ID",
                description: "The unique ID of the maintenance",
                column_type: ColumnType::new_scalar(InnerColumnType::String {
                    min_length: Some(128),
                    max_length: Some(128),
                    allowed_values: vec![],
                    kind: InnerColumnTypeStringKind::Normal,
                }),
                nullable: false,
                unique: true,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![OperationType::Create],
                secret: None,
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {
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
                                        accepted_range: config::CONFIG.servers.main.get().to_string()
                                    });
                                }
                                Ok(())
                            }.boxed()),
                            on_condition: None
                        },
                    ],
                }),
                default_pre_checks: settings_wrap_precheck(vec![]),
            },
            Column {
                id: "title",
                name: "Title",
                description: "The title of the maintenance",
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
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
                default_pre_checks: settings_wrap_precheck(vec![]),
            },
            Column {
                id: "description",
                name: "Description",
                description: "The description of the maintenance",
                column_type: ColumnType::new_scalar(InnerColumnType::String {
                    min_length: None,
                    max_length: None,
                    allowed_values: vec![],
                    kind: InnerColumnTypeStringKind::Textarea,
                }),
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: None,
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
                default_pre_checks: settings_wrap_precheck(vec![]),
            },
            Column {
                id: "entries",
                name: "Entries",
                description: "The entries of the maintenance",
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
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
                default_pre_checks: settings_wrap_precheck(vec![]),
            },
            module_settings::common_columns::created_at(),
            module_settings::common_columns::created_by(),
            module_settings::common_columns::last_updated_at(),
            module_settings::common_columns::last_updated_by(),
            Column {
                id: "current",
                name: "Currently Running",
                description: "Whether the maintenance is currently running",
                column_type: ColumnType::new_scalar(InnerColumnType::Boolean {}),
                ignored_for: vec![OperationType::Create],
                secret: None,
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
                default_pre_checks: settings_wrap_precheck(vec![]),
            },
        ]),
        title_template: "{title} - {id}",
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
});

pub static INSPECTOR_FAKE_BOTS: Lazy<ConfigOption> = Lazy::new(|| ConfigOption {
    id: "inspector__fake_bots",
    name: "Inspector (Fake Bots)",
    description: "Stores a list of official bots and their ID to allow detection of fake bots",
    table: "inspector__fake_bots",
    guild_id: "published_from",
    primary_key: "bot_id",
    max_entries: 1024,
    columns: settings_wrap_columns(vec![
        Column {
            id: "bot_id",
            name: "Bot ID",
            description: "The ID of the bot",
            column_type: ColumnType::new_scalar(InnerColumnType::String {
                min_length: None,
                max_length: Some(32),
                allowed_values: vec![],
                kind: InnerColumnTypeStringKind::Normal,
            }),
            nullable: false,
            unique: true,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: None,
            pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
            default_pre_checks: settings_wrap_precheck(vec![]),
        },
        Column {
            id: "name",
            name: "Bot Name",
            description: "The name of the bot",
            column_type: ColumnType::new_scalar(InnerColumnType::String {
                min_length: None,
                max_length: Some(512),
                allowed_values: vec![],
                kind: InnerColumnTypeStringKind::Normal,
            }),
            nullable: false,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: None,
            pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
            default_pre_checks: settings_wrap_precheck(vec![]),
        },
        Column {
            id: "official_bot_ids",
            name: "Official Bot IDs",
            description: "The discriminator of the bot",
            column_type: ColumnType::new_array(InnerColumnType::String {
                min_length: None,
                max_length: Some(32),
                allowed_values: vec![],
                kind: InnerColumnTypeStringKind::Normal,
            }),
            nullable: false,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: None,
            pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
            default_pre_checks: settings_wrap_precheck(vec![]),
        },
        module_settings::common_columns::created_at(),
        module_settings::common_columns::created_by(),
        module_settings::common_columns::last_updated_at(),
        module_settings::common_columns::last_updated_by(),
        Column {
            id: "comments",
            name: "Comments",
            description: "Comments about the entry",
            column_type: ColumnType::new_scalar(InnerColumnType::String {
                min_length: None,
                max_length: Some(1024),
                allowed_values: vec![],
                kind: InnerColumnTypeStringKind::Normal,
            }),
            nullable: true,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: None,
            pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
            default_pre_checks: settings_wrap_precheck(vec![]),
        },
    ]),
    title_template: "{name} - {bot_id}",
    operations: indexmap::indexmap! {
        OperationType::View => OperationSpecific {
            corresponding_command: "sudo inspector__fake_bots_list",
            columns_to_set: indexmap::indexmap! {
            },
        },
        OperationType::Create => OperationSpecific {
            corresponding_command: "sudo inspector__fake_bots_create",
            columns_to_set: indexmap::indexmap! {
                "created_at" => "{__now}",
                "created_by" => "{__author}",
                "last_updated_at" => "{__now}",
                "last_updated_by" => "{__author}",
            },
        },
        OperationType::Update => OperationSpecific {
            corresponding_command: "sudo inspector__fake_bots_update",
            columns_to_set: indexmap::indexmap! {
                "last_updated_at" => "{__now}",
                "last_updated_by" => "{__author}",
            },
        },
        OperationType::Delete => OperationSpecific {
            corresponding_command: "sudo inspector__fake_bots_delete",
            columns_to_set: indexmap::indexmap! {},
        },
    },
});

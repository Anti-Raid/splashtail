use futures_util::future::FutureExt;
use module_settings::{
    data_stores::PostgresDataStore,
    types::{
        settings_wrap_columns, settings_wrap_datastore, settings_wrap_postactions,
        settings_wrap_precheck, Column, ColumnAction, ColumnSuggestion, ColumnType, ConfigOption,
        InnerColumnType, InnerColumnTypeStringKind, OperationSpecific, OperationType,
        SettingsError,
    },
};
use once_cell::sync::Lazy;

pub static CAN_USE_BOT: Lazy<ConfigOption> = Lazy::new(|| ConfigOption {
    id: "can_use_bot",
    name: "Can Use Bot Whitelist",
    description: "Stores a list of users and guilds who can use the bot",
    table: "can_use_bot",
    common_filters: indexmap::indexmap! {},
    default_common_filters: indexmap::indexmap! {},
    primary_key: "id",
    max_entries: 1024,
    data_store: settings_wrap_datastore(PostgresDataStore {}),
    columns: settings_wrap_columns(vec![
        Column {
            id: "id",
            name: "ID",
            description: "The ID of the entity",
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
            secret: false,
            pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
            default_pre_checks: settings_wrap_precheck(vec![]),
        },
        Column {
            id: "type",
            name: "Type",
            description: "The type of the entity",
            column_type: ColumnType::new_scalar(InnerColumnType::String {
                min_length: None,
                max_length: Some(512),
                allowed_values: vec!["user", "guild"],
                kind: InnerColumnTypeStringKind::Normal,
            }),
            nullable: false,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
            pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
            default_pre_checks: settings_wrap_precheck(vec![]),
        },
        Column {
            id: "name",
            name: "Name",
            description: "The name of the entity",
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
            secret: false,
            pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
            default_pre_checks: settings_wrap_precheck(vec![]),
        },
        Column {
            id: "protected",
            name: "Protected",
            description:
                "The protected status. Cannot be changed without manual database intervention",
            column_type: ColumnType::new_scalar(InnerColumnType::Boolean {}),
            nullable: false,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![OperationType::Create, OperationType::Update],
            secret: false,
            pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
            default_pre_checks: settings_wrap_precheck(vec![ColumnAction::NativeAction {
                action: Box::new(|ctx, state| {
                    async move {
                        if ctx.operation_type != OperationType::Update
                            && ctx.operation_type != OperationType::Delete
                        {
                            return Ok(());
                        }

                        if let Some(splashcore_rs::value::Value::Boolean(true)) =
                            state.state.get("protected")
                        {
                            return Err(SettingsError::Generic {
                                message: "Cannot change protected entries".to_string(),
                                src: "can_use_bot::protected".to_string(),
                                typ: "internal".to_string(),
                            });
                        }

                        Ok(())
                    }
                    .boxed()
                }),
                on_condition: None,
            }]),
        },
        module_settings::common_columns::created_at(),
        module_settings::common_columns::created_by(),
        module_settings::common_columns::last_updated_at(),
        module_settings::common_columns::last_updated_by(),
    ]),
    title_template: "{name} - {id}",
    operations: indexmap::indexmap! {
        OperationType::View => OperationSpecific {
            corresponding_command: "sudo can_use_bot_list",
            columns_to_set: indexmap::indexmap! {},
        },
        OperationType::Create => OperationSpecific {
            corresponding_command: "sudo can_use_bot_add",
            columns_to_set: indexmap::indexmap! {
                "created_at" => "{__now}",
                "created_by" => "{__author}",
                "last_updated_at" => "{__now}",
                "last_updated_by" => "{__author}",
            },
        },
        OperationType::Update => OperationSpecific {
            corresponding_command: "sudo can_use_bot_update",
            columns_to_set: indexmap::indexmap! {
                "last_updated_at" => "{__now}",
                "last_updated_by" => "{__author}",
            },
        },
        OperationType::Delete => OperationSpecific {
            corresponding_command: "sudo can_use_bot_delete",
            columns_to_set: indexmap::indexmap! {},
        },
    },
    post_actions: settings_wrap_postactions(vec![ColumnAction::IpcPerModuleFunction {
        module: "root",
        function: "reset_can_use_bot_whitelist",
        arguments: indexmap::indexmap! {},
        on_condition: None,
    }]),
});

pub static INSPECTOR_FAKE_BOTS: Lazy<ConfigOption> = Lazy::new(|| ConfigOption {
    id: "inspector__fake_bots",
    name: "Inspector (Fake Bots)",
    description: "Stores a list of official bots and their ID to allow detection of fake bots",
    table: "inspector__fake_bots",
    common_filters: indexmap::indexmap! {},
    default_common_filters: indexmap::indexmap! {},
    primary_key: "bot_id",
    max_entries: 1024,
    data_store: settings_wrap_datastore(PostgresDataStore {}),
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
            secret: false,
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
            secret: false,
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
            secret: false,
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
            secret: false,
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
    post_actions: settings_wrap_postactions(vec![ColumnAction::NativeAction {
        action: Box::new(|ctx, _state| {
            async move {
                if ctx.operation_type == OperationType::View {
                    return Ok(());
                }

                crate::modules::inspector::cache::setup_fake_bots_cache(ctx.pool)
                    .await
                    .map_err(|e| SettingsError::Generic {
                        message: format!("Failed to setup fake bots cache: {}", e),
                        src: "inspector__fake_bots::post_actions".to_string(),
                        typ: "internal".to_string(),
                    })?;

                Ok(())
            }
            .boxed()
        }),
        on_condition: None,
    }]),
});

use module_settings::data_stores::PostgresDataStore;
use module_settings::types::{
    settings_wrap, Column, ColumnSuggestion, ColumnType, ConfigOption, InnerColumnType,
    InnerColumnTypeStringKind, NoOpPostAction, NoOpValidator, OperationSpecific, OperationType,
};
use std::sync::LazyLock;

pub static CUSTOM_TAGS: LazyLock<ConfigOption> = LazyLock::new(|| ConfigOption {
    id: "custom_tags",
    name: "Custom Tags",
    description: "Create custom tags on your server",
    table: "tags__custom_tags",
    common_filters: indexmap::indexmap! {},
    default_common_filters: indexmap::indexmap! {
        "guild_id" => "{__guild_id}"
    },
    primary_key: "id",
    max_return: 10,
    max_entries: Some(100),
    data_store: settings_wrap(PostgresDataStore {}),
    columns: settings_wrap(vec![
        Column {
            id: "id",
            name: "ID",
            description: "The ID used to refer to this tag",
            column_type: ColumnType::new_scalar(InnerColumnType::Uuid {}),
            nullable: false,
            default: None,
            unique: true,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![OperationType::Create],
            secret: false,
        },
        module_settings::common_columns::guild_id(
            "guild_id",
            "Guild ID",
            "The Guild ID the tag belongs to",
        ),
        Column {
            id: "name",
            name: "Name",
            description: "The name of the tag",
            column_type: ColumnType::new_scalar(InnerColumnType::String {
                min_length: Some(1),
                max_length: Some(100),
                allowed_values: vec![],
                kind: InnerColumnTypeStringKind::Normal,
            }),
            nullable: false,
            default: None,
            unique: true,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
        },
        Column {
            id: "template",
            name: "Template",
            description: "A template for the tag. Must return a Message",
            column_type: ColumnType::new_scalar(InnerColumnType::String {
                min_length: None,
                max_length: None,
                allowed_values: vec![],
                kind: InnerColumnTypeStringKind::Template {
                    kind: "message",
                    ctx: "TagContext",
                },
            }),
            nullable: false,
            default: None,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
        },
        module_settings::common_columns::created_at(),
        module_settings::common_columns::created_by(),
        module_settings::common_columns::last_updated_at(),
        module_settings::common_columns::last_updated_by(),
    ]),
    title_template: "{name}",
    operations: indexmap::indexmap! {
        OperationType::View => OperationSpecific {
            columns_to_set: indexmap::indexmap! {},
        },
        OperationType::Create => OperationSpecific {
            columns_to_set: indexmap::indexmap! {
                "created_at" => "{__now}",
                "created_by" => "{__author}",
                "last_updated_at" => "{__now}",
                "last_updated_by" => "{__author}",
            },
        },
        OperationType::Update => OperationSpecific {
            columns_to_set: indexmap::indexmap! {
                "last_updated_at" => "{__now}",
                "last_updated_by" => "{__author}",
            },
        },
        OperationType::Delete => OperationSpecific {
            columns_to_set: indexmap::indexmap! {},
        },
    },
    validator: settings_wrap(NoOpValidator {}),
    post_action: settings_wrap(NoOpPostAction {}),
});

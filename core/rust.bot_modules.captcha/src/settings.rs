use module_settings::data_stores::PostgresDataStore;
use module_settings::types::{
    settings_wrap, Column, ColumnSuggestion, ColumnType, ConfigOption, InnerColumnType,
    InnerColumnTypeStringKind, NoOpPostAction, NoOpValidator, OperationSpecific, OperationType,
};
use std::sync::LazyLock;

pub static CAPTCHA: LazyLock<ConfigOption> = LazyLock::new(|| ConfigOption {
    id: "captcha",
    name: "Captcha Settings",
    description: "CAPTCHA Protection",
    table: "captcha__guild_captchas",
    common_filters: indexmap::indexmap! {},
    default_common_filters: indexmap::indexmap! {
        "guild_id" => "{__guild_id}"
    },
    primary_key: "guild_id",
    max_return: 1,
    max_entries: Some(1),
    data_store: settings_wrap(PostgresDataStore {}),
    columns: settings_wrap(vec![
        module_settings::common_columns::guild_id(
            "guild_id",
            "Guild ID",
            "The Guild ID the CAPTCHA belongs to",
        ),
        Column {
            id: "template",
            name: "Template",
            description: "A template that returns the filters to use for the CAPTCHA",
            column_type: ColumnType::new_scalar(InnerColumnType::String {
                min_length: None,
                max_length: None,
                allowed_values: vec![],
                kind: InnerColumnTypeStringKind::Template {
                    kind: "captcha",
                    ctx: "CaptchaContext",
                },
            }),
            nullable: false,
            default: None,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
        },
    ]),
    title_template: "CAPTCHA Setup",
    operations: indexmap::indexmap! {
        OperationType::View => OperationSpecific {
            columns_to_set: indexmap::indexmap! {},
        },
        OperationType::Create => OperationSpecific {
            columns_to_set: indexmap::indexmap! {
            },
        },
        OperationType::Update => OperationSpecific {
            columns_to_set: indexmap::indexmap! {
            },
        },
        OperationType::Delete => OperationSpecific {
            columns_to_set: indexmap::indexmap! {},
        },
    },
    validator: settings_wrap(NoOpValidator {}),
    post_action: settings_wrap(NoOpPostAction {}),
});

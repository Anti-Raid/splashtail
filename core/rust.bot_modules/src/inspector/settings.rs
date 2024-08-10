use module_settings::{
    data_stores::PostgresDataStore,
    types::{
        settings_wrap_columns, settings_wrap_datastore, settings_wrap_postactions,
        settings_wrap_precheck, Column, ColumnSuggestion, ColumnType, ConfigOption,
        InnerColumnType, OperationSpecific, OperationType,
    },
};
use once_cell::sync::Lazy;

use super::types::{DehoistOptions, FakeBotDetectionOptions, GuildProtectionOptions};

pub static INSPECTOR_OPTIONS: Lazy<ConfigOption> = Lazy::new(|| ConfigOption {
    id: "options",
    name: "Inspector Options",
    description: "Setup inspector here",
    table: "inspector__options",
    common_filters: indexmap::indexmap! {},
    default_common_filters: indexmap::indexmap! {
        "guild_id" => "{__guild_id}"
    },
    primary_key: "guild_id",
    max_entries: Some(1),
    data_store: settings_wrap_datastore(PostgresDataStore {}),
    columns: settings_wrap_columns(vec![
        module_settings::common_columns::guild_id(
            "guild_id",
            "Guild ID",
            "Guild ID of the server in question",
        ),
        Column {
            id: "minimum_account_age",
            name: "Minimum Account Age",
            description: "Minimum account age required to join the server",
            column_type: ColumnType::new_scalar(InnerColumnType::Integer {}),
            nullable: true,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
            pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
            default_pre_checks: settings_wrap_precheck(vec![]),
        },
        Column {
            id: "maximum_account_age",
            name: "Maximum Account Age",
            description: "Maximum account age to join the server",
            column_type: ColumnType::new_scalar(InnerColumnType::Integer {}),
            nullable: true,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
            pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
            default_pre_checks: settings_wrap_precheck(vec![]),
        },
        Column {
            id: "anti_invite",
            name: "Anti Invite",
            description: "Number of stings to give when an invite is sent",
            column_type: ColumnType::new_scalar(InnerColumnType::Integer {}),
            nullable: true,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
            pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
            default_pre_checks: settings_wrap_precheck(vec![]),
        },
        Column {
            id: "anti_everyone",
            name: "Anti Everyone",
            description: "Number of stings to give when an everyone ping is sent",
            column_type: ColumnType::new_scalar(InnerColumnType::Integer {}),
            nullable: true,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
            pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
            default_pre_checks: settings_wrap_precheck(vec![]),
        },
        Column {
            id: "sting_retention",
            name: "Sting Retention",
            description: "Number of seconds to keep stings for",
            column_type: ColumnType::new_scalar(InnerColumnType::Integer {}),
            nullable: false,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
            pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
            default_pre_checks: settings_wrap_precheck(vec![]),
        },
        Column {
            id: "hoist_detection",
            name: "Hoist Detection",
            description: "Hoist detection options",
            column_type: ColumnType::new_scalar(InnerColumnType::BitFlag {
                values: DehoistOptions::all()
                    .into_iter()
                    .map(|x| (x.to_string(), x.bits() as i64))
                    .collect(),
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
            id: "guild_protection",
            name: "Guild Protection",
            description: "Guild protection options",
            column_type: ColumnType::new_scalar(InnerColumnType::BitFlag {
                values: GuildProtectionOptions::all()
                    .into_iter()
                    .map(|x| (x.to_string(), x.bits() as i64))
                    .collect(),
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
            id: "fake_bot_detection",
            name: "Fake Bot Detection",
            description: "Fake bot detection options",
            column_type: ColumnType::new_scalar(InnerColumnType::BitFlag {
                values: FakeBotDetectionOptions::all()
                    .into_iter()
                    .map(|x| (x.to_string(), x.bits() as i64))
                    .collect(),
            }),
            nullable: false,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
            pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
            default_pre_checks: settings_wrap_precheck(vec![]),
        },
    ]),
    title_template: "Servers Inspector Setup",
    operations: indexmap::indexmap! {
        OperationType::View => OperationSpecific {
            corresponding_command: "inspector list",
            columns_to_set: indexmap::indexmap! {},
        },
        OperationType::Create => OperationSpecific {
            corresponding_command: "inspector setup",
            columns_to_set: indexmap::indexmap! {},
        },
        OperationType::Update => OperationSpecific {
            corresponding_command: "inspector update",
            columns_to_set: indexmap::indexmap! {},
        },
        OperationType::Delete => OperationSpecific {
            corresponding_command: "inspector disable",
            columns_to_set: indexmap::indexmap! {},
        },
    },
    post_actions: settings_wrap_postactions(vec![]),
});

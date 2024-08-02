use module_settings::{
    data_stores::PostgresDataStore,
    types::{
        settings_wrap_columns, settings_wrap_datastore, settings_wrap_postactions,
        settings_wrap_precheck, Column, ColumnSuggestion, ColumnType, ConfigOption,
        InnerColumnType, InnerColumnTypeStringKind, OperationSpecific, OperationType,
    },
};
use once_cell::sync::Lazy;
use strum::VariantNames;

pub static GUILD_LIMITS: Lazy<ConfigOption> = Lazy::new(|| {
    ConfigOption {
        id: "guild_limits",
        name: "Limits",
        description: "Limits can be used to rate limit actions on your server. For example, you can use limits to enforce 2 channels are created every minute. Once a limit has been exceeded, the infringing user will be given `stings` (like a bee!). Punishments are then applied in a unified and consistent fashion based on the number of stings a user has.",
        table: "limits__guild_limits",
        common_filters: indexmap::indexmap! {},
        default_common_filters: indexmap::indexmap! {
            "guild_id" => "{__guild_id}"
        },
        primary_key: "limit_id",
        max_entries: 10,
        data_store: settings_wrap_datastore(PostgresDataStore {}),
        columns: settings_wrap_columns(vec![
            Column {
                id: "limit_id",
                name: "Limit ID",
                description: "The unique identifier for the limit.",
                column_type: ColumnType::new_scalar(InnerColumnType::Uuid {}),
                nullable: false,
                unique: true,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![OperationType::Create],
                secret: false,
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
                default_pre_checks: settings_wrap_precheck(vec![]),
            },
            Column {
                id: "limit_name",
                name: "Limit Name",
                description: "The name to give to the limit",
                column_type: ColumnType::new_scalar(InnerColumnType::String {
                    kind: InnerColumnTypeStringKind::Normal,
                    min_length: None,
                    max_length: Some(64),
                    allowed_values: vec![],
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
                id: "limit_type",
                name: "Limit Type",
                description: "The type of limit",
                column_type: ColumnType::new_scalar(InnerColumnType::String {
                    kind: InnerColumnTypeStringKind::Normal,
                    min_length: None,
                    max_length: Some(64),
                    allowed_values: super::core::LimitTypes::VARIANTS.to_vec(),
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
                id: "limit_per",
                name: "Limit Per",
                description: "How many times to allow this action to occur in the unit. E.g. for '2 channel deletes per hour', the 2 would be the limit_per",
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
                id: "limit_time",
                name: "Limit Time",
                description: "How many unit of time in which limit_per actions can be executed. E.g. for '2 channel deletes per hour', '1 hour' would be the limit_time",
                column_type: ColumnType::new_scalar(InnerColumnType::Interval {}),
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
                default_pre_checks: settings_wrap_precheck(vec![]),
            },
            Column {
                id: "stings",
                name: "Stings",
                description: "How many stings should the user be given when they exceed the limit",
                column_type: ColumnType::new_scalar(InnerColumnType::Integer {}),
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
        ]),
        title_template: "{limit_name}: On {limit_type}, {limit_per} times every {limit_time} [{limit_id}]",
        operations: indexmap::indexmap! {
            OperationType::View => OperationSpecific {
                corresponding_command: "limits view",
                columns_to_set: indexmap::indexmap! {},
            },
            OperationType::Create => OperationSpecific {
                corresponding_command: "limits add",
                columns_to_set: indexmap::indexmap! {
                    "created_at" => "{__now}",
                    "created_by" => "{__author}",
                    "last_updated_at" => "{__now}",
                    "last_updated_by" => "{__author}",
                },
            },
            OperationType::Update => OperationSpecific {
                corresponding_command: "limits update",
                columns_to_set: indexmap::indexmap! {
                    "last_updated_at" => "{__now}",
                    "last_updated_by" => "{__author}",
                },
            },
            OperationType::Delete => OperationSpecific {
                corresponding_command: "limits remove",
                columns_to_set: indexmap::indexmap! {},
            },
        },
        post_actions: settings_wrap_postactions(vec![])
    }
});

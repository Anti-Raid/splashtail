use module_settings::{
    data_stores::PostgresDataStore,
    types::{
        settings_wrap_columns, settings_wrap_datastore, settings_wrap_postactions,
        settings_wrap_precheck, Column, ColumnSuggestion, ColumnType, ConfigOption,
        InnerColumnType, InnerColumnTypeStringKind, OperationSpecific, OperationType,
        SettingsError, ColumnAction
    },
};
use std::sync::LazyLock;
use strum::VariantNames;
use splashcore_rs::value::Value;
use futures_util::FutureExt;

pub static USER_STINGS: LazyLock<ConfigOption> = LazyLock::new(|| ConfigOption {
    id: "user_stings",
    name: "User Stings",
    description: "All stings users have recieved due to hitting limits",
    table: "limits__user_stings",
    common_filters: indexmap::indexmap! {},
    default_common_filters: indexmap::indexmap! {
        "guild_id" => "{__guild_id}"
    },
    primary_key: "id",
    max_entries: None,
    max_return: 20,
    data_store: settings_wrap_datastore(PostgresDataStore {}),
    columns: settings_wrap_columns(vec![
        Column {
            id: "id",
            name: "ID",
            description: "The unique identifier for the limit user sting.",
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
            id: "user_id",
            name: "User ID",
            description: "The User ID who has been stung by this limit",
            column_type: ColumnType::new_scalar(InnerColumnType::String {
                kind: InnerColumnTypeStringKind::User,
                min_length: None,
                max_length: None,
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
        module_settings::common_columns::guild_id(
            "guild_id",
            "Guild ID",
            "The Guild ID referring to this user sting",
        ),
        Column {
            id: "stings",
            name: "Stings",
            description: "The number of stings the user received",
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
            id: "hit_limits",
            name: "Hit Limits",
            description: "The limits the user hit",
            column_type: ColumnType::new_array(InnerColumnType::String {
                kind: InnerColumnTypeStringKind::Normal,
                min_length: None,
                max_length: Some(256),
                allowed_values: vec![],
            }),
            nullable: true,
            unique: false,
            suggestions: ColumnSuggestion::SettingsReference {
                module: "limits",
                setting: "guild_limits",
            },
            ignored_for: vec![],
            secret: false,
            pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
            default_pre_checks: settings_wrap_precheck(vec![]),
        },
        Column {
            id: "causes",
            name: "Causes",
            description: "A JSON object containing more context about the stings",
            column_type: ColumnType::new_scalar(InnerColumnType::Json {}),
            nullable: true,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
            pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
            default_pre_checks: settings_wrap_precheck(vec![]),
        },
        Column {
            id: "expiry",
            name: "Expiry",
            description: "When the stings expire",
            column_type: ColumnType::new_scalar(InnerColumnType::TimestampTz {}),
            nullable: false,
            unique: true,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![OperationType::Create],
            secret: false,
            pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
            default_pre_checks: settings_wrap_precheck(vec![]),
        },
        module_settings::common_columns::created_at(),
    ]),
    title_template: "{id} - {user_id} - {created_at}",
    operations: indexmap::indexmap! {
        OperationType::View => OperationSpecific {
            corresponding_command: "limits_user_stings view",
            columns_to_set: indexmap::indexmap! {},
        },
        OperationType::Delete => OperationSpecific {
            corresponding_command: "limits_user_stings remove",
            columns_to_set: indexmap::indexmap! {},
        },
    },
    post_actions: settings_wrap_postactions(vec![]),
});

pub static USER_ACTIONS: LazyLock<ConfigOption> = LazyLock::new(|| ConfigOption {
    id: "user_actions",
    name: "User Actions",
    description: "All actions that have been performed by a user",
    table: "limits__user_actions",
    common_filters: indexmap::indexmap! {},
    default_common_filters: indexmap::indexmap! {
        "guild_id" => "{__guild_id}"
    },
    primary_key: "action_id",
    max_entries: None,
    max_return: 20,
    data_store: settings_wrap_datastore(PostgresDataStore {}),
    columns: settings_wrap_columns(vec![
        Column {
            id: "action_id",
            name: "Action ID",
            description: "The unique identifier for the user action.",
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
            id: "user_id",
            name: "User ID",
            description: "The User ID who hit this limit",
            column_type: ColumnType::new_scalar(InnerColumnType::String {
                kind: InnerColumnTypeStringKind::User,
                min_length: None,
                max_length: None,
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
            description: "The limit this action contributed to",
            column_type: ColumnType::new_scalar(InnerColumnType::String {
                kind: InnerColumnTypeStringKind::Normal,
                min_length: None,
                max_length: None,
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
            id: "target",
            name: "Target",
            description: "The target of the action",
            column_type: ColumnType::new_scalar(InnerColumnType::String {
                kind: InnerColumnTypeStringKind::Normal,
                min_length: None,
                max_length: None,
                allowed_values: vec![],
            }),
            nullable: true,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
            pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
            default_pre_checks: settings_wrap_precheck(vec![]),
        },
        Column {
            id: "action_data",
            name: "Action Data",
            description: "Any extra data regarding the action",
            column_type: ColumnType::new_scalar(InnerColumnType::Json {}),
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
            description: "The number of stings the user received for this action",
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
            id: "stings_expiry",
            name: "Stings Expiry",
            description: "The number of stings the user received for this action",
            column_type: ColumnType::new_scalar(InnerColumnType::TimestampTz {}),
            nullable: false,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
            pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
            default_pre_checks: settings_wrap_precheck(vec![]),
        },
        module_settings::common_columns::guild_id(
            "guild_id",
            "Guild ID",
            "The Guild ID the hit limit belongs to",
        ),
        module_settings::common_columns::created_at(),
    ]),
    title_template: "{action_id} - {user_id} got {stings} stings",
    operations: indexmap::indexmap! {
        OperationType::View => OperationSpecific {
            corresponding_command: "limit_user_actions view",
            columns_to_set: indexmap::indexmap! {},
        },
        OperationType::Delete => OperationSpecific {
            corresponding_command: "limit_user_actions remove",
            columns_to_set: indexmap::indexmap! {},
        },
    },
    post_actions: settings_wrap_postactions(vec![]),
});

pub static GUILD_LIMITS: LazyLock<ConfigOption> = LazyLock::new(|| {
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
        max_entries: Some(10),
        max_return: 10,
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
            module_settings::common_columns::guild_id("guild_id", "Guild ID", "The Guild ID the limit belongs to"),
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
        post_actions: settings_wrap_postactions(vec![ColumnAction::NativeAction {
            action: Box::new(|_ctx, state| {
                async move {
                    let Some(Value::String(guild_id)) = state.state.get("guild_id") else {
                        return Err(SettingsError::MissingOrInvalidField {
                            field: "guild_id".to_string(),
                            src: "index->NativeAction [post_actions]".to_string(),
                        });
                    };
    
                    let guild_id = guild_id.parse::<serenity::all::GuildId>().map_err(|e| {
                        SettingsError::Generic {
                            message: format!("Error while parsing guild_id: {}", e),
                            typ: "value_error".to_string(),
                            src: "inspector__options.guild_id".to_string(),
                        }
                    })?;
    
                    super::cache::GUILD_LIMITS
                        .invalidate(&guild_id)
                        .await;
    
                    Ok(())
                }
                .boxed()
            }),
            on_condition: Some(|ctx, _state| Ok(ctx.operation_type != OperationType::View)),
        }]),    
    }
});

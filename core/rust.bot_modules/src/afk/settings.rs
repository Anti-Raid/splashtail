use futures_util::future::FutureExt;
use module_settings::data_stores::PostgresDataStore;
use module_settings::types::{
    settings_wrap_columns, settings_wrap_datastore, settings_wrap_postactions,
    settings_wrap_precheck, Column, ColumnAction, ColumnSuggestion, ColumnType, ConfigOption,
    InnerColumnType, InnerColumnTypeStringKind, OperationSpecific, OperationType, SettingsError,
};
use once_cell::sync::Lazy;
use splashcore_rs::value::Value;

pub const MAX_AFK: i64 = 60 * 60 * 24 * 30; // 30 days

pub static AFKS: Lazy<ConfigOption> = Lazy::new(|| {
    ConfigOption {
        id: "afks",
        name: "AFKs",
        description: "The list of all current AFK's",
        table: "afk__afks",
        common_filters: indexmap::indexmap! {},
        default_common_filters: indexmap::indexmap! {
            "guild_id" => "{__guild_id}"
        },
        primary_key: "id",
        max_entries: None,
        data_store: settings_wrap_datastore(PostgresDataStore {}),
        columns: settings_wrap_columns(vec![
            Column {
                id: "id",
                name: "AFK ID",
                description: "Unique identifier for the AFK",
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
                description: "The user id who made the AFK",
                column_type: ColumnType::new_scalar(InnerColumnType::String {
                    min_length: None,
                    max_length: None,
                    allowed_values: vec![],
                    kind: InnerColumnTypeStringKind::User,
                }),
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![OperationType::Create, OperationType::Update],
                secret: false,
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
                default_pre_checks: settings_wrap_precheck(vec![]),
            },
            module_settings::common_columns::guild_id(
                "guild_id",
                "Guild ID",
                "The Guild ID the afk belongs to",
            ),
            Column {
                id: "reason",
                name: "Reason",
                description: "The reason/explanation for the AFK",
                column_type: ColumnType::new_scalar(InnerColumnType::String {
                    min_length: None,
                    max_length: Some(128),
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
                id: "expires_at",
                name: "Expires At",
                description: "The time at which the AFK expires",
                column_type: ColumnType::new_scalar(InnerColumnType::TimestampTz {}),
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
                default_pre_checks: settings_wrap_precheck(vec![
                    // Set sink display type
                    ColumnAction::NativeAction {
                        action: Box::new(|ctx, state| {
                            async move {
                                match ctx.operation_type {
                                    OperationType::View | OperationType::Delete => return Ok(()),
                                    _ => {}
                                }

                                // Check that user_id is the author
                                let Some(Value::TimestampTz(expires_at)) =
                                    state.state.get("expires_at")
                                else {
                                    return Err(SettingsError::Generic {
                                        message: "User ID is required".to_string(),
                                        src: "NativeActions#expires_at".to_string(),
                                        typ: "external".to_string(),
                                    });
                                };

                                // Check the difference between the current time and the expires_at time
                                let curr_time_epoch = chrono::Utc::now().timestamp();
                                let expires_at_epoch = expires_at.timestamp();

                                let diff = if curr_time_epoch > expires_at_epoch {
                                    curr_time_epoch - expires_at_epoch
                                } else {
                                    expires_at_epoch - curr_time_epoch
                                };

                                if diff > MAX_AFK {
                                    return Err(SettingsError::Generic {
                                        message: format!(
                                            "AFK's can only be set for a maximum of {} days",
                                            MAX_AFK / (60 * 60 * 24)
                                        ),
                                        src: "NativeActions#expires_at".to_string(),
                                        typ: "external".to_string(),
                                    });
                                }

                                Ok(())
                            }
                            .boxed()
                        }),
                        on_condition: None,
                    },
                ]),
            },
            module_settings::common_columns::created_at(),
            module_settings::common_columns::created_by(),
            module_settings::common_columns::last_updated_at(),
            module_settings::common_columns::last_updated_by(),
        ]),
        title_template: "{created_at} - {expires_at}",
        operations: indexmap::indexmap! {
            OperationType::View => OperationSpecific {
                corresponding_command: "afk list",
                columns_to_set: indexmap::indexmap! {},
            },
            OperationType::Create => OperationSpecific {
                corresponding_command: "afk create",
                columns_to_set: indexmap::indexmap! {
                    "user_id" => "{__author}",
                    "created_at" => "{__now}",
                    "created_by" => "{__author}",
                    "last_updated_at" => "{__now}",
                    "last_updated_by" => "{__author}",
                },
            },
            OperationType::Update => OperationSpecific {
                corresponding_command: "afk update",
                columns_to_set: indexmap::indexmap! {
                    "last_updated_at" => "{__now}",
                    "last_updated_by" => "{__author}",
                },
            },
            OperationType::Delete => OperationSpecific {
                corresponding_command: "afk delete",
                columns_to_set: indexmap::indexmap! {},
            },
        },
        post_actions: settings_wrap_postactions(vec![]),
    }
});
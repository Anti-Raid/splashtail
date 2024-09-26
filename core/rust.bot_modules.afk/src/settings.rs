use module_settings::data_stores::PostgresDataStore;
use module_settings::state::State;
use module_settings::types::{
    settings_wrap, Column, ColumnSuggestion, ColumnType, ConfigOption, HookContext,
    InnerColumnType, InnerColumnTypeStringKind, NoOpPostAction, OperationSpecific, OperationType,
    SettingDataValidator, SettingsError,
};
use splashcore_rs::value::Value;
use std::sync::LazyLock;

pub const MAX_AFK: i64 = 60 * 60 * 24 * 30; // 30 days

pub static AFKS: LazyLock<ConfigOption> = LazyLock::new(|| ConfigOption {
    id: "afks",
    name: "AFKs",
    description: "The list of all current AFK's",
    table: "afk__afks",
    common_filters: indexmap::indexmap! {},
    default_common_filters: indexmap::indexmap! {
        "guild_id" => "{__guild_id}"
    },
    primary_key: "id",
    max_return: 15,
    max_entries: None,
    data_store: settings_wrap(PostgresDataStore {}),
    columns: settings_wrap(vec![
        Column {
            id: "id",
            name: "AFK ID",
            description: "Unique identifier for the AFK",
            column_type: ColumnType::new_scalar(InnerColumnType::Uuid {}),
            nullable: false,
            default: None,
            unique: true,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![OperationType::Create],
            secret: false,
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
            default: None,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![OperationType::Create, OperationType::Update],
            secret: false,
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
            default: None,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
        },
        Column {
            id: "expires_at",
            name: "Expires At",
            description: "The time at which the AFK expires",
            column_type: ColumnType::new_scalar(InnerColumnType::TimestampTz {}),
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
    post_action: settings_wrap(NoOpPostAction {}),
    validator: settings_wrap(AfksValidator {}),
});

/// AFK's need special validation to protect against invalid AFK's
pub struct AfksValidator;

#[async_trait::async_trait]
impl SettingDataValidator for AfksValidator {
    async fn validate<'a>(
        &self,
        ctx: HookContext<'a>,
        state: &'a mut State,
    ) -> Result<(), SettingsError> {
        match ctx.operation_type {
            OperationType::View | OperationType::Delete => return Ok(()),
            _ => {}
        }

        // Check that user_id is the author
        let Some(Value::TimestampTz(expires_at)) = state.state.get("expires_at") else {
            return Err(SettingsError::Generic {
                message: "User ID is required".to_string(),
                src: "AfksValidator".to_string(),
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
                src: "AfksValidator".to_string(),
                typ: "external".to_string(),
            });
        }

        Ok(())
    }
}

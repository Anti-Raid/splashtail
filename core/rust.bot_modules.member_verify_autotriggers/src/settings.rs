use module_settings::{
    data_stores::PostgresDataStore,
    state::State,
    types::{
        settings_wrap, Column, ColumnSuggestion, ColumnType, ConfigOption, HookContext,
        InnerColumnType, InnerColumnTypeStringKind, NoOpPostAction, OperationSpecific,
        OperationType, SettingDataValidator, SettingsError,
    },
};
use std::sync::LazyLock;

pub static AUTOTRIGGERS: LazyLock<ConfigOption> = LazyLock::new(|| ConfigOption {
    id: "member_verify_autotriggers",
    name: "Member Verify Autotriggers",
    description: "What should happen when a member is verified",
    table: "member_verify_autotriggers__trigger",
    common_filters: indexmap::indexmap! {},
    default_common_filters: indexmap::indexmap! {
        "guild_id" => "{__guild_id}"
    },
    primary_key: "guild_id",
    max_entries: None,
    max_return: 20,
    data_store: settings_wrap(PostgresDataStore {}),
    columns: settings_wrap(vec![
        module_settings::common_columns::guild_id(
            "guild_id",
            "Guild ID",
            "Guild ID of the server in question",
        ),
        module_settings::common_columns::created_by(),
        Column {
            id: "give_roles",
            name: "Give Roles",
            description: "The number of stings required to trigger the action",
            column_type: ColumnType::new_array(InnerColumnType::String {
                min_length: Some(1),
                max_length: Some(64),
                allowed_values: vec![],
                kind: InnerColumnTypeStringKind::Role,
            }),
            nullable: false,
            default: None,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
        },
        Column {
            id: "remove_roles",
            name: "Remove Roles",
            description: "The number of stings required to trigger the action",
            column_type: ColumnType::new_array(InnerColumnType::String {
                min_length: Some(1),
                max_length: Some(64),
                allowed_values: vec![],
                kind: InnerColumnTypeStringKind::Role,
            }),
            nullable: false,
            default: None,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
        },
    ]),
    title_template: "At {stings} stings, {action} will be triggered",
    operations: indexmap::indexmap! {
        OperationType::View => OperationSpecific {
            columns_to_set: indexmap::indexmap! {},
        },
        OperationType::Create => OperationSpecific {
            columns_to_set: indexmap::indexmap! {
                "created_at" => "{__now}",
                "created_by" => "{__author}",
            },
        },
        OperationType::Update => OperationSpecific {
            columns_to_set: indexmap::indexmap! {},
        },
        OperationType::Delete => OperationSpecific {
            columns_to_set: indexmap::indexmap! {},
        },
    },
    validator: settings_wrap(AutotriggerValidator {}),
    post_action: settings_wrap(NoOpPostAction {}),
});

pub struct AutotriggerValidator;

#[async_trait::async_trait]
impl SettingDataValidator for AutotriggerValidator {
    async fn validate<'a>(
        &self,
        ctx: HookContext<'a>,
        state: &'a mut State,
    ) -> Result<(), SettingsError> {
        if ctx.operation_type == OperationType::View {
            return Ok(()); // No need to validate view operations
        }

        let mut total_count = 0;

        let Some(splashcore_rs::value::Value::List(ref give_roles)) = state.state.get("give_roles")
        else {
            return Err(SettingsError::Generic {
                message: "Action is required".to_string(),
                src: "AutotriggerValidator".to_string(),
                typ: "external".to_string(),
            });
        };

        total_count += give_roles.len();

        let Some(splashcore_rs::value::Value::List(ref remove_roles)) =
            state.state.get("remove_roles")
        else {
            return Err(SettingsError::Generic {
                message: "Action is required".to_string(),
                src: "AutotriggerValidator".to_string(),
                typ: "external".to_string(),
            });
        };

        total_count += remove_roles.len();

        if total_count == 0 {
            return Err(SettingsError::Generic {
                message: "At least one role must be given or removed".to_string(),
                src: "AutotriggerValidator".to_string(),
                typ: "external".to_string(),
            });
        }

        if total_count > 3 {
            return Err(SettingsError::Generic {
                message: "A maximum of 3 roles can be given or removed at this time".to_string(),
                src: "AutotriggerValidator".to_string(),
                typ: "external".to_string(),
            });
        }

        Ok(())
    }
}

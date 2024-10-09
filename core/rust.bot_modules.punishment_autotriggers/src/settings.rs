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
    id: "punishment_autotriggers",
    name: "Punishment Autotriggers",
    description: "All punishments that should be trigggred automatically based on stings",
    table: "punishment_autotriggers__autotriggers",
    common_filters: indexmap::indexmap! {},
    default_common_filters: indexmap::indexmap! {
        "guild_id" => "{__guild_id}"
    },
    primary_key: "id",
    max_entries: None,
    max_return: 20,
    data_store: settings_wrap(PostgresDataStore {}),
    columns: settings_wrap(vec![
        Column {
            id: "id",
            name: "ID",
            description: "The ID used to refer to this autotrigger",
            column_type: ColumnType::new_scalar(InnerColumnType::Uuid {}),
            nullable: true,
            default: None,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![OperationType::Create],
            secret: false,
        },
        module_settings::common_columns::guild_id(
            "guild_id",
            "Guild ID",
            "Guild ID of the server in question",
        ),
        module_settings::common_columns::created_by(),
        Column {
            id: "stings",
            name: "Stings",
            description: "The number of stings required to trigger the action",
            column_type: ColumnType::new_scalar(InnerColumnType::Integer {}),
            nullable: false,
            default: None,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
        },
        Column {
            id: "action",
            name: "Action",
            description: "The action to trigger when the stings are reached",
            column_type: ColumnType::new_scalar(InnerColumnType::String {
                min_length: Some(1),
                max_length: Some(100),
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
            id: "modifiers",
            name: "Modifiers",
            description: "Any modifiers to the action",
            column_type: ColumnType::new_array(InnerColumnType::String {
                min_length: Some(1),
                max_length: Some(100),
                allowed_values: vec![],
                kind: InnerColumnTypeStringKind::Modifier,
            }),
            nullable: false,
            default: None,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
        },
        module_settings::common_columns::created_at(),
        Column {
            id: "duration",
            name: "Duration",
            description: "The duration of the punishment to apply/use",
            column_type: ColumnType::new_scalar(InnerColumnType::Interval {}),
            nullable: true,
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

        let Some(splashcore_rs::value::Value::String(ref action)) = state.state.get("action")
        else {
            return Err(SettingsError::Generic {
                message: "Action is required".to_string(),
                src: "AutotriggerValidator".to_string(),
                typ: "external".to_string(),
            });
        };

        let actions_map = silverpelt::punishments::get_punishment_actions_for_guild(
            ctx.guild_id,
            &ctx.data.serenity_context.data::<silverpelt::data::Data>(),
        )
        .await
        .map_err(|e| SettingsError::Generic {
            message: format!("Error getting punishment actions: {}", e),
            src: "AutotriggerValidator".to_string(),
            typ: "external".to_string(),
        })?;

        match silverpelt::punishments::from_punishment_action_string(&actions_map, action) {
            Ok(_) => {}
            Err(e) => {
                return Err(SettingsError::Generic {
                    message: format!("Invalid action: {}", e),
                    src: "AutotriggerValidator".to_string(),
                    typ: "external".to_string(),
                });
            }
        }

        Ok(())
    }
}

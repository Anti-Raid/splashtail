use ar_settings::{
    data_stores::PostgresDataStore,
    state::State,
    types::{
        settings_wrap, Column, ColumnSuggestion, ColumnType, HookContext, InnerColumnType,
        InnerColumnTypeStringKind, NoOpPostAction, OperationSpecific, OperationType, Setting,
        SettingDataValidator, SettingsError,
    },
};
use std::sync::LazyLock;

pub static AUTOTRIGGERS: LazyLock<Setting> = LazyLock::new(|| Setting {
    id: "punishment_autotriggers".to_string(),
    name: "Punishment Autotriggers".to_string(),
    description: "All punishments that should be trigggred automatically based on stings"
        .to_string(),
    primary_key: "id".to_string(),
    columns: settings_wrap(vec![
        Column {
            id: "id".to_string(),
            name: "ID".to_string(),
            description: "The ID used to refer to this autotrigger".to_string(),
            column_type: ColumnType::new_scalar(InnerColumnType::Uuid {}),
            nullable: true,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![OperationType::Create],
            secret: false,
        },
        ar_settings::common_columns::guild_id(
            "guild_id",
            "Guild ID",
            "Guild ID of the server in question",
        ),
        ar_settings::common_columns::created_by(),
        Column {
            id: "stings".to_string(),
            name: "Stings".to_string(),
            description: "The number of stings required to trigger the action".to_string(),
            column_type: ColumnType::new_scalar(InnerColumnType::Integer {}),
            nullable: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
        },
        Column {
            id: "action".to_string(),
            name: "Action".to_string(),
            description: "The action to trigger when the stings are reached".to_string(),
            column_type: ColumnType::new_scalar(InnerColumnType::String {
                min_length: Some(1),
                max_length: Some(100),
                allowed_values: vec![],
                kind: InnerColumnTypeStringKind::Normal,
            }),
            nullable: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
        },
        Column {
            id: "modifiers".to_string(),
            name: "Modifiers".to_string(),
            description: "Any modifiers to the action".to_string(),
            column_type: ColumnType::new_array(InnerColumnType::String {
                min_length: Some(1),
                max_length: Some(100),
                allowed_values: vec![],
                kind: InnerColumnTypeStringKind::Modifier,
            }),
            nullable: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
        },
        ar_settings::common_columns::created_at(),
        Column {
            id: "duration".to_string(),
            name: "Duration".to_string(),
            description: "The duration of the punishment to apply/use".to_string(),
            column_type: ColumnType::new_scalar(InnerColumnType::Interval {}),
            nullable: true,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
        },
    ]),
    title_template: "At {stings} stings, {action} will be triggered".to_string(),
    supported_operations: vec![
        OperationType::View,
        OperationType::Create,
        OperationType::Update,
        OperationType::Delete,
    ],
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

        if action.is_empty() {
            return Err(SettingsError::Generic {
                message: "Action cannot be empty".to_string(),
                src: "AutotriggerValidator".to_string(),
                typ: "external".to_string(),
            });
        }

        if !["ban", "kick", "timeout", "removeallroles"].contains(&action.as_str()) {
            return Err(SettingsError::Generic {
                message: "Action must be one of ban, kick, timeout and removeallroles".to_string(),
                src: "AutotriggerValidator".to_string(),
                typ: "external".to_string(),
            });
        }

        Ok(())
    }
}

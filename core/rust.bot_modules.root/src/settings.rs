use module_settings::{
    data_stores::PostgresDataStore,
    state::State,
    types::{
        settings_wrap, Column, ColumnSuggestion, ColumnType, ConfigOption, HookContext,
        InnerColumnType, InnerColumnTypeStringKind, NoOpValidator, OperationSpecific,
        OperationType, PostAction, SettingsError,
    },
};
use std::sync::LazyLock;

pub static INSPECTOR_FAKE_BOTS: LazyLock<ConfigOption> = LazyLock::new(|| ConfigOption {
    id: "sudo__ifb",
    name: "Inspector (Fake Bots)",
    description: "Stores a list of official bots and their ID to allow detection of fake bots",
    table: "inspector__fake_bots",
    common_filters: indexmap::indexmap! {},
    default_common_filters: indexmap::indexmap! {},
    primary_key: "bot_id",
    max_entries: None,
    max_return: 15,
    data_store: settings_wrap(PostgresDataStore {}),
    columns: settings_wrap(vec![
        Column {
            id: "bot_id",
            name: "Bot ID",
            description: "The ID of the bot",
            column_type: ColumnType::new_scalar(InnerColumnType::String {
                min_length: None,
                max_length: Some(32),
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
            id: "name",
            name: "Bot Name",
            description: "The name of the bot",
            column_type: ColumnType::new_scalar(InnerColumnType::String {
                min_length: None,
                max_length: Some(512),
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
            id: "official_bot_ids",
            name: "Official Bot IDs",
            description: "The discriminator of the bot",
            column_type: ColumnType::new_array(InnerColumnType::String {
                min_length: None,
                max_length: Some(32),
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
        module_settings::common_columns::created_at(),
        module_settings::common_columns::created_by(),
        module_settings::common_columns::last_updated_at(),
        module_settings::common_columns::last_updated_by(),
        Column {
            id: "comments",
            name: "Comments",
            description: "Comments about the entry",
            column_type: ColumnType::new_scalar(InnerColumnType::String {
                min_length: None,
                max_length: Some(1024),
                allowed_values: vec![],
                kind: InnerColumnTypeStringKind::Normal,
            }),
            nullable: true,
            default: None,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
        },
        /*Column {
            id: "test",
            name: "Test",
            description: "Test bitflag",
            column_type: ColumnType::new_scalar(InnerColumnType::BitFlag {
                values: indexmap::indexmap! {
                    "A".to_string() => 1 << 0,
                    "ABC".to_string() => 1 << 1,
                    "DEF".to_string() => 1 << 2,
                },
            }),
            nullable: true,
            default: None,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![OperationType::Create],
            secret: false,
        },
        Column {
            id: "test2",
            name: "Test2",
            description: "Test bitflag 2",
            column_type: ColumnType::new_scalar(InnerColumnType::BitFlag {
                values: indexmap::indexmap! {
                    "ABSINTHE".to_string() => 1 << 0,
                    "GOA".to_string() => 1 << 1,
                    "LUA".to_string() => 1 << 2,
                },
            }),
            nullable: true,
            default: None,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![OperationType::Create],
            secret: false,
        },*/
    ]),
    title_template: "{name} - {bot_id}",
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
    post_action: settings_wrap(InspectorFakeBotsPostAction {}),
    validator: settings_wrap(NoOpValidator {}),
});

/// Inspector Fake Bots Post Action
pub struct InspectorFakeBotsPostAction;

#[async_trait::async_trait]
impl PostAction for InspectorFakeBotsPostAction {
    async fn post_action<'a>(
        &self,
        ctx: HookContext<'a>,
        _state: &'a mut State,
    ) -> Result<(), SettingsError> {
        if ctx.operation_type == OperationType::View {
            return Ok(());
        }

        let data = silverpelt::data::Data::data(ctx.data);

        silverpelt::ar_event::dispatch_event_to_modules_errflatten(std::sync::Arc::new(
            silverpelt::ar_event::EventHandlerContext {
                guild_id: ctx.guild_id,
                data,
                event: silverpelt::ar_event::AntiraidEvent::TrustedWebEvent((
                    "inspector.resetFakeBotsCache".to_string(),
                    serde_json::Value::Null,
                )),
                serenity_context: ctx.data.serenity_context.clone(),
            },
        ))
        .await
        .map_err(|e| SettingsError::Generic {
            message: format!("Failed to dispatch event: {:#?}", e),
            typ: "InspectorFakeBotsPostAction".to_string(),
            src: "internal".to_string(),
        })?;

        Ok(())
    }
}

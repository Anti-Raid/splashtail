use module_settings::data_stores::PostgresDataStore;
use module_settings::state::State;
use module_settings::types::{
    settings_wrap, Column, ColumnSuggestion, ColumnType, ConfigOption, HookContext,
    InnerColumnType, InnerColumnTypeStringKind, NoOpValidator, OperationSpecific, OperationType,
    PostAction, SettingsError,
};
use serenity::all::{ChannelType, Permissions};
use splashcore_rs::value::Value;
use std::sync::LazyLock;

pub static SINK: LazyLock<ConfigOption> = LazyLock::new(|| {
    ConfigOption {
        id: "auditlog_sinks",
        name: "Audit Log Sinks",
        description: "A sink is a place where audit logs are sent to. This can be a channel or a webhook at this time. More sink types may be added in the future.",
        table: "auditlogs__sinks",
        common_filters: indexmap::indexmap! {},
        default_common_filters: indexmap::indexmap! {
            "guild_id" => "{__guild_id}"
        },
        primary_key: "id",
        max_return: 15,
        max_entries: Some(10),
        data_store: settings_wrap(PostgresDataStore {}),
        columns: settings_wrap(vec![
            Column {
                id: "id",
                name: "Sink ID",
                description: "The unique identifier for the sink.",
                column_type: ColumnType::new_scalar(InnerColumnType::Uuid {}),
                nullable: false,
                default: None,
                unique: true,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![OperationType::Create],
                secret: false,
            },
            module_settings::common_columns::guild_id("guild_id", "Guild ID", "The Guild ID the sink belongs to"),
            Column {
                id: "sink",
                name: "Sink",
                description: "The sink where the logs are sent to if returned by template",
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::Channel {
                    allowed_types: vec![ChannelType::Text, ChannelType::Voice, ChannelType::PublicThread, ChannelType::PrivateThread, ChannelType::News],
                    needed_bot_permissions: Permissions::VIEW_CHANNEL | Permissions::SEND_MESSAGES | Permissions::EMBED_LINKS,
                } }),
                nullable: false,
                default: None,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
            },
            Column {
                id: "events",
                name: "Events",
                description: "The events that are sent to the sink. If empty, all events are sent. Prefix with R/ for regex (rust style regex) matching.",
                column_type: ColumnType::new_array(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::Normal }),
                nullable: true,
                default: None,
                unique: false,
                suggestions: ColumnSuggestion::Static { suggestions: gwevent::core::event_list().to_vec() },
                ignored_for: vec![],
                secret: false,
            },
            Column {
                id: "embed_template",
                name: "Template",
                description: "The custom template for the embed. This template will be executed when an event is sent to the sink. If empty, falls back to default handling",
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::Template { kind: "message", ctx: "AuditLogContext" }}),
                ignored_for: vec![],
                secret: false,
                nullable: true,
                default: None,
                unique: false,
                suggestions: ColumnSuggestion::None {},
            },
            Column {
                id: "broken",
                name: "Marked as Broken",
                description: "If the sink is marked as broken, it will not be used for sending logs. This can be useful in debugging too!",
                column_type: ColumnType::new_scalar(InnerColumnType::Boolean {}),
                ignored_for: vec![OperationType::Create],
                secret: false,
                nullable: false,
                default: Some(|_| Value::Boolean(false)),
                unique: false,
                suggestions: ColumnSuggestion::None {},
            },
            module_settings::common_columns::created_at(),
            module_settings::common_columns::created_by(),
            module_settings::common_columns::last_updated_at(),
            module_settings::common_columns::last_updated_by(),
        ]),
        title_template: "{type} {sink} [{id}]",
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
        validator: settings_wrap(NoOpValidator {}),
        post_action: settings_wrap(ClearCachePostAction {}),
    }
});

/// Post action to clear the cache
pub struct ClearCachePostAction;

#[async_trait::async_trait]
impl PostAction for ClearCachePostAction {
    async fn post_action<'a>(
        &self,
        ctx: HookContext<'a>,
        _state: &'a mut State,
    ) -> Result<(), SettingsError> {
        if ctx.operation_type == OperationType::View {
            return Ok(());
        }

        super::cache::SINKS_CACHE.invalidate(&ctx.guild_id).await;

        Ok(())
    }
}

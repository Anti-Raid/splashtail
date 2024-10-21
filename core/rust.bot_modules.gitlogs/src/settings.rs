use module_settings::types::{
    settings_wrap, Column, ColumnSuggestion, ColumnType, ConfigOption, InnerColumnType, InnerColumnTypeStringKind, OperationSpecific, OperationType, SettingsError,
    SettingDataValidator, NoOpPostAction, HookContext
};
use module_settings::state::State;
use module_settings::data_stores::PostgresDataStore;
use serenity::all::{Permissions, ChannelType};
use splashcore_rs::value::Value;
use std::sync::LazyLock;

pub static WEBHOOKS: LazyLock<ConfigOption> = LazyLock::new(|| {
    ConfigOption {
        id: "gitlogs__webhooks",
        name: "Webhooks",
        description:
            "Stores a list of webhooks to which Github can post events to.",
        table: "gitlogs__webhooks",
        common_filters: indexmap::indexmap! {},
        default_common_filters: indexmap::indexmap! {
            "guild_id" => "{__guild_id}"
        },
        primary_key: "id",
        max_entries: Some(5),
        max_return: 7,
        data_store: settings_wrap(PostgresDataStore {}),
        columns: settings_wrap(vec![
            Column {
                id: "id",
                name: "Webhook ID",
                description: "Unique identifier for the webhook",
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::Normal }),
                nullable: false,
                default: None,
                unique: true,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![OperationType::Create],
                secret: false,
            },
            module_settings::common_columns::guild_id("guild_id", "Guild ID", "The Guild ID the webhook belongs to"),
            Column {
                id: "comment",
                name: "Comment",
                description: "A comment to describe the webhook. Not used for any purpose beyond documentation.",
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: Some(64), allowed_values: vec![], kind: InnerColumnTypeStringKind::Normal }),
                nullable: false,
                default: None,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
            },
            Column {
                id: "secret",
                name: "Secret",
                description: "A secret to verify the authenticity of the webhook. This is used to ensure that the webhook is from Github and not a malicious actor.",
                column_type: ColumnType::new_scalar(
                    InnerColumnType::String { 
                        min_length: None, 
                        max_length: Some(256), 
                        allowed_values: vec![], 
                        kind: InnerColumnTypeStringKind::Token { 
                            default_length: 256
                        } 
                    }
                ),
                nullable: false,
                default: None,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: true,
            },
            module_settings::common_columns::created_at(),
            module_settings::common_columns::created_by(),
            module_settings::common_columns::last_updated_at(),
            module_settings::common_columns::last_updated_by(),
        ]),
        title_template: "{comment} - {id}",
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
        validator: settings_wrap(WebhookValidator {}),
        post_action: settings_wrap(NoOpPostAction {}),
    }
});

/// Webhook validator
pub struct WebhookValidator;

#[async_trait::async_trait]
impl SettingDataValidator for WebhookValidator {
    async fn validate<'a>(
        &self,
        ctx: HookContext<'a>,
        state: &'a mut State,
    ) -> Result<(), SettingsError> {
        // Ignore for View
        if ctx.operation_type == OperationType::View {
            return Ok(());
        }

        if ctx.operation_type == OperationType::Create {
            // ID Fixup on create
            let id = botox::crypto::gen_random(128);
            state.state.insert("id".to_string(), Value::String(id.to_string()));
            state.bypass_ignore_for.insert("id".to_string());

            // Secret fixup
            let Some(Value::String(secret)) = state.state.get("secret") else {
                return Err(SettingsError::MissingOrInvalidField { 
                    field: "secret".to_string(),
                    src: "secret->NativeActon [pre_checks]".to_string(),
                });
            };

            let Some(Value::String(id)) = state.state.get("id") else {
                return Err(SettingsError::MissingOrInvalidField { 
                    field: "id".to_string(),
                    src: "id->NativeActon [pre_checks]".to_string(),
                });
            };

            // Insert message
            state.state.insert(
                "__message".to_string(), 
                Value::String(format!(
                    "
Next, add the following webhook to your Github repositories (or organizations): `{api_url}/integrations/gitlogs/kittycat?id={id}`

Set the `Secret` field to `{webh_secret}` and ensure that Content Type is set to `application/json`. 

When creating repositories, use `{id}` as the ID.

**Note that the above URL and secret is unique and should not be shared with others**
                    ",
                    api_url=config::CONFIG.sites.api,
                    id=id,
                    webh_secret=secret
                )
            ));
        }

        Ok(())
    }
}

pub static REPOS: LazyLock<ConfigOption> = LazyLock::new(|| {
    ConfigOption {
        id: "gitlogs__repos",
        name: "Repositories",
        description:
            "In order for the Git Logs integration to post webhooks, you must provide a list of repositories",
        table: "gitlogs__repos",
        common_filters: indexmap::indexmap! {},
        default_common_filters: indexmap::indexmap! {
            "guild_id" => "{__guild_id}"
        },
        primary_key: "id",
        max_entries: Some(10),
        max_return: 12,
        data_store: settings_wrap(PostgresDataStore {}),
        columns: settings_wrap(vec![
            Column {
                id: "id",
                name: "Repo ID",
                description: "Unique identifier for the repository",
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::Normal }),
                nullable: false,
                default: None,
                unique: true,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![OperationType::Create],
                secret: false,
            },
            Column {
                id: "webhook_id",
                name: "Webhook ID",
                description: "The webhook to which the repository will post events to.",
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::Normal }),
                nullable: false,
                default: None,
                unique: false,
                suggestions: ColumnSuggestion::SettingsReference {
                    module: "gitlogs",
                    setting: "webhooks",
                },
                ignored_for: vec![],
                secret: false,
            },
            module_settings::common_columns::guild_id("guild_id", "Guild ID", "The Guild ID the repository belongs to"),
            Column {
                id: "repo_name",
                name: "Repository Name [format: org/repo]",
                description: "The name of the repository in the format of org/repo.\n\n**Example**: Anti-Raid/splashtail",
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::Normal }),
                nullable: false,
                default: None,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
            },
            Column {
                id: "channel_id",
                name: "Channel ID",
                description: "The channel to which the repository will post events to.",
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: Some(64), allowed_values: vec![], kind: InnerColumnTypeStringKind::Channel {
                    allowed_types: vec![ChannelType::Text, ChannelType::Voice, ChannelType::PublicThread, ChannelType::PrivateThread, ChannelType::News],
                    needed_bot_permissions: Permissions::VIEW_CHANNEL | Permissions::SEND_MESSAGES | Permissions::EMBED_LINKS | Permissions::READ_MESSAGE_HISTORY,
                } }),
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
        title_template: "{repo_name} - {id}",
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
        validator: settings_wrap(RepoValidator {}),
        post_action: settings_wrap(NoOpPostAction {}),
    }
});

/// Repo validator
pub struct RepoValidator;

#[async_trait::async_trait]
impl SettingDataValidator for RepoValidator {
    async fn validate<'a>(
        &self,
        ctx: HookContext<'a>,
        state: &'a mut State,
    ) -> Result<(), SettingsError> {
        // Ignore for View
        if ctx.operation_type == OperationType::View {
            return Ok(());
        }

        if ctx.operation_type == OperationType::Create {
                // ID fixup on create
                let id = botox::crypto::gen_random(32);
                state.state.insert("id".to_string(), Value::String(id.to_string()));
                state.bypass_ignore_for.insert("id".to_string());
        }
        
        // Check for webhook
        let Some(Value::String(webhook_id)) = state.state.get("webhook_id") else {
            return Err(SettingsError::MissingOrInvalidField { 
                field: "webhook_id".to_string(),
                src: "webhook_id->NativeAction [default_pre_checks]".to_string(),
            });
        };
        
        // Check if the webhook exists
        let webhook = sqlx::query!(
            "SELECT COUNT(1) FROM gitlogs__webhooks WHERE id = $1 AND guild_id = $2",
            webhook_id,
            ctx.guild_id.to_string()
        )
        .fetch_one(&ctx.data.pool)
        .await
        .map_err(|e| SettingsError::Generic { 
            message: e.to_string(),
            src: "webhook_id->NativeAction [default_pre_checks]".to_string(),
            typ: "database error".to_string(),
        })?;

        if webhook.count.unwrap_or_default() == 0 {
            return Err(SettingsError::SchemaCheckValidationError { 
                column: "webhook_id".to_string(),
                check: "webhook_id->NativeAction [default_pre_checks]".to_string(),
                error: "The specified webhook doesn't exist!".to_string(),
                accepted_range: "Valid webhook ID".to_string(),
            });
        }

        // Check if repo exists
        if ctx.operation_type == OperationType::Create {
            if let Some(Value::String(repo_name)) = state.state.get("repo_name") {
                let split = repo_name.split('/').collect::<Vec<&str>>();

                if split.len() != 2 {
                    return Err(SettingsError::SchemaCheckValidationError { 
                        column: "repo_name".to_string(),
                        check: "repo_name->NativeAction [default_pre_checks]".to_string(),
                        error: "Repository name must be in the format org/repo".to_string(),
                        accepted_range: "Valid repository name".to_string(),
                    });
                }
                
                // Check if the repo exists
                let repo = sqlx::query!(
                    "SELECT COUNT(1) FROM gitlogs__repos WHERE lower(repo_name) = $1 AND guild_id = $2 AND webhook_id = $3",
                    repo_name,
                    webhook_id,
                    ctx.guild_id.to_string()
                )
                .fetch_one(&ctx.data.pool)
                .await
                .map_err(|e| SettingsError::Generic { 
                    message: e.to_string(),
                    src: "repo_id->NativeAction [default_pre_checks]".to_string(),
                    typ: "database error".to_string(),
                })?;

                if repo.count.unwrap_or_default() > 0 {
                    return Err(SettingsError::SchemaCheckValidationError { 
                        column: "repo_id".to_string(),
                        check: "repo_id->NativeAction [default_pre_checks]".to_string(),
                        error: "The specified repository already exists".to_string(),
                        accepted_range: "Valid repository ID".to_string(),
                    });
                }
            }
        }

        Ok(())
    }
}

pub static EVENT_MODIFIERS: LazyLock<ConfigOption> = LazyLock::new(|| {
    ConfigOption {
        id: "gitlogs__event_modifiers",
        name: "Event Modifiers",
        description:
            "An event modifier allows customizing and redirecting webhooks based on the event type.",
        table: "gitlogs__event_modifiers",
        common_filters: indexmap::indexmap! {},
        default_common_filters: indexmap::indexmap! {
            "guild_id" => "{__guild_id}"
        },
        primary_key: "id",
        max_entries: Some(50),
        max_return: 20,
        data_store: settings_wrap(PostgresDataStore {}),
        columns: settings_wrap(vec![
            Column {
                id: "id",
                name: "Modifier ID",
                description: "Unique identifier for the event modifier",
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::Normal }),
                nullable: false,
                default: None,
                unique: true,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![OperationType::Create],
                secret: false,
            },
            Column {
                id: "webhook_id",
                name: "Webhook ID",
                description: "The webhook to which the repository will post events to.",
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::Normal }),
                nullable: false,
                default: None,
                unique: false,
                suggestions: ColumnSuggestion::SettingsReference {
                    module: "gitlogs",
                    setting: "webhooks",
                },
                ignored_for: vec![],
                secret: false,
            },
            Column {
                id: "repo_id",
                name: "Repo ID",
                description: "The repository to which the modifier will apply. If not set, the modifier will apply to all repositories.",
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::Normal }),
                nullable: true,
                default: None,
                unique: false,
                suggestions: ColumnSuggestion::SettingsReference {
                    module: "gitlogs",
                    setting: "repos",
                },
                ignored_for: vec![],
                secret: false,
            },
            module_settings::common_columns::guild_id("guild_id", "Guild ID", "The Guild ID the event modifier belongs to"),
            Column {
                id: "events",
                name: "Events",
                description: "The events to which the modifier will apply. If not set, the modifier will apply to all events.",
                column_type: ColumnType::new_array(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::Normal }),
                nullable: false,
                default: None,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
            },
            Column {
                id: "blacklisted",
                name: "Blacklisted",
                description: "If set to true, the modifier will block the event from being posted to the webhook.",
                column_type: ColumnType::new_scalar(InnerColumnType::Boolean {}),
                nullable: false,
                default: Some(|_| Value::Boolean(false)),
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
            },
            Column {
                id: "whitelisted",
                name: "Whitelisted [Other events will not be allowed]",
                description: "If set to true, the modifier will only allow the specified events to be posted to the webhook.",
                column_type: ColumnType::new_scalar(InnerColumnType::Boolean {}),
                nullable: false,
                default: Some(|_| Value::Boolean(false)),
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
            },
            Column {
                id: "redirect_channel",
                name: "Redirect Channel",
                description: "If set, the modifier will redirect the events to the specified channel.",
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::Channel {
                    allowed_types: vec![ChannelType::Text, ChannelType::Voice, ChannelType::PublicThread, ChannelType::PrivateThread, ChannelType::News],
                    needed_bot_permissions: Permissions::VIEW_CHANNEL | Permissions::SEND_MESSAGES | Permissions::EMBED_LINKS | Permissions::READ_MESSAGE_HISTORY,
                } }),
                nullable: true,
                default: None,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
            },
            Column {
                id: "priority",
                name: "Priority",
                description: "The priority of the modifier. The modifier with the highest priority will be applied first.",
                column_type: ColumnType::new_scalar(InnerColumnType::Integer {}),
                nullable: false,
                default: Some(|_| Value::Integer(0)),
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
        title_template: "{id}",
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
        validator: settings_wrap(EventModifierValidator {}),
        post_action: settings_wrap(NoOpPostAction {}),
    }
});

/// Event Modifier validator
pub struct EventModifierValidator;

#[async_trait::async_trait]
impl SettingDataValidator for EventModifierValidator {
    async fn validate<'a>(
        &self,
        ctx: HookContext<'a>,
        state: &'a mut State,
    ) -> Result<(), SettingsError> {
        // Ignore for View
        if ctx.operation_type == OperationType::View {
            return Ok(());
        }  

        // Before doing anything else, check the priority field to allow for an early exit if the priority is invalid
        if let Some(Value::Integer(priority)) = state.state.get("priority") {
            if *priority < 0 {
                return Err(SettingsError::SchemaCheckValidationError { 
                    column: "priority".to_string(),
                    check: "priority->NativeAction [default_pre_checks]".to_string(),
                    error: "Priority must be greater than or equal to 0".to_string(),
                    accepted_range: "Priority >= 0".to_string(),
                });
            }
        }

        if ctx.operation_type == OperationType::Create {
            // ID fixup on create
            let id = botox::crypto::gen_random(256);
            state.state.insert("id".to_string(), Value::String(id.to_string()));
            state.bypass_ignore_for.insert("id".to_string());            
        }

        let Some(Value::String(webhook_id)) = state.state.get("webhook_id") else {
            return Err(SettingsError::MissingOrInvalidField { 
                field: "webhook_id".to_string(),
                src: "webhook_id->NativeAction [default_pre_checks]".to_string(),
            });
        };
        
        // Check if the webhook exists
        let webhook = sqlx::query!(
            "SELECT COUNT(1) FROM gitlogs__webhooks WHERE id = $1 AND guild_id = $2",
            webhook_id,
            ctx.guild_id.to_string()
        )
        .fetch_one(&ctx.data.pool)
        .await
        .map_err(|e| SettingsError::Generic { 
            message: e.to_string(),
            src: "webhook_id->NativeAction [default_pre_checks]".to_string(),
            typ: "database error".to_string(),
        })?;

        if webhook.count.unwrap_or_default() == 0 {
            return Err(SettingsError::SchemaCheckValidationError { 
                column: "webhook_id".to_string(),
                check: "webhook_id->NativeAction [default_pre_checks]".to_string(),
                error: "The specified webhook doesn't exist!".to_string(),
                accepted_range: "Valid webhook ID".to_string(),
            });
        }

        // Check if repo exists
        if let Some(Value::String(repo_id)) = state.state.get("repo_id") {
            // Check if the webhook exists
            let repo = sqlx::query!(
                "SELECT COUNT(1) FROM gitlogs__repos WHERE id = $1 AND guild_id = $2",
                repo_id,
                ctx.guild_id.to_string()
            )
            .fetch_one(&ctx.data.pool)
            .await
            .map_err(|e| SettingsError::Generic { 
                message: e.to_string(),
                src: "repo_id->NativeAction [default_pre_checks]".to_string(),
                typ: "database error".to_string(),
            })?;

            if repo.count.unwrap_or_default() == 0 {
                return Err(SettingsError::SchemaCheckValidationError { 
                    column: "repo_id".to_string(),
                    check: "repo_id->NativeAction [default_pre_checks]".to_string(),
                    error: "The specified repository does not exist".to_string(),
                    accepted_range: "Valid repository ID".to_string(),
                });
            }
        }

        Ok(())
    }
}
use futures_util::future::FutureExt;
use module_settings::types::{
    settings_wrap_columns, settings_wrap_precheck, settings_wrap_postactions, settings_wrap_datastore, Column, ColumnAction, ColumnSuggestion, ColumnType, ConfigOption, InnerColumnType, InnerColumnTypeStringKind, OperationSpecific, OperationType, SettingsError
};
use module_settings::data_stores::PostgresDataStore;
use serenity::all::{Permissions, ChannelType};
use splashcore_rs::value::Value;
use once_cell::sync::Lazy;

pub static WEBHOOKS: Lazy<ConfigOption> = Lazy::new(|| {
    ConfigOption {
        id: "webhooks",
        name: "Webhooks",
        description:
            "Stores a list of webhooks to which Github can post events to.",
        table: "gitlogs__webhooks",
        common_filters: indexmap::indexmap! {},
        default_common_filters: indexmap::indexmap! {
            "guild_id" => "{__guild_id}"
        },
        primary_key: "id",
        max_entries: 5,
        data_store: settings_wrap_datastore(PostgresDataStore {}),
        columns: settings_wrap_columns(vec![
            Column {
                id: "id",
                name: "Webhook ID",
                description: "Unique identifier for the webhook",
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::Normal }),
                nullable: false,
                unique: true,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![OperationType::Create],
                secret: false,
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {
                    OperationType::Create => vec![
                        // Set sink display type
                        ColumnAction::NativeAction {
                            action: Box::new(|_ctx, state| async move {
                                let id = botox::crypto::gen_random(128);
                                state.state.insert("id".to_string(), Value::String(id.to_string()));
                                state.bypass_ignore_for.insert("id".to_string());
                                Ok(())
                            }.boxed()),
                            on_condition: None
                        },
                    ],
                }),
                default_pre_checks: settings_wrap_precheck(vec![]),
            },
            module_settings::common_columns::guild_id("guild_id", "Guild ID", "The Guild ID the webhook belongs to"),
            Column {
                id: "comment",
                name: "Comment",
                description: "A comment to describe the webhook. Not used for any purpose beyond documentation.",
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: Some(64), allowed_values: vec![], kind: InnerColumnTypeStringKind::Normal }),
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
                default_pre_checks: settings_wrap_precheck(vec![]),
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
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: true,
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {
                    OperationType::Create => vec![
                        // Set sink display type
                        ColumnAction::NativeAction {
                            action: Box::new(|_ctx, state| async move {
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
                                        api_url=config::CONFIG.sites.api.get(),
                                        id=id,
                                        webh_secret=secret
                                    )
                                ));

                                Ok(())
                            }.boxed()),
                            on_condition: None
                        },
                    ],
                }),
                default_pre_checks: settings_wrap_precheck(vec![]),
            },
            module_settings::common_columns::created_at(),
            module_settings::common_columns::created_by(),
            module_settings::common_columns::last_updated_at(),
            module_settings::common_columns::last_updated_by(),
        ]),
        title_template: "{comment} - {id}",
        operations: indexmap::indexmap! {
            OperationType::View => OperationSpecific {
                corresponding_command: "gitlogs webhooks_list",
                columns_to_set: indexmap::indexmap! {},
            },
            OperationType::Create => OperationSpecific {
                corresponding_command: "gitlogs webhooks_create",
                columns_to_set: indexmap::indexmap! {
                    "created_at" => "{__now}",
                    "created_by" => "{__author}",
                    "last_updated_at" => "{__now}",
                    "last_updated_by" => "{__author}",
                },
            },
            OperationType::Update => OperationSpecific {
                corresponding_command: "gitlogs webhooks_update",
                columns_to_set: indexmap::indexmap! {
                    "last_updated_at" => "{__now}",
                    "last_updated_by" => "{__author}",
                },
            },
            OperationType::Delete => OperationSpecific {
                corresponding_command: "gitlogs webhooks_delete",
                columns_to_set: indexmap::indexmap! {},
            },
        },
        post_actions: settings_wrap_postactions(vec![])
    }
});

pub static REPOS: Lazy<ConfigOption> = Lazy::new(|| {
    ConfigOption {
        id: "repos",
        name: "Repositories",
        description:
            "In order for the Git Logs integration to post webhooks, you must provide a list of repositories",
        table: "gitlogs__repos",
        common_filters: indexmap::indexmap! {},
        default_common_filters: indexmap::indexmap! {
            "guild_id" => "{__guild_id}"
        },
        primary_key: "id",
        max_entries: 10,
        data_store: settings_wrap_datastore(PostgresDataStore {}),
        columns: settings_wrap_columns(vec![
            Column {
                id: "id",
                name: "Repo ID",
                description: "Unique identifier for the repository",
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::Normal }),
                nullable: false,
                unique: true,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![OperationType::Create],
                secret: false,
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {
                    OperationType::Create => vec![
                        // Set sink display type
                        ColumnAction::NativeAction {
                            action: Box::new(|_ctx, state| async move {
                                let id = botox::crypto::gen_random(32);
                                state.state.insert("id".to_string(), Value::String(id.to_string()));
                                state.bypass_ignore_for.insert("id".to_string());
                                Ok(())
                            }.boxed()),
                            on_condition: None
                        },
                    ],
                }),
                default_pre_checks: settings_wrap_precheck(vec![]),
            },
            Column {
                id: "webhook_id",
                name: "Webhook ID",
                description: "The webhook to which the repository will post events to.",
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::Normal }),
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::Dynamic {
                    table_name: "gitlogs__webhooks",
                    value_column: "comment",
                    id_column: "id",
                    guild_id_column: "guild_id",
                },
                ignored_for: vec![],
                secret: false,
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
                default_pre_checks: settings_wrap_precheck(vec![
                    // Set sink display type
                    ColumnAction::NativeAction {
                        action: Box::new(|ctx, state| async move { 
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
                            .fetch_one(ctx.pool)
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

                            Ok(()) 
                        }.boxed()),
                        on_condition: None,
                    },
                ]),
            },
            module_settings::common_columns::guild_id("guild_id", "Guild ID", "The Guild ID the repository belongs to"),
            Column {
                id: "repo_name",
                name: "Repository Name [format: org/repo]",
                description: "The name of the repository in the format of org/repo.\n\n**Example**: Anti-Raid/splashtail",
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::Normal }),
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {
                    OperationType::Create => vec![
                        // Set sink display type
                        ColumnAction::NativeAction {
                            action: Box::new(|ctx, state| async move { 
                                let Some(Value::String(webhook_id)) = state.state.get("webhook_id") else {
                                    return Err(SettingsError::MissingOrInvalidField { 
                                        field: "webhook_id".to_string(),
                                        src: "webhook_id->NativeAction [default_pre_checks]".to_string(),
                                    });
                                };

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
                                    .fetch_one(ctx.pool)
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

                                } else {
                                    return Ok(())
                                }

                                Ok(()) 
                            }.boxed()),
                            on_condition: None,
                        },
                    ]
                }),
                default_pre_checks: settings_wrap_precheck(vec![]),
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
        title_template: "{repo_name} - {id}",
        operations: indexmap::indexmap! {
            OperationType::View => OperationSpecific {
                corresponding_command: "gitlogs repo_list",
                columns_to_set: indexmap::indexmap! {},
            },
            OperationType::Create => OperationSpecific {
                corresponding_command: "gitlogs repo_create",
                columns_to_set: indexmap::indexmap! {
                    "created_at" => "{__now}",
                    "created_by" => "{__author}",
                    "last_updated_at" => "{__now}",
                    "last_updated_by" => "{__author}",
                },
            },
            OperationType::Update => OperationSpecific {
                corresponding_command: "gitlogs repo_update",
                columns_to_set: indexmap::indexmap! {
                    "last_updated_at" => "{__now}",
                    "last_updated_by" => "{__author}",
                },
            },
            OperationType::Delete => OperationSpecific {
                corresponding_command: "gitlogs repo_delete",
                columns_to_set: indexmap::indexmap! {},
            },
        },
        post_actions: settings_wrap_postactions(vec![])
    }
});

pub static EVENT_MODIFIERS: Lazy<ConfigOption> = Lazy::new(|| {
    ConfigOption {
        id: "event_modifiers",
        name: "Event Modifiers",
        description:
            "An event modifier allows customizing and redirecting webhooks based on the event type.",
        table: "gitlogs__event_modifiers",
        common_filters: indexmap::indexmap! {},
        default_common_filters: indexmap::indexmap! {
            "guild_id" => "{__guild_id}"
        },
        primary_key: "id",
        max_entries: 50,
        data_store: settings_wrap_datastore(PostgresDataStore {}),
        columns: settings_wrap_columns(vec![
            Column {
                id: "id",
                name: "Modifier ID",
                description: "Unique identifier for the event modifier",
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::Normal }),
                nullable: false,
                unique: true,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![OperationType::Create],
                secret: false,
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {
                    OperationType::Create => vec![
                        // Set sink display type
                        ColumnAction::NativeAction {
                            action: Box::new(|_ctx, state| async move {
                                let id = botox::crypto::gen_random(256);
                                state.state.insert("id".to_string(), Value::String(id.to_string()));
                                state.bypass_ignore_for.insert("id".to_string());
                                Ok(())
                            }.boxed()),
                            on_condition: None
                        },
                    ],
                }),
                default_pre_checks: settings_wrap_precheck(vec![]),
            },
            Column {
                id: "webhook_id",
                name: "Webhook ID",
                description: "The webhook to which the repository will post events to.",
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::Normal }),
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::Dynamic {
                    table_name: "gitlogs__webhooks",
                    value_column: "comment",
                    id_column: "id",
                    guild_id_column: "guild_id",
                },
                ignored_for: vec![],
                secret: false,
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
                default_pre_checks: settings_wrap_precheck(vec![
                    // Set sink display type
                    ColumnAction::NativeAction {
                        action: Box::new(|ctx, state| async move { 
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
                            .fetch_one(ctx.pool)
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

                            Ok(()) 
                        }.boxed()),
                        on_condition: None,
                    },
                ]),
            },
            Column {
                id: "repo_id",
                name: "Repo ID",
                description: "The repository to which the modifier will apply. If not set, the modifier will apply to all repositories.",
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::Normal }),
                nullable: true,
                unique: false,
                suggestions: ColumnSuggestion::Dynamic {
                    table_name: "gitlogs__repos",
                    value_column: "repo_name",
                    id_column: "id",
                    guild_id_column: "guild_id",
                },
                ignored_for: vec![],
                secret: false,
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
                default_pre_checks: settings_wrap_precheck(vec![
                    // Set sink display type
                    ColumnAction::NativeAction {
                        action: Box::new(|ctx, state| async move { 
                            if let Some(Value::String(repo_id)) = state.state.get("repo_id") {
                                // Check if the webhook exists
                                let repo = sqlx::query!(
                                    "SELECT COUNT(1) FROM gitlogs__repos WHERE id = $1 AND guild_id = $2",
                                    repo_id,
                                    ctx.guild_id.to_string()
                                )
                                .fetch_one(ctx.pool)
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

                            } else {
                                return Ok(())
                            }

                            Ok(()) 
                        }.boxed()),
                        on_condition: None,
                    },
                ]),
            },
            module_settings::common_columns::guild_id("guild_id", "Guild ID", "The Guild ID the event modifier belongs to"),
            Column {
                id: "events",
                name: "Events",
                description: "The events to which the modifier will apply. If not set, the modifier will apply to all events.",
                column_type: ColumnType::new_array(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::Normal }),
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
                default_pre_checks: settings_wrap_precheck(vec![]),
            },
            Column {
                id: "blacklisted",
                name: "Blacklisted",
                description: "If set to true, the modifier will block the event from being posted to the webhook.",
                column_type: ColumnType::new_scalar(InnerColumnType::Boolean {}),
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
                default_pre_checks: settings_wrap_precheck(vec![]),
            },
            Column {
                id: "whitelisted",
                name: "Whitelisted [Other events will not be allowed]",
                description: "If set to true, the modifier will only allow the specified events to be posted to the webhook.",
                column_type: ColumnType::new_scalar(InnerColumnType::Boolean {}),
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
                default_pre_checks: settings_wrap_precheck(vec![]),
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
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
                default_pre_checks: settings_wrap_precheck(vec![]),
            },
            Column {
                id: "priority",
                name: "Priority",
                description: "The priority of the modifier. The modifier with the highest priority will be applied first.",
                column_type: ColumnType::new_scalar(InnerColumnType::Integer {}),
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
                default_pre_checks: settings_wrap_precheck(vec![
                    ColumnAction::NativeAction {
                        action: Box::new(|_ctx, state| async move {
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

                            Ok(())
                        }.boxed()),
                        on_condition: None,
                    },
                ]),
            },
            module_settings::common_columns::created_at(),
            module_settings::common_columns::created_by(),
            module_settings::common_columns::last_updated_at(),
            module_settings::common_columns::last_updated_by(),
        ]),
        title_template: "{id}",
        operations: indexmap::indexmap! {
            OperationType::View => OperationSpecific {
                corresponding_command: "gitlogs eventmods_list",
                columns_to_set: indexmap::indexmap! {},
            },
            OperationType::Create => OperationSpecific {
                corresponding_command: "gitlogs eventmods_create",
                columns_to_set: indexmap::indexmap! {
                    "created_at" => "{__now}",
                    "created_by" => "{__author}",
                    "last_updated_at" => "{__now}",
                    "last_updated_by" => "{__author}",
                },
            },
            OperationType::Update => OperationSpecific {
                corresponding_command: "gitlogs eventmods_update",
                columns_to_set: indexmap::indexmap! {
                    "last_updated_at" => "{__now}",
                    "last_updated_by" => "{__author}",
                },
            },
            OperationType::Delete => OperationSpecific {
                corresponding_command: "gitlogs eventmods_delete",
                columns_to_set: indexmap::indexmap! {},
            },
        },
        post_actions: settings_wrap_postactions(vec![])
    }
});

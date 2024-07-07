use futures_util::FutureExt;
use module_settings::types::{
    settings_wrap_columns, settings_wrap_precheck, Column, ColumnAction, ColumnSuggestion, ColumnType, ConfigOption, InnerColumnType, InnerColumnTypeStringKind, OperationSpecific, OperationType, SettingsError
};
use splashcore_rs::value::Value;
use once_cell::sync::Lazy;

pub static WEBHOOKS: Lazy<ConfigOption> = Lazy::new(|| {
    ConfigOption {
        id: "webhooks",
        name: "Webhooks",
        description:
            "Stores a list of webhooks to which Github can post events to.",
        table: "gitlogs__webhooks",
        guild_id: "guild_id",
        primary_key: "id",
        max_entries: 5,
        columns: settings_wrap_columns(vec![
            Column {
                id: "id",
                name: "Webhook ID",
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::Normal }),
                nullable: false,
                unique: true,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![OperationType::Create],
                secret: None,
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
            Column {
                id: "comment",
                name: "Comment",
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: Some(64), allowed_values: vec![], kind: InnerColumnTypeStringKind::Normal }),
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: None,
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
                default_pre_checks: settings_wrap_precheck(vec![]),
            },
            Column {
                id: "secret",
                name: "Secret",
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: Some(256), allowed_values: vec![], kind: InnerColumnTypeStringKind::Normal }),
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: Some(256),
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
    }
});

pub static REPOS: Lazy<ConfigOption> = Lazy::new(|| {
    ConfigOption {
        id: "repos",
        name: "Repositories",
        description:
            "In order for the Git Logs integration to post webhooks, you must provide a list of repositories",
        table: "gitlogs__repos",
        guild_id: "guild_id",
        primary_key: "id",
        max_entries: 10,
        columns: settings_wrap_columns(vec![
            Column {
                id: "id",
                name: "Repo ID",
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::Normal }),
                nullable: false,
                unique: true,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![OperationType::Create],
                secret: None,
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
                secret: None,
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
                            .fetch_one(&ctx.pool)
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
                id: "repo_name",
                name: "Repository Name [format: org/repo]",
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::Normal }),
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: None,
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
                                    .fetch_one(&ctx.pool)
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
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: Some(64), allowed_values: vec![], kind: InnerColumnTypeStringKind::Channel }),
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: None,
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
                default_pre_checks: settings_wrap_precheck(vec![]),
            },
            module_settings::common_columns::created_at(),
            module_settings::common_columns::created_by(),
            module_settings::common_columns::last_updated_at(),
            module_settings::common_columns::last_updated_by(),
        ]),
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
    }
});

pub static EVENT_MODIFIERS: Lazy<ConfigOption> = Lazy::new(|| {
    ConfigOption {
        id: "event_modifiers",
        name: "Event Modifiers",
        description:
            "An event modifier allows customizing and redirecting webhooks based on the event type.",
        table: "gitlogs__event_modifiers",
        guild_id: "guild_id",
        primary_key: "id",
        max_entries: 50,
        columns: settings_wrap_columns(vec![
            Column {
                id: "id",
                name: "Modifier ID",
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::Normal }),
                nullable: false,
                unique: true,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![OperationType::Create],
                secret: None,
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
                secret: None,
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
                            .fetch_one(&ctx.pool)
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
                secret: None,
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
                                .fetch_one(&ctx.pool)
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
            Column {
                id: "events",
                name: "Events",
                column_type: ColumnType::new_array(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::Normal }),
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: None,
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
                default_pre_checks: settings_wrap_precheck(vec![]),
            },
            Column {
                id: "blacklisted",
                name: "Blacklisted",
                column_type: ColumnType::new_scalar(InnerColumnType::Boolean {}),
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: None,
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
                default_pre_checks: settings_wrap_precheck(vec![]),
            },
            Column {
                id: "whitelisted",
                name: "Whitelisted [Other events will not be allowed]",
                column_type: ColumnType::new_scalar(InnerColumnType::Boolean {}),
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: None,
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
                default_pre_checks: settings_wrap_precheck(vec![]),
            },
            Column {
                id: "redirect_channel",
                name: "Redirect Channel",
                column_type: ColumnType::new_scalar(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::Channel }),
                nullable: true,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: None,
                pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
                default_pre_checks: settings_wrap_precheck(vec![]),
            },
            Column {
                id: "priority",
                name: "Priority",
                column_type: ColumnType::new_scalar(InnerColumnType::Integer {}),
                nullable: false,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: None,
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
    }
});

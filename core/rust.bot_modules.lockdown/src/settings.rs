use futures_util::FutureExt;
use module_settings::{
    data_stores::PostgresDataStore,
    types::{
        settings_wrap_columns, settings_wrap_datastore, settings_wrap_postactions,
        settings_wrap_precheck, Column, ColumnAction, ColumnSuggestion, ColumnType, ConfigOption,
        InnerColumnType, InnerColumnTypeStringKind, OperationSpecific, OperationType,
        SettingsError,
    },
};
use splashcore_rs::value::Value;
use std::sync::LazyLock;

pub static LOCKDOWN_SETTINGS: LazyLock<ConfigOption> = LazyLock::new(|| {
    ConfigOption {
    id: "lockdown_guilds",
    name: "Lockdown Settings",
    description: "Setup standard lockdown settings for a server",
    table: "lockdown__guilds",
    common_filters: indexmap::indexmap! {},
    default_common_filters: indexmap::indexmap! {
        "guild_id" => "{__guild_id}"
    },
    primary_key: "guild_id",
    max_entries: Some(1),
    max_return: 2,
    data_store: settings_wrap_datastore(PostgresDataStore {}),
    columns: settings_wrap_columns(vec![
        module_settings::common_columns::guild_id(
            "guild_id",
            "Guild ID",
            "Guild ID of the server in question",
        ),
        Column {
            id: "member_roles",
            name: "Member Roles",
            description: "Which roles to use as member roles for the purpose of lockdown. These roles will be explicitly modified during lockdown",
            column_type: ColumnType::new_array(InnerColumnType::String {
                kind: InnerColumnTypeStringKind::Role,
                min_length: None,
                max_length: None,
                allowed_values: vec![],
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
            id: "require_correct_layout",
            name: "Require Correct Layout",
            description: "Whether or not a lockdown can proceed even without correct critical role permissions. May lead to partial lockdowns if disabled",
            column_type: ColumnType::new_scalar(InnerColumnType::Boolean {}),
            nullable: false,
            unique: true,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![OperationType::Create],
            secret: false,
            pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
            default_pre_checks: settings_wrap_precheck(vec![]),
        },
    ]),
    title_template: "Lockdown Settings",
    operations: indexmap::indexmap! {
        OperationType::View => OperationSpecific {
            corresponding_command: "lockdown_settings view",
            columns_to_set: indexmap::indexmap! {},
        },
        OperationType::Create => OperationSpecific {
            corresponding_command: "lockdown_settings create",
            columns_to_set: indexmap::indexmap! {
                "created_at" => "{__now}",
                "created_by" => "{__author}",
                "last_updated_at" => "{__now}",
                "last_updated_by" => "{__author}",
            },
        },
        OperationType::Update => OperationSpecific {
            corresponding_command: "lockdown_settings update",
            columns_to_set: indexmap::indexmap! {
                "last_updated_at" => "{__now}",
                "last_updated_by" => "{__author}",
            },
        },
        OperationType::Delete => OperationSpecific {
            corresponding_command: "lockdown_settings delete",
            columns_to_set: indexmap::indexmap! {},
        },
    },
    post_actions: settings_wrap_postactions(vec![ColumnAction::NativeAction {
        action: Box::new(|ctx, _state| {
            async move {
                super::cache::GUILD_LOCKDOWN_SETTINGS
                    .invalidate(&ctx.guild_id)
                    .await;

                Ok(())
            }
            .boxed()
        }),
        on_condition: Some(|ctx, _state| Ok(ctx.operation_type != OperationType::View)),
    }]),
}
});

pub static QUICK_SERVER_LOCKDOWNS: LazyLock<ConfigOption> = LazyLock::new(|| ConfigOption {
    id: "quick_server_lockdowns",
    name: "Quick Server Lockdowns",
    description: "Quick server lockdowns",
    table: "lockdown__quick_server_lockdowns",
    common_filters: indexmap::indexmap! {},
    default_common_filters: indexmap::indexmap! {
        "guild_id" => "{__guild_id}"
    },
    primary_key: "guild_id",
    max_entries: Some(1),
    max_return: 5,
    data_store: settings_wrap_datastore(PostgresDataStore {}),
    columns: settings_wrap_columns(vec![
        module_settings::common_columns::guild_id(
            "guild_id",
            "Guild ID",
            "The Guild ID referring to this lockdown",
        ),
        Column {
            id: "reason",
            name: "Reason",
            description: "The reason for starting the lockdown.",
            column_type: ColumnType::new_scalar(InnerColumnType::String {
                kind: InnerColumnTypeStringKind::Normal,
                min_length: Some(1),
                max_length: Some(256),
                allowed_values: vec![],
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
            id: "old_permissions",
            name: "Old Permissions",
            description: "The old permissions",
            column_type: ColumnType::new_scalar(InnerColumnType::Json {}),
            nullable: false,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![OperationType::Create],
            secret: false,
            pre_checks: settings_wrap_precheck(indexmap::indexmap! {
                OperationType::Create => vec![ColumnAction::NativeAction { 
                    action: Box::new(|ctx, state| {
                        async move {
                            let Some(Value::Boolean(ongoing)) = state.state.get("ongoing") else {
                                return Err(SettingsError::MissingOrInvalidField {
                                    field: "ongoing".to_string(),
                                    src: "index->NativeAction [pre_checks]".to_string(),
                                });
                            };

                            if *ongoing {
                                // Get the old permissions
                                let pg = proxy_support::guild(
                                    &ctx.data.cache_http,
                                    &ctx.data.reqwest,
                                    ctx.guild_id,
                                )
                                .await
                                .map_err(|e| {
                                    SettingsError::Generic {
                                        message: format!("Error while fetching guild: {}", e),
                                        typ: "value_error".to_string(),
                                        src: "lockdown__server_lockdowns.old_permissions".to_string(),
                                    }
                                })?;

                                let mut map = indexmap::IndexMap::new();

                                for role in pg.roles.iter() {
                                    map.insert(role.id.to_string(), Value::String(role.permissions.bits().to_string()));
                                }

                                state.bypass_ignore_for.insert("old_permissions".to_string());
                                state.state.insert("old_permissions".to_string(), Value::Map(map));
                            }

                            Ok(())
                        }
                        .boxed()
                    }),
                    on_condition: None
                }],
            }),
            default_pre_checks: settings_wrap_precheck(vec![]),
        },
        Column {
            id: "ongoing",
            name: "Ongoing",
            description: "Is the lockdown still ongoing",
            column_type: ColumnType::new_scalar(InnerColumnType::Boolean {}),
            nullable: true,
            unique: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
            pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
            default_pre_checks: settings_wrap_precheck(vec![
                    ColumnAction::NativeAction {
                        action: Box::new(|ctx, state| {
                            async move {
                                // Ensure we are set to internally disable the lockdown entirely
                                if ctx.operation_type == OperationType::Delete {
                                    state.state.insert("ongoing".to_string(), Value::Boolean(false));
                                }

                                let Some(Value::Boolean(ongoing)) = state.state.get("ongoing") else {
                                    return Err(SettingsError::MissingOrInvalidField {
                                        field: "ongoing".to_string(),
                                        src: "index->NativeAction [pre_checks]".to_string(),
                                    });
                                };

                                let lockdown_settings = super::cache::get_guild_lockdown_settings(
                                    &ctx.data.pool,
                                    ctx.guild_id
                                )
                                .await
                                .map_err(|e| {
                                    SettingsError::Generic {
                                        message: format!("Error while fetching lockdown settings: {}", e),
                                        typ: "value_error".to_string(),
                                        src: "lockdown__server_lockdowns.ongoing".to_string(),
                                    }
                                })?;
        
                                let pg = proxy_support::guild(
                                    &ctx.data.cache_http,
                                    &ctx.data.reqwest,
                                    ctx.guild_id,
                                )
                                .await
                                .map_err(|e| {
                                    SettingsError::Generic {
                                        message: format!("Error while fetching guild: {}", e),
                                        typ: "value_error".to_string(),
                                        src: "lockdown__server_lockdowns.ongoing".to_string(),
                                    }
                                })?;
        
                                if *ongoing && lockdown_settings.require_correct_layout {
                                    let lockdown_test = crate::quick::test_quick_lockdown(&pg, &lockdown_settings.member_roles)
                                    .await
                                    .map_err(|e| {
                                        SettingsError::Generic {
                                            message: format!("Error while testing lockdown: {}", e),
                                            typ: "value_error".to_string(),
                                            src: "lockdown__server_lockdowns.ongoing".to_string(),
                                        }
                                    })?;
        
                                    if !lockdown_test.can_apply_perfectly() {
                                        let mut needed_changes = String::new();
        
                                        needed_changes.push_str("The following roles need to be changed:\n");
                                        for (role_id, perms) in lockdown_test.changes_needed.iter() {
                                            if needed_changes.len() > 3700 {
                                                break;
                                            }

                                            let role_name = pg.roles.get(role_id).map(|r| r.name.to_string()).unwrap_or_else(|| "Unknown".to_string());
        
                                            needed_changes.push_str(&format!("Role: {} ({})\n", role_name, role_id));
                                            needed_changes.push_str(&format!("Permissions: {} {}\n", perms.0, perms.1));
                                            needed_changes.push('\n');
                                        }
        
                                        return Err(SettingsError::Generic {
                                            message: format!("Lockdown cannot proceed without correct layout. Needed changes:\n{}", needed_changes),
                                            typ: "value_error".to_string(),
                                            src: "lockdown__server_lockdowns.ongoing".to_string(),
                                        });
                                    }
                                } else if !*ongoing {
                                    let Some(Value::Map(old_permissions)) = state.state.get("old_permissions") else {
                                        return Err(SettingsError::MissingOrInvalidField {
                                            field: "old_permissions".to_string(),
                                            src: "index->NativeAction [post_actions]".to_string(),
                                        });
                                    };
                        
                                    for (k, v) in old_permissions.iter() {
                                        k.parse::<serenity::all::RoleId>().map_err(|e| {
                                            SettingsError::Generic {
                                                message: format!("Error while parsing role ID: {}", e),
                                                typ: "value_error".to_string(),
                                                src: "lockdown__server_lockdowns.ongoing".to_string(),
                                            }
                                        })?;
                                        v.as_u64().ok_or_else(|| {
                                            SettingsError::Generic {
                                                message: format!("Error while converting permissions: {}", v),
                                                typ: "value_error".to_string(),
                                                src: "lockdown__server_lockdowns.ongoing".to_string(),
                                            }
                                        })?;
                                     }            
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
    title_template: "Ongoing: {ongoing}, Reason: {reason}",
    operations: indexmap::indexmap! {
        OperationType::View => OperationSpecific {
            corresponding_command: "lockserver list",
            columns_to_set: indexmap::indexmap! {},
        },
        OperationType::Create => OperationSpecific {
            corresponding_command: "lockserver lock",
            columns_to_set: indexmap::indexmap! {
                "created_at" => "{__now}",
                "created_by" => "{__author}",
                "last_updated_at" => "{__now}",
                "last_updated_by" => "{__author}",
            },
        },
        OperationType::Update => OperationSpecific {
            corresponding_command: "lockserver update",
            columns_to_set: indexmap::indexmap! {
                "last_updated_at" => "{__now}",
                "last_updated_by" => "{__author}",
            },
        },
        OperationType::Delete => OperationSpecific {
            corresponding_command: "lockserver unlock",
            columns_to_set: indexmap::indexmap! {},
        }
    },
    post_actions: settings_wrap_postactions(vec![
        ColumnAction::NativeAction {
            action: Box::new(|ctx, state| {
                async move {
                    let Some(Value::Boolean(ongoing)) = state.state.get("ongoing") else {
                        return Err(SettingsError::MissingOrInvalidField {
                            field: "ongoing".to_string(),
                            src: "index->NativeAction [post_actions]".to_string(),
                        });
                    };

                    let lockdown_settings = super::cache::get_guild_lockdown_settings(
                        &ctx.data.pool,
                        ctx.guild_id
                    )
                    .await
                    .map_err(|e| {
                        SettingsError::Generic {
                            message: format!("Error while fetching lockdown settings: {}", e),
                            typ: "value_error".to_string(),
                            src: "lockdown__server_lockdowns.ongoing".to_string(),
                        }
                    })?;

                    let mut pg = proxy_support::guild(
                        &ctx.data.cache_http,
                        &ctx.data.reqwest,
                        ctx.guild_id,
                    )
                    .await
                    .map_err(|e| {
                        SettingsError::Generic {
                            message: format!("Error while fetching guild: {}", e),
                            typ: "value_error".to_string(),
                            src: "lockdown__server_lockdowns.ongoing".to_string(),
                        }
                    })?;

                    let crit_roles = super::quick::get_critical_roles(&pg, &lockdown_settings.member_roles)
                    .map_err(|e| {
                        SettingsError::Generic {
                            message: format!("Error while fetching critical roles: {}", e),
                            typ: "value_error".to_string(),
                            src: "lockdown__server_lockdowns.ongoing".to_string(),
                        }
                    })?;

                    if *ongoing {
                        super::quick::create_quick_lockdown(
                            &ctx.data.cache_http,
                            &mut pg,
                            crit_roles,
                        )
                        .await
                        .map_err(|e| {
                            SettingsError::Generic {
                                message: format!("Error while creating lockdown: {}", e),
                                typ: "value_error".to_string(),
                                src: "lockdown__server_lockdowns.ongoing".to_string(),
                            }
                        })?;
                    } else {
                        let Some(Value::Map(old_permissions)) = state.state.get("old_permissions") else {
                            return Err(SettingsError::MissingOrInvalidField {
                                field: "old_permissions".to_string(),
                                src: "index->NativeAction [post_actions]".to_string(),
                            });
                        };

                        let mut op = std::collections::HashMap::new();

                        for (k, v) in old_permissions.iter() {
                            let role_id = k.parse::<serenity::all::RoleId>().map_err(|e| {
                                SettingsError::Generic {
                                    message: format!("Error while parsing role ID: {}", e),
                                    typ: "value_error".to_string(),
                                    src: "lockdown__server_lockdowns.ongoing".to_string(),
                                }
                            })?;
                            let value = v.as_u64().ok_or_else(|| {
                                SettingsError::Generic {
                                    message: format!("Error while converting permissions: {}", v),
                                    typ: "value_error".to_string(),
                                    src: "lockdown__server_lockdowns.ongoing".to_string(),
                                }
                            })?;

                            let value_perms = serenity::all::Permissions::from_bits_retain(value);

                            op.insert(role_id, value_perms);
                        }

                        super::quick::revert_quick_lockdown(
                            &ctx.data.cache_http,
                            &mut pg,
                            crit_roles,
                            op,
                        )
                        .await
                        .map_err(|e| {
                            SettingsError::Generic {
                                message: format!("Error while reverting lockdown: {}", e),
                                typ: "value_error".to_string(),
                                src: "lockdown__server_lockdowns.ongoing".to_string(),
                            }
                        })?;
                    }

                    Ok(())
                }
                .boxed()
            }),
            on_condition: None,
        },
    ]),
});

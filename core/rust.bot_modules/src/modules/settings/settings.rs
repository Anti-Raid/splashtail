use kittycat::perms::Permission;
use module_settings::{
    data_stores::PostgresDataStore,
    types::{
        settings_wrap_columns, settings_wrap_datastore, settings_wrap_precheck, Column,
        ColumnAction, ColumnSuggestion, ColumnType, ConfigOption, InnerColumnType,
        InnerColumnTypeStringKind, OperationSpecific, OperationType, SettingsError,
    },
};
use futures_util::FutureExt;
use once_cell::sync::Lazy;
use splashcore_rs::value::Value;

pub static GUILD_ROLES: Lazy<ConfigOption> = Lazy::new(|| {
    ConfigOption {
    id: "guild_roles",
    name: "Guild Roles",
    description: "Configure/setup guild roles which can then have permissions on Anti-Raid",
    table: "guild_roles",
    guild_id: "guild_id",
    primary_key: "id",
    max_entries: 512,
    data_store: settings_wrap_datastore(PostgresDataStore {}),
    columns: settings_wrap_columns(vec![
        Column {
            id: "id",
            name: "ID",
            description: "The unique identifier for the guild role.",
            column_type: ColumnType::new_scalar(InnerColumnType::Uuid {}),
            nullable: false,
            unique: true,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![OperationType::Create],
            secret: false,
            pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
            default_pre_checks: settings_wrap_precheck(vec![]),
        },
        Column {
            id: "role_id",
            name: "Role ID",
            description: "The role ID",
            column_type: ColumnType::new_scalar(InnerColumnType::String {
                kind: InnerColumnTypeStringKind::Role,
                min_length: None,
                max_length: Some(64),
                allowed_values: vec![],
            }),
            nullable: false,
            unique: true,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![OperationType::Create],
            secret: false,
            pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
            default_pre_checks: settings_wrap_precheck(vec![]),
        },
        Column {
            id: "perms",
            name: "Permissions",
            description: "What permissions should the role have",
            column_type: ColumnType::new_array(InnerColumnType::String {
                kind: InnerColumnTypeStringKind::KittycatPermission,
                min_length: None,
                max_length: Some(64),
                allowed_values: vec![],
            }),
            nullable: false,
            unique: true,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
            pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
            default_pre_checks: settings_wrap_precheck(vec![]),
        },
        Column {
            id: "index",
            name: "Index",
            description: "Where in the role hierarchy should this role be on Anti-Raid for permission purposes. Note that a lower index implies higher on the hierarchy and vice versa",
            column_type: ColumnType::new_array(InnerColumnType::String {
                kind: InnerColumnTypeStringKind::KittycatPermission,
                min_length: None,
                max_length: Some(64),
                allowed_values: vec![],
            }),
            nullable: false,
            unique: true,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
            pre_checks: settings_wrap_precheck(indexmap::indexmap! {
                OperationType::View => vec![], // No pre-checks for view
            }),
            default_pre_checks: settings_wrap_precheck(vec![
                ColumnAction::NativeAction {
                    action: Box::new(|ctx, state| async move {
                        let Some(Value::String(settings_role_id_str)) = state.state.get("role_id") else {
                            return Err(SettingsError::MissingOrInvalidField { 
                                field: "role_id".to_string(),
                                src: "index->NativeAction [default_pre_checks]".to_string(),
                            });
                        };

                        let settings_role_id = settings_role_id_str.parse::<serenity::all::RoleId>().map_err(|e| SettingsError::Generic {
                            message: format!("Failed to parse role id despite already having parsed it: {:?}", e),
                            src: "NativeAction->index".to_string(),
                            typ: "internal".to_string(),
                        })?;

                        let guild = proxy_support::guild(ctx.cache_http, ctx.reqwest_client, ctx.guild_id)
                            .await
                            .map_err(|e| SettingsError::Generic {
                                message: format!("Failed to get guild: {:?}", e),
                                src: "NativeAction->index".to_string(),
                                typ: "internal".to_string(),
                            })?;

                        // If owner, early return
                        if guild.owner_id == ctx.author {
                            return Ok(())
                        }

                        let Some(member) = proxy_support::member_in_guild(ctx.cache_http, ctx.reqwest_client, ctx.guild_id, ctx.author)
                        .await
                        .map_err(|e| SettingsError::Generic {
                            message: format!("Failed to get member: {:?}", e),
                            src: "NativeAction->index".to_string(),
                            typ: "internal".to_string(),
                        })? else {
                            return Err(SettingsError::Generic {
                                message: "You must be in the guild to run this command!".to_string(),
                                src: "NativeAction->index".to_string(),
                                typ: "internal".to_string(),
                            });
                        };

                        let current_roles = sqlx::query!(
                            "SELECT index, role_id, perms FROM guild_roles WHERE guild_id = $1",
                            ctx.guild_id.to_string()
                        )
                        .fetch_all(&ctx.pool)
                        .await
                        .map_err(|e| SettingsError::Generic {
                            message: format!("Failed to get current role configuration: {:?}", e),
                            src: "NativeAction->index".to_string(),
                            typ: "internal".to_string(),
                        })?;

                        // Convert current_roles to a map on role_id
                        let current_roles = current_roles
                            .into_iter()
                            .map(|x| {
                                (
                                    x.role_id,
                                    (
                                        x.index,
                                        x.perms,
                                    )
                                )
                            })
                            .collect::<std::collections::HashMap<String, (i32, Vec<String>)>>();

                        let Some(first_role) = member.roles.first() else {
                            return Err(SettingsError::Generic {
                                message: "You must have at least one role to run this command!".to_string(),
                                src: "NativeAction->index".to_string(),
                                typ: "internal".to_string(),
                            });
                        };
                
                        let Some(first_role) = guild.roles.get(first_role) else {
                            return Err(SettingsError::Generic {
                                message: "Could not find your first role".to_string(),
                                src: "NativeAction->index".to_string(),
                                typ: "internal".to_string(),
                            });
                        };
                        
                        let mut highest_role = first_role;
                        let mut lowest_index: Option<i32> = None;
                
                        for r in &member.roles {
                            if let Some((index, _)) = current_roles.get(&r.to_string()) {
                                match lowest_index {
                                    Some(li) => {
                                        if *index < li {
                                            lowest_index = Some(*index);
                                        }
                                    }
                                    None => {
                                        lowest_index = Some(*index);
                                    }
                                }
                            }

                            let Some(r) = guild.roles.get(r) else {
                                continue;
                            };
            
                            if r > highest_role {
                                highest_role = r;
                            }
                        }

                        // Check that our index is lower than the targets index
                        let Some(lowest_index) = lowest_index else {
                            return Err(SettingsError::Generic {
                                message: "You do not have any Anti-Raid configured roles yet!".to_string(),
                                src: "NativeAction->index".to_string(),
                                typ: "index_check".to_string(),
                            });
                        };

                        let Some(settings_role) = guild.roles.get(&settings_role_id) else {
                            return Err(SettingsError::SchemaCheckValidationError { 
                                column: "role_id".to_string(),
                                check: "find_role_id_in_guild".to_string(),
                                error: "Could not find role in guild".to_string(),
                                accepted_range: "Any role in the guild".to_string(),
                            });
                        };
            
                        if highest_role <= settings_role {
                            return Err(SettingsError::Generic {
                                message: "You do not have permission to edit this role's permissions as they are higher than you".to_string(),
                                src: "NativeAction->index".to_string(),
                                typ: "internal".to_string(),
                            });
                        }

                        let author_kittycat_perms = crate::silverpelt::member_permission_calc::get_kittycat_perms(
                            &ctx.pool,
                            ctx.guild_id,
                            guild.owner_id,
                            member.user.id,
                            &member.roles,
                        )
                        .await
                        .map_err(|e| SettingsError::Generic {
                            message: format!("Failed to get kittycat permissions: {:?}", e),
                            src: "NativeAction->index".to_string(),
                            typ: "internal".to_string(),
                        })?;

                    // Get the new permissions as a Vec<String>
                    let Some(Value::List(perms_value)) = state.state.get("perms") else {
                        return Err(SettingsError::MissingOrInvalidField { 
                            field: "perms".to_string(),
                            src: "index->NativeAction [default_pre_checks]".to_string(),
                        });
                    };

                    let mut perms = Vec::with_capacity(perms_value.len());

                    for perm in perms_value {
                        if let Value::String(perm) = perm {
                            perms.push(perm);
                        } else {
                            return Err(SettingsError::Generic {
                                message: "Failed to parse permissions".to_string(),
                                src: "NativeAction->index".to_string(),
                                typ: "internal".to_string(),
                            });
                        }
                    }

                    // Get the new index and check it while we're at it
                    let Some(Value::Integer(new_index)) = state.state.get("index") else {
                        return Err(SettingsError::MissingOrInvalidField { 
                            field: "index".to_string(),
                            src: "index->NativeAction [default_pre_checks]".to_string(),
                        });
                    };

                    if *new_index < lowest_index.into() {
                        return Err(SettingsError::Generic {
                            message: format!("You do not have permission to edit this role's permissions as the new index would be lower than you: {} < {}", *new_index, lowest_index),
                            src: "NativeAction->index".to_string(),
                            typ: "internal".to_string(),
                        });
                    }

                    match ctx.operation_type {
                        OperationType::Create => {
                            kittycat::perms::check_patch_changes(
                                &author_kittycat_perms,
                                &[],
                                &perms
                                    .iter()
                                    .map(|x| Permission::from_string(x))
                                    .collect::<Vec<Permission>>(),
                            )
                            .map_err(|e| SettingsError::Generic {
                                    message: format!(
                                        "You do not have permission to add a role with these permissions: {}",
                                        e
                                    ),
                                    src: "NativeAction->index".to_string(),
                                    typ: "perm_check_failed".to_string(),
                            })?;
                        },
                        OperationType::Update => {
                            let Some((index, current_perms)) = current_roles.get(settings_role_id_str.as_str()) else {
                                return Err(SettingsError::Generic {
                                    message: "Could not find role in guild".to_string(),
                                    src: "NativeAction->index".to_string(),
                                    typ: "internal".to_string(),
                                });
                            };

                            if *index < lowest_index {
                                return Err(SettingsError::Generic {
                                    message: format!("You do not have permission to edit this role's permissions as the current index is lower than you: {} < {}", *index, lowest_index),
                                    src: "NativeAction->index".to_string(),
                                    typ: "internal".to_string(),
                                });
                            }

                            kittycat::perms::check_patch_changes(
                                &author_kittycat_perms,
                                &current_perms
                                    .iter()
                                    .map(|x| Permission::from_string(x))
                                    .collect::<Vec<Permission>>(),
                                &perms
                                    .iter()
                                    .map(|x| Permission::from_string(x))
                                    .collect::<Vec<Permission>>(),
                                )
                                .map_err(|e| {
                                    SettingsError::Generic {
                                        message: format!(
                                            "You do not have permission to edit this role's permissions: {}",
                                            e
                                        ),
                                        src: "NativeAction->index".to_string(),
                                        typ: "perm_check_failed".to_string(),
                                    }
                                })?;
                            },
                            OperationType::Delete => {
                                kittycat::perms::check_patch_changes(
                                    &author_kittycat_perms,
                                    &perms
                                        .iter()
                                        .map(|x| Permission::from_string(x))
                                        .collect::<Vec<Permission>>(),
                                    &[],
                                )
                                .map_err(|e| SettingsError::Generic {
                                    message: format!(
                                        "You do not have permission to remove a role with these permissions: {}",
                                        e
                                    ),
                                    src: "NativeAction->index".to_string(),
                                    typ: "perm_check_failed".to_string(),
                                })?;
                            },
                            _ => {
                                return Err(SettingsError::OperationNotSupported { operation: ctx.operation_type });
                            },
                        }
                        
                        Ok(())
                    }.boxed()),
                    on_condition: None,
                } 
            ]),
        },
    ]),
    title_template: "{index} - {role_id}",
    operations: indexmap::indexmap! {
        OperationType::View => OperationSpecific {
            corresponding_command: "guildroles list",
            columns_to_set: indexmap::indexmap! {},
        },
        OperationType::Create => OperationSpecific {
            corresponding_command: "guildroles add",
            columns_to_set: indexmap::indexmap! {
                "created_at" => "{__now}",
                "created_by" => "{__author}",
                "last_updated_at" => "{__now}",
                "last_updated_by" => "{__author}",
            },
        },
        OperationType::Update => OperationSpecific {
            corresponding_command: "guildroles edit",
            columns_to_set: indexmap::indexmap! {
                "last_updated_at" => "{__now}",
                "last_updated_by" => "{__author}",
            },
        },
        OperationType::Delete => OperationSpecific {
            corresponding_command: "guildroles remove",
            columns_to_set: indexmap::indexmap! {},
        },
    }
    }
});

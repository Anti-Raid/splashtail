use kittycat::perms::Permission;
use module_settings::state::State;
use module_settings::types::SettingsData;
use module_settings::{
    data_stores::{PostgresDataStore, PostgresDataStoreImpl},
    types::{
        settings_wrap, Column, ColumnSuggestion, ColumnType, ConfigOption, HookContext,
        InnerColumnType, InnerColumnTypeStringKind, OperationSpecific, OperationType, PostAction,
        SettingDataValidator, SettingsError, NoOpPostAction, NoOpValidator
    },
};
use splashcore_rs::value::Value;
use std::sync::LazyLock;

pub static GUILD_ROLES: LazyLock<ConfigOption> = LazyLock::new(|| {
    ConfigOption {
        id: "guild_roles",
        name: "Server Roles",
        description: "Configure/setup server roles which can then have permissions on AntiRaid",
        table: "guild_roles",
        common_filters: indexmap::indexmap! {},
        default_common_filters: indexmap::indexmap! {
            "guild_id" => "{__guild_id}"
        },
        primary_key: "role_id",
        max_entries: None,
        max_return: 20,
        data_store: settings_wrap(PostgresDataStore {}),
        columns: settings_wrap(vec![
            module_settings::common_columns::guild_id("guild_id", "Guild ID", "The Guild ID"),
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
                default: None,
                unique: true,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
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
                default: None,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
            },
            Column {
                id: "index",
                name: "Index",
                description: "Where in the role hierarchy should this role be on Anti-Raid for permission purposes. Note that a lower index implies higher on the hierarchy and vice versa",
                column_type: ColumnType::new_scalar(InnerColumnType::Integer {}),
                nullable: true,
                default: None,
                unique: true,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
            },
            Column {
                id: "display_name",
                name: "Display Name",
                description: "What should the role be displayed as in API's etc",
                column_type: ColumnType::new_scalar(InnerColumnType::String {
                    kind: InnerColumnTypeStringKind::KittycatPermission,
                    min_length: None,
                    max_length: Some(64),
                    allowed_values: vec![],
                }),
                nullable: true,
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
        title_template: "{index} - {role_id}",
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
        validator: settings_wrap(GuildRolesValidator {}),
        post_action: settings_wrap(GuildRolesPostAction {}),
    }
});

/// GuildRolesValidator handles all the required permission checking etc. for guild roles
pub struct GuildRolesValidator;

#[async_trait::async_trait]
impl SettingDataValidator for GuildRolesValidator {
    async fn validate<'a>(
        &self,
        ctx: HookContext<'a>,
        state: &'a mut State,
    ) -> Result<(), SettingsError> {
        // Early return if we are viewing, we don't need to check perms if so
        if ctx.operation_type == OperationType::View {
            return Ok(());
        }

        // This should be safe as all actions for Create/Update/Delete run after fetching all prerequisite fields
        let parsed_value = if let Some(new_index_val) = state.state.get("index") {
            match new_index_val {
                Value::Integer(new_index) => Value::Integer(*new_index),
                Value::None => Value::None,
                _ => {
                    return Err(SettingsError::MissingOrInvalidField {
                        field: "index".to_string(),
                        src: "index->NativeAction [default_pre_checks]".to_string(),
                    })
                }
            }
        } else {
            Value::None
        };

        let pg_data_store = PostgresDataStoreImpl::from_data_store(ctx.data_store)?;

        let new_index = match parsed_value {
            Value::Integer(new_index_val) => new_index_val,
            Value::None => {
                let highest_index_rec = if pg_data_store.tx.is_some() {
                    let tx = pg_data_store.tx.as_deref_mut().unwrap();

                    sqlx::query!(
                        "SELECT MAX(index) FROM guild_roles WHERE guild_id = $1",
                        ctx.guild_id.to_string()
                    )
                    .fetch_one(tx)
                    .await
                    .map_err(|e| SettingsError::Generic {
                        message: format!("Failed to get highest index: {:?}", e),
                        src: "NativeAction->index".to_string(),
                        typ: "internal".to_string(),
                    })?
                    .max
                    .unwrap_or(0)
                } else {
                    sqlx::query!(
                        "SELECT MAX(index) FROM guild_roles WHERE guild_id = $1",
                        ctx.guild_id.to_string()
                    )
                    .fetch_one(&ctx.data.pool)
                    .await
                    .map_err(|e| SettingsError::Generic {
                        message: format!("Failed to get highest index: {:?}", e),
                        src: "NativeAction->index".to_string(),
                        typ: "internal".to_string(),
                    })?
                    .max
                    .unwrap_or(0)
                };

                let index_i64 = (highest_index_rec + 1).into();

                state
                    .state
                    .insert("index".to_string(), Value::Integer(index_i64)); // Set the index

                index_i64
            }
            _ => {
                return Err(SettingsError::MissingOrInvalidField {
                    field: "index".to_string(),
                    src: "index->NativeAction [default_pre_checks]".to_string(),
                })
            }
        };

        let Some(Value::String(settings_role_id_str)) = state.state.get("role_id") else {
            return Err(SettingsError::MissingOrInvalidField {
                field: "role_id".to_string(),
                src: "index->NativeAction [default_pre_checks]".to_string(),
            });
        };

        let settings_role_id = settings_role_id_str
            .parse::<serenity::all::RoleId>()
            .map_err(|e| SettingsError::Generic {
                message: format!(
                    "Failed to parse role id despite already having parsed it: {:?}",
                    e
                ),
                src: "NativeAction->index".to_string(),
                typ: "internal".to_string(),
            })?;

        let guild = sandwich_driver::guild(&ctx.data.cache_http, &ctx.data.reqwest, ctx.guild_id)
            .await
            .map_err(|e| SettingsError::Generic {
                message: format!("Failed to get guild: {:?}", e),
                src: "NativeAction->index".to_string(),
                typ: "internal".to_string(),
            })?;

        // If owner, early return
        if guild.owner_id == ctx.author {
            return Ok(());
        }

        let Some(member) = sandwich_driver::member_in_guild(
            &ctx.data.cache_http,
            &ctx.data.reqwest,
            ctx.guild_id,
            ctx.author,
        )
        .await
        .map_err(|e| SettingsError::Generic {
            message: format!("Failed to get member: {:?}", e),
            src: "NativeAction->index".to_string(),
            typ: "internal".to_string(),
        })?
        else {
            return Err(SettingsError::Generic {
                message: "You must be in the guild to run this command!".to_string(),
                src: "NativeAction->index".to_string(),
                typ: "internal".to_string(),
            });
        };

        let current_roles = if pg_data_store.tx.is_some() {
            let tx = pg_data_store.tx.as_deref_mut().unwrap();
            let query = sqlx::query!(
                "SELECT index, role_id, perms FROM guild_roles WHERE guild_id = $1",
                ctx.guild_id.to_string()
            )
            .fetch_all(tx)
            .await
            .map_err(|e| SettingsError::Generic {
                message: format!("Failed to get current role configuration: {:?}", e),
                src: "NativeAction->index".to_string(),
                typ: "internal".to_string(),
            })?;

            query
                .into_iter()
                .map(|x| (x.role_id, (x.index, x.perms)))
                .collect::<std::collections::HashMap<String, (i32, Vec<String>)>>()
        } else {
            let query = sqlx::query!(
                "SELECT index, role_id, perms FROM guild_roles WHERE guild_id = $1",
                ctx.guild_id.to_string()
            )
            .fetch_all(&ctx.data.pool)
            .await
            .map_err(|e| SettingsError::Generic {
                message: format!("Failed to get current role configuration: {:?}", e),
                src: "NativeAction->index".to_string(),
                typ: "internal".to_string(),
            })?;

            query
                .into_iter()
                .map(|x| (x.role_id, (x.index, x.perms)))
                .collect::<std::collections::HashMap<String, (i32, Vec<String>)>>()
        };

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

        let author_kittycat_perms = if pg_data_store.tx.is_some() {
            let tx = pg_data_store.tx.as_deref_mut().unwrap();

            silverpelt::member_permission_calc::get_kittycat_perms(
                &mut *tx,
                ctx.guild_id,
                guild.owner_id,
                ctx.author,
                &member.roles,
            )
            .await
            .map_err(|e| SettingsError::Generic {
                message: format!("Failed to get author permissions: {:?}", e),
                src: "NativeAction->index".to_string(),
                typ: "internal".to_string(),
            })?
        } else {
            let mut conn = ctx.data.pool.acquire().await.map_err(|e| SettingsError::Generic {
                message: format!("Failed to get connection: {:?}", e),
                src: "NativeAction->index".to_string(),
                typ: "internal".to_string(),
            })?;
            silverpelt::member_permission_calc::get_kittycat_perms(
                &mut conn,
                ctx.guild_id,
                guild.owner_id,
                ctx.author,
                &member.roles,
            )
            .await
            .map_err(|e| SettingsError::Generic {
                message: format!("Failed to get author permissions: {:?}", e),
                src: "NativeAction->index".to_string(),
                typ: "internal".to_string(),
            })?
        };

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

        if new_index < lowest_index.into() {
            return Err(SettingsError::Generic {
            message: format!("You do not have permission to edit this role's permissions as the new index would be lower than you: {} < {}", new_index, lowest_index),
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
            }
            OperationType::Update => {
                let Some((index, current_perms)) = current_roles.get(settings_role_id_str.as_str())
                else {
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
                .map_err(|e| SettingsError::Generic {
                    message: format!(
                        "You do not have permission to edit this role's permissions: {}",
                        e
                    ),
                    src: "NativeAction->index".to_string(),
                    typ: "perm_check_failed".to_string(),
                })?;
            }
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
            }
            _ => {
                return Err(SettingsError::OperationNotSupported {
                    operation: ctx.operation_type,
                });
            }
        }

        Ok(())
    }
}

/// Updates the cache to request a permission rederive
pub struct GuildRolesPostAction;

#[async_trait::async_trait]
impl PostAction for GuildRolesPostAction {
    async fn post_action<'a>(
        &self,
        ctx: HookContext<'a>,
        state: &'a mut State,
    ) -> Result<(), SettingsError> {
        let Some(Value::String(settings_role_id_str)) = state.state.get("role_id") else {
            return Err(SettingsError::MissingOrInvalidField {
                field: "role_id".to_string(),
                src: "index->NativeAction [default_pre_checks]".to_string(),
            });
        };

        sqlx::query!(
            "UPDATE guild_members SET needs_perm_rederive = true WHERE guild_id = $1 AND $2 = ANY(roles)",
            ctx.guild_id.to_string(),
            settings_role_id_str.to_string()
        )
        .execute(&ctx.data.pool)
        .await
        .map_err(|e| SettingsError::Generic {
            message: format!("Failed to update guild members: {:?}", e),
            src: "post_action#GuildRoles".to_string(),
            typ: "internal".to_string(),
        })?;

        Ok(())
    }
}

pub static GUILD_MEMBERS: LazyLock<ConfigOption> = LazyLock::new(|| {
    ConfigOption {
        id: "guild_members",
        name: "Server Members",
        description: "Manage server members",
        table: "guild_members",
        common_filters: indexmap::indexmap! {},
        default_common_filters: indexmap::indexmap! {
            "guild_id" => "{__guild_id}"
        },
        primary_key: "user_id",
        max_entries: None,
        max_return: 20,
        data_store: settings_wrap(PostgresDataStore {}),
        columns: settings_wrap(vec![
            module_settings::common_columns::guild_id("guild_id", "Guild ID", "The Guild ID"),
            Column {
                id: "user_id",
                name: "User ID",
                description: "The user ID. Cannot be updated once set",
                column_type: ColumnType::new_scalar(InnerColumnType::String {
                    kind: InnerColumnTypeStringKind::User,
                    min_length: None,
                    max_length: Some(64),
                    allowed_values: vec![],
                }),
                nullable: false,
                default: None,
                unique: true,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![OperationType::Update], 
                secret: false,
            },
            Column {
                id: "roles",
                name: "Roles",
                description: "The roles the member has. Cannot be editted and is updated internally",
                column_type: ColumnType::new_array(InnerColumnType::String {
                    kind: InnerColumnTypeStringKind::Role,
                    min_length: None,
                    max_length: Some(64),
                    allowed_values: vec![],
                }),
                nullable: false,
                default: None,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![OperationType::Create, OperationType::Update, OperationType::Delete],
                secret: false,
            },
            Column {
                id: "perm_overrides",
                name: "Permission Overrides",
                description: "Any permission overrides the member has. This can and should be edited if needed",
                column_type: ColumnType::new_array(InnerColumnType::String {
                    kind: InnerColumnTypeStringKind::KittycatPermission,
                    min_length: None,
                    max_length: Some(64),
                    allowed_values: vec![],
                }),
                nullable: false,
                default: Some(|_| Value::List(Vec::new())),
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
            },
            Column {
                id: "resolved_perms_cache",
                name: "Resolved Permissions Cache",
                description: "A cache of the resolved permissions for the member. This is updated internally and cannot be edited but could be useful for debugging",
                column_type: ColumnType::new_array(InnerColumnType::String {
                    kind: InnerColumnTypeStringKind::KittycatPermission,
                    min_length: None,
                    max_length: Some(64),
                    allowed_values: vec![],
                }),
                nullable: false,
                default: None,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![OperationType::Create, OperationType::Update, OperationType::Delete],
                secret: false,
            },
            Column {
                id: "needs_perm_rederive",
                name: "Needs Permission Rederive",
                description: "Whether the member needs their permissions rederived. This is updated internally and cannot be edited but could be useful for debugging",
                column_type: ColumnType::new_scalar(InnerColumnType::Boolean {}),
                nullable: false,
                default: None,
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![OperationType::Create, OperationType::Update, OperationType::Delete],
                secret: false,
            },
            Column {
                id: "public",
                name: "Public",
                description: "Whether the member is public or not",
                column_type: ColumnType::new_scalar(InnerColumnType::Boolean {}),
                nullable: false,
                default: Some(|_| Value::Boolean(false)),
                unique: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
            },
            module_settings::common_columns::created_at(),
        ]),
        title_template: "{user_id}, perm_overrides={perm_overrides}",
        operations: indexmap::indexmap! {
            OperationType::View => OperationSpecific {
                columns_to_set: indexmap::indexmap! {},
            },
            OperationType::Create => OperationSpecific {
                columns_to_set: indexmap::indexmap! {
                    "created_at" => "{__now}",
                    "needs_perm_rederive" => "{__true}",
                },
            },
            OperationType::Update => OperationSpecific {
                columns_to_set: indexmap::indexmap! {
                    "needs_perm_rederive" => "{__true}",
                },
            },
            OperationType::Delete => OperationSpecific {
                columns_to_set: indexmap::indexmap! {},
            },
        },
        validator: settings_wrap(GuildMembersValidator {}),
        post_action: settings_wrap(NoOpPostAction {}),
    }
});

/// GuildMembersValidator handles all the required permission checking etc. for guild members
pub struct GuildMembersValidator;

impl GuildMembersValidator {
    async fn get_kittycat_perms_for_user<'a>(
        &self, 
        data: &SettingsData, 
        conn: &mut sqlx::PgConnection,
        guild_id: serenity::all::GuildId,
        guild_owner_id: serenity::all::UserId, 
        user_id: serenity::all::UserId,
) -> Result<(Vec<serenity::all::RoleId>, Vec<kittycat::perms::Permission>), SettingsError> {
        let Some(member) = sandwich_driver::member_in_guild(
            &data.cache_http,
            &data.reqwest,
            guild_id,
            user_id,
        )
        .await
        .map_err(|e| SettingsError::Generic {
            message: format!("Failed to get user {}: {:?}", user_id, e),
            src: "NativeAction->index".to_string(),
            typ: "internal".to_string(),
        })? else {
            return Ok((
                Vec::new(),
                Vec::new(),
            ));
        };

        let kittycat_perms = silverpelt::member_permission_calc::get_kittycat_perms(
            &mut *conn,
            guild_id,
            guild_owner_id,
            user_id,
            &member.roles,
        )
        .await
        .map_err(|e| SettingsError::Generic {
            message: format!("Failed to get user permissions: {:?} ({})", e, user_id),
            src: "NativeAction->index".to_string(),
            typ: "internal".to_string(),
        })?;

        let roles = member.roles.iter().copied().collect::<Vec<serenity::all::RoleId>>();

        Ok((roles, kittycat_perms))
    }
}

#[async_trait::async_trait]
impl SettingDataValidator for GuildMembersValidator {
    async fn validate<'a>(
        &self,
        ctx: HookContext<'a>,
        state: &'a mut State,
    ) -> Result<(), SettingsError> {
        // Early return if we are viewing, we don't need to check perms if so
        if ctx.operation_type == OperationType::View {
            return Ok(());
        }

        // Get the user id as this is required for all operations
        let Some(Value::String(user_id)) = state.state.get("user_id") else {
            return Err(SettingsError::MissingOrInvalidField {
                field: "user_id".to_string(),
                src: "guildmembers->user_id".to_string(),
            });
        };

        // Parse the user id
        let user_id: serenity::all::UserId = user_id
            .parse()
            .map_err(|e| SettingsError::Generic {
                message: format!("Failed to parse user id: {:?}", e),
                src: "guildmembers->user_id".to_string(),
                typ: "external".to_string(),
            })?;

        // Only the author can set public to true
        if !ctx.unchanged_fields.contains(&"public".to_string()) {
            if let Some(Value::Boolean(public)) = state.state.get("public") {
                if *public && ctx.author != user_id {
                    return Err(SettingsError::Generic {
                        message: "Only the author can set publicity".to_string(),
                        src: "guildmembers->public".to_string(),
                        typ: "external".to_string(),
                    });
                }
            }
        }

        // Get perm overrides
        let perm_overrides = {
            let Some(Value::List(perm_overrides_value)) = state.state.get("perm_overrides") else {
                return Err(SettingsError::MissingOrInvalidField {
                    field: "perm_overrides".to_string(),
                    src: "guildmembers->perm_overrides".to_string(),
                });
            };

            let mut perm_overrides = Vec::with_capacity(perm_overrides_value.len());

            for perm in perm_overrides_value {
                if let Value::String(perm) = perm {
                    perm_overrides.push(kittycat::perms::Permission::from_string(perm));
                } else {
                    return Err(SettingsError::Generic {
                        message: "Failed to parse permissions".to_string(),
                        src: "NativeAction->index".to_string(),
                        typ: "internal".to_string(),
                    });
                }
            }

            perm_overrides
        };

        let guild = sandwich_driver::guild(&ctx.data.cache_http, &ctx.data.reqwest, ctx.guild_id)
        .await
        .map_err(|e| SettingsError::Generic {
            message: format!("Failed to get guild: {:?}", e),
            src: "NativeAction->index".to_string(),
            typ: "internal".to_string(),
        })?;

        // If owner, early return
        if guild.owner_id == ctx.author {
            return Ok(());
        }

        let settings_data = ctx.data;

        let pg_data_store = PostgresDataStoreImpl::from_data_store(ctx.data_store)?;

        // Get the transaction connection or acquire one from pool if not in a transaction
        let conn = if pg_data_store.tx.is_some() {
            pg_data_store.tx.as_deref_mut().unwrap()
        } else {
            &mut *ctx.data.pool.acquire().await.map_err(|e| SettingsError::Generic {
                message: format!("Failed to get connection: {:?}", e),
                src: "NativeAction->index".to_string(),
                typ: "internal".to_string(),
            })?
        };

        // Get the authors kittycat permissions
        let author_kittycat_perms = match self.get_kittycat_perms_for_user(settings_data, conn, ctx.guild_id, guild.owner_id, ctx.author).await {
            Ok((_, author_kittycat_perms)) => author_kittycat_perms,
            Err(e) => return Err(SettingsError::Generic {
                message: format!("Failed to get author permissions: {:?}", e),
                src: "NativeAction->index".to_string(),
                typ: "internal".to_string(),
            }),
        };

        // Get the target members current kittycat permissions (if any) as well as their roles (for finding new permissions with overrides taken into account)
        let (target_member_roles, current_user_kittycat_perms) = match self.get_kittycat_perms_for_user(settings_data, conn, ctx.guild_id, guild.owner_id, user_id).await {
            Ok((target_member_roles, current_user_kittycat_perms)) => (target_member_roles, current_user_kittycat_perms),
            Err(e) => return Err(SettingsError::Generic {
                message: format!("Failed to get target member permissions: {:?}", e),
                src: "NativeAction->index".to_string(),
                typ: "internal".to_string(),
            }),
        };

        // Find new user's permissions with the given perm overrides
        let new_user_kittycat_perms = {
            let roles_str = silverpelt::member_permission_calc::create_roles_list_for_guild(&target_member_roles, ctx.guild_id);

            let user_positions = silverpelt::member_permission_calc::get_user_positions_from_db(&mut *conn, ctx.guild_id, &roles_str).await
            .map_err(|e| SettingsError::Generic {
                message: format!("Failed to get user positions: {:?}", e),
                src: "NativeAction->index".to_string(),
                typ: "internal".to_string(),
            })?;

            silverpelt::member_permission_calc::rederive_perms_impl(ctx.guild_id, user_id, user_positions, perm_overrides)
        };

        // Check permissions
        match ctx.operation_type {
            OperationType::Create => {
                kittycat::perms::check_patch_changes(
                    &author_kittycat_perms,
                    &[],
                    &new_user_kittycat_perms,
                )
                .map_err(|e| SettingsError::Generic {
                    message: format!(
                        "You do not have permission to add a role with these permissions: {}",
                        e
                    ),
                    src: "NativeAction->index".to_string(),
                    typ: "perm_check_failed".to_string(),
                })?;
            }
            OperationType::Update => {
                kittycat::perms::check_patch_changes(
                    &author_kittycat_perms,
                    &current_user_kittycat_perms,
                    &new_user_kittycat_perms,
                )
                .map_err(|e| SettingsError::Generic {
                    message: format!(
                        "You do not have permission to edit this role's permissions: {}",
                        e
                    ),
                    src: "NativeAction->index".to_string(),
                    typ: "perm_check_failed".to_string(),
                })?;
            }
            OperationType::Delete => {
                kittycat::perms::check_patch_changes(
                    &author_kittycat_perms,
                    &current_user_kittycat_perms,
                    &[],
                )
                .map_err(|e| SettingsError::Generic {
                    message: format!(
                        "You do not have permission to remove this members permission overrides: {}",
                        e
                    ),
                    src: "NativeAction->index".to_string(),
                    typ: "perm_check_failed".to_string(),
                })?;
            }
            _ => {
                return Err(SettingsError::OperationNotSupported {
                    operation: ctx.operation_type,
                });
            }
        }
        
        Ok(())
    }
}

pub static GUILD_TEMPLATES: LazyLock<ConfigOption> = LazyLock::new(|| {
    ConfigOption {
        id: "guild_templates",
        name: "Server Templates",
        description: "Configure/Setup Server Templates (Lua/Roblox Luau scripts)",
        table: "guild_templates",
        common_filters: indexmap::indexmap! {},
        default_common_filters: indexmap::indexmap! {
            "guild_id" => "{__guild_id}"
        },
        primary_key: "name",
        max_entries: None,
        max_return: 10,
        data_store: settings_wrap(PostgresDataStore {}),
        columns: settings_wrap(vec![
            module_settings::common_columns::guild_id("guild_id", "Guild ID", "The Guild ID"),
            Column {
                id: "name",
                name: "Name",
                description: "The name to give to the template",
                column_type: ColumnType::new_scalar(InnerColumnType::String {
                    kind: InnerColumnTypeStringKind::Normal,
                    min_length: None,
                    max_length: Some(64),
                    allowed_values: vec![],
                }),
                nullable: false,
                default: None,
                unique: true,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
            },
            Column {
                id: "content",
                name: "Content",
                description: "The content of the template",
                column_type: ColumnType::new_scalar(InnerColumnType::String {
                    kind: InnerColumnTypeStringKind::Textarea {
                        ctx: "template",
                    },
                    min_length: None,
                    max_length: None,
                    allowed_values: vec![],
                }),
                nullable: false,
                default: None,
                unique: true,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
            },
            module_settings::common_columns::created_at(),
            module_settings::common_columns::created_by(),
            module_settings::common_columns::last_updated_at(),
            module_settings::common_columns::last_updated_by(),
        ]),
        title_template: "{name}",
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
        post_action: settings_wrap(NoOpPostAction {}),
    }
});

pub struct GuildTemplateValidator;

#[async_trait::async_trait]
impl SettingDataValidator for GuildTemplateValidator {
    async fn validate<'a>(
        &self,
        ctx: HookContext<'a>,
        state: &'a mut State,
    ) -> Result<(), SettingsError> {
        if ctx.operation_type == OperationType::View {
            return Ok(());
        }

        let Some(Value::String(s)) = state.state.get("content") else {
            return Err(SettingsError::MissingOrInvalidField {
                field: "content".to_string(),
                src: "guild_templates->content".to_string(),
            });
        };

        let compiled = templating::parse(
            ctx.guild_id,
            templating::Template::Raw(s.to_string()),
            ctx.data.pool.clone(),
            ctx.data.cache_http.clone(),
            ctx.data.reqwest.clone(),
        )
        .await;

        if let Err(err) = compiled {
            return Err(SettingsError::SchemaCheckValidationError {
                column: "content".to_string(),
                check: "GuildTemplateValidator".to_string(),
                accepted_range: "Valid template".to_string(),
                error: err.to_string(),
            });
        }

        Ok(())
    }
}

pub static GUILD_TEMPLATES_KV: LazyLock<ConfigOption> = LazyLock::new(|| {
    ConfigOption {
        id: "guild_templates_kv",
        name: "Server Templates (key-value db)",
        description: "Key-value database available to templates on this server",
        table: "guild_templates_kv",
        common_filters: indexmap::indexmap! {},
        default_common_filters: indexmap::indexmap! {
            "guild_id" => "{__guild_id}"
        },
        primary_key: "key",
        max_entries: Some(templating::LuaKVConstraints::default().max_keys),
        max_return: 10,
        data_store: settings_wrap(PostgresDataStore {}),
        columns: settings_wrap(vec![
            module_settings::common_columns::guild_id("guild_id", "Guild ID", "The Guild ID"),
            Column {
                id: "key",
                name: "Key",
                description: "key",
                column_type: ColumnType::new_scalar(InnerColumnType::String {
                    kind: InnerColumnTypeStringKind::Normal,
                    min_length: None,
                    max_length: Some(templating::LuaKVConstraints::default().max_key_length),
                    allowed_values: vec![],
                }),
                nullable: false,
                default: None,
                unique: true,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
            },
            Column {
                id: "value",
                name: "Content",
                description: "The content of the template",
                column_type: ColumnType::new_scalar(InnerColumnType::Json {
                    max_bytes: Some(templating::LuaKVConstraints::default().max_value_bytes),
                }),
                nullable: false,
                default: None,
                unique: true,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
            },
            module_settings::common_columns::created_at(),
            module_settings::common_columns::last_updated_at(),
        ]),
        title_template: "{name}",
        operations: indexmap::indexmap! {
            OperationType::View => OperationSpecific {
                columns_to_set: indexmap::indexmap! {},
            },
            OperationType::Create => OperationSpecific {
                columns_to_set: indexmap::indexmap! {
                    "created_at" => "{__now}",
                    "last_updated_at" => "{__now}",
                },
            },
            OperationType::Update => OperationSpecific {
                columns_to_set: indexmap::indexmap! {
                    "last_updated_at" => "{__now}",
                },
            },
            OperationType::Delete => OperationSpecific {
                columns_to_set: indexmap::indexmap! {},
            },
        },
        validator: settings_wrap(NoOpValidator {}),
        post_action: settings_wrap(NoOpPostAction {}),
    }
});
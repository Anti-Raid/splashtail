use ar_settings::types::{
    settings_wrap, Column, ColumnSuggestion, ColumnType, HookContext, InnerColumnType,
    InnerColumnTypeStringKind, OperationType, Setting, SettingsError,
};
use ar_settings::types::{
    SettingCreator, SettingDeleter, SettingUpdater, SettingView, SettingsData,
};
use kittycat::perms::Permission;
use splashcore_rs::value::Value;
use std::sync::LazyLock;

pub static GUILD_ROLES: LazyLock<Setting> = LazyLock::new(|| {
    Setting {
        id: "guild_roles".to_string(),
        name: "Server Roles".to_string(),
        description: "Configure/setup server roles which can then have permissions on AntiRaid".to_string(),
        primary_key: "role_id".to_string(),
        columns: settings_wrap(vec![
            ar_settings::common_columns::guild_id("guild_id", "Guild ID", "The Guild ID"),
            Column {
                id: "role_id".to_string(),
                name: "Role ID".to_string(),
                description: "The role ID".to_string(),
                column_type: ColumnType::new_scalar(InnerColumnType::String {
                    kind: InnerColumnTypeStringKind::Role,
                    min_length: None,
                    max_length: Some(64),
                    allowed_values: vec![],
                }),
                nullable: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
            },
            Column {
                id: "perms".to_string(),
                name: "Permissions".to_string(),
                description: "What permissions should the role have".to_string(),
                column_type: ColumnType::new_array(InnerColumnType::String {
                    kind: InnerColumnTypeStringKind::KittycatPermission,
                    min_length: None,
                    max_length: Some(64),
                    allowed_values: vec![],
                }),
                nullable: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
            },
            Column {
                id: "index".to_string(),
                name: "Index".to_string(),
                description: "Where in the role hierarchy should this role be on Anti-Raid for permission purposes. Note that a lower index implies higher on the hierarchy and vice versa".to_string(),
                column_type: ColumnType::new_scalar(InnerColumnType::Integer {}),
                nullable: true,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
            },
            Column {
                id: "display_name".to_string(),
                name: "Display Name".to_string(),
                description: "What should the role be displayed as in API's etc".to_string(),
                column_type: ColumnType::new_scalar(InnerColumnType::String {
                    kind: InnerColumnTypeStringKind::KittycatPermission,
                    min_length: None,
                    max_length: Some(64),
                    allowed_values: vec![],
                }),
                nullable: true,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
            },
            ar_settings::common_columns::created_at(),
            ar_settings::common_columns::created_by(),
            ar_settings::common_columns::last_updated_at(),
            ar_settings::common_columns::last_updated_by(),
        ]),
        title_template: "{index} - {role_id}".to_string(),
        operations: GuildRolesExecutor.into(),
    }
});

#[derive(Clone)]
pub struct GuildRolesExecutor;

#[async_trait::async_trait]
impl SettingView for GuildRolesExecutor {
    async fn view<'a>(
        &self,
        context: HookContext<'a>,
        _filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<Vec<indexmap::IndexMap<String, splashcore_rs::value::Value>>, SettingsError> {
        let rows = sqlx::query!("SELECT role_id, perms, index, display_name, created_at, created_by, last_updated_at, last_updated_by FROM guild_roles WHERE guild_id = $1", context.guild_id.to_string())
        .fetch_all(&context.data.pool)
        .await
        .map_err(|e| SettingsError::Generic {
            message: format!("Error while fetching guild roles: {}", e),
            src: "GuildRolesExecutor".to_string(),
            typ: "value_error".to_string(),
        })?;

        let mut result = vec![];

        for row in rows {
            let map = indexmap::indexmap! {
                "guild_id".to_string() => Value::String(context.guild_id.to_string()),
                "role_id".to_string() => Value::String(row.role_id),
                "perms".to_string() => Value::List(row.perms.iter().map(|x| Value::String(x.to_string())).collect()),
                "index".to_string() => Value::Integer(row.index.into()),
                "display_name".to_string() => Value::String(row.display_name),
                "created_at".to_string() => Value::TimestampTz(row.created_at),
                "created_by".to_string() => Value::String(row.created_by),
                "last_updated_at".to_string() => Value::TimestampTz(row.last_updated_at),
                "last_updated_by".to_string() => Value::String(row.last_updated_by),
            };

            result.push(map);
        }

        Ok(result) // TODO: Implement
    }
}

#[async_trait::async_trait]
impl SettingCreator for GuildRolesExecutor {
    async fn create<'a>(
        &self,
        ctx: HookContext<'a>,
        entry: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<indexmap::IndexMap<String, splashcore_rs::value::Value>, SettingsError> {
        let res = self
            .base_verify_checks(&ctx, &entry, OperationType::Create)
            .await?;

        let count = sqlx::query!(
            "SELECT COUNT(*) FROM guild_roles WHERE guild_id = $1 AND role_id = $2",
            ctx.guild_id.to_string(),
            res.role_id.to_string()
        )
        .fetch_one(&ctx.data.pool)
        .await
        .map_err(|e| SettingsError::Generic {
            message: format!("Failed to check if role exists: {:?}", e),
            src: "GuildRolesExecutor->create".to_string(),
            typ: "internal".to_string(),
        })?
        .count
        .unwrap_or_default();

        if count > 0 {
            return Err(SettingsError::Generic {
                message: "Role already exists".to_string(),
                src: "GuildRolesExecutor->create".to_string(),
                typ: "internal".to_string(),
            });
        }

        sqlx::query!(
            "INSERT INTO guild_roles (guild_id, role_id, perms, index, display_name, created_by, last_updated_by) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            ctx.guild_id.to_string(),
            res.role_id.to_string(),
            &res.perms,
            res.index,
            res.display_name,
            ctx.author.to_string(),
            ctx.author.to_string()
        )
        .execute(&ctx.data.pool)
        .await
        .map_err(|e| SettingsError::Generic {
            message: format!("Failed to insert role: {:?}", e),
            src: "GuildRolesExecutor->create".to_string(),
            typ: "internal".to_string(),
        })?;

        sqlx::query!(
            "UPDATE guild_members SET needs_perm_rederive = true WHERE guild_id = $1 AND $2 = ANY(roles)",
            ctx.guild_id.to_string(),
            res.role_id.to_string()
        )
        .execute(&ctx.data.pool)
        .await
        .map_err(|e| SettingsError::Generic {
            message: format!("Failed to update guild members cache: {:?}", e),
            src: "GuildRolesExecutor".to_string(),
            typ: "internal".to_string(),
        })?;

        Ok(entry)
    }
}

#[async_trait::async_trait]
impl SettingUpdater for GuildRolesExecutor {
    async fn update<'a>(
        &self,
        ctx: HookContext<'a>,
        entry: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<indexmap::IndexMap<String, splashcore_rs::value::Value>, SettingsError> {
        let res = self
            .base_verify_checks(&ctx, &entry, OperationType::Update)
            .await?;

        sqlx::query!(
            "UPDATE guild_roles SET perms = $1, index = $2, display_name = $3, last_updated_by = $4 WHERE guild_id = $5 AND role_id = $6",
            &res.perms,
            res.index,
            res.display_name,
            ctx.author.to_string(),
            ctx.guild_id.to_string(),
            res.role_id.to_string()
        )
        .execute(&ctx.data.pool)
        .await
        .map_err(|e| SettingsError::Generic {
            message: format!("Failed to update role: {:?}", e),
            src: "GuildRolesExecutor->update".to_string(),
            typ: "internal".to_string(),
        })?;

        sqlx::query!(
            "UPDATE guild_members SET needs_perm_rederive = true WHERE guild_id = $1 AND $2 = ANY(roles)",
            ctx.guild_id.to_string(),
            res.role_id.to_string()
        )
        .execute(&ctx.data.pool)
        .await
        .map_err(|e| SettingsError::Generic {
            message: format!("Failed to update guild members cache: {:?}", e),
            src: "GuildRolesExecutor".to_string(),
            typ: "internal".to_string(),
        })?;

        Ok(entry)
    }
}

#[async_trait::async_trait]
impl SettingDeleter for GuildRolesExecutor {
    async fn delete<'a>(
        &self,
        ctx: HookContext<'a>,
        primary_key: splashcore_rs::value::Value,
    ) -> Result<(), SettingsError> {
        let Some(row) = sqlx::query!("SELECT role_id, perms, index, display_name FROM guild_roles WHERE guild_id = $1 AND role_id = $2", ctx.guild_id.to_string(), primary_key.to_string())
        .fetch_optional(&ctx.data.pool)
        .await
        .map_err(|e| SettingsError::Generic {
            message: format!("Error while fetching roles: {}", e),
            src: "GuildRolesExecutor".to_string(),
            typ: "value_error".to_string(),
        })? else {
            return Err(SettingsError::RowDoesNotExist {
                column_id: "role_id".to_string(),
            });
        };

        let entry = indexmap::indexmap! {
            "guild_id".to_string() => Value::String(ctx.guild_id.to_string()),
            "role_id".to_string() => Value::String(row.role_id),
            "perms".to_string() => Value::List(row.perms.iter().map(|x| Value::String(x.to_string())).collect()),
            "index".to_string() => Value::Integer(row.index.into()),
            "display_name".to_string() => Value::String(row.display_name),
        };

        let res = self
            .base_verify_checks(&ctx, &entry, OperationType::Delete)
            .await?;

        sqlx::query!(
            "DELETE FROM guild_roles WHERE guild_id = $1 AND role_id = $2",
            ctx.guild_id.to_string(),
            res.role_id.to_string()
        )
        .execute(&ctx.data.pool)
        .await
        .map_err(|e| SettingsError::Generic {
            message: format!("Failed to delete role: {:?}", e),
            src: "GuildRolesExecutor->delete".to_string(),
            typ: "internal".to_string(),
        })?;

        sqlx::query!(
            "UPDATE guild_members SET needs_perm_rederive = true WHERE guild_id = $1 AND $2 = ANY(roles)",
            ctx.guild_id.to_string(),
            res.role_id.to_string()
        )
        .execute(&ctx.data.pool)
        .await
        .map_err(|e| SettingsError::Generic {
            message: format!("Failed to update guild members cache: {:?}", e),
            src: "GuildRolesExecutor".to_string(),
            typ: "internal".to_string(),
        })?;

        Ok(())
    }
}

pub struct GreBaseVerifyChecksResult {
    pub role_id: serenity::all::RoleId,
    pub index: i64,
    pub perms: Vec<String>,
    pub display_name: Option<String>,
}

impl GuildRolesExecutor {
    async fn base_verify_checks<'a>(
        &self,
        ctx: &HookContext<'a>,
        state: &indexmap::IndexMap<String, Value>,
        op: OperationType,
    ) -> Result<GreBaseVerifyChecksResult, silverpelt::Error> {
        let parsed_value = if let Some(new_index_val) = state.get("index") {
            match new_index_val {
                Value::Integer(new_index) => Value::Integer(*new_index),
                Value::None => Value::None,
                _ => {
                    return Err(SettingsError::MissingOrInvalidField {
                        field: "index".to_string(),
                        src: "base_verify_checks".to_string(),
                    })
                }
            }
        } else {
            Value::None
        };

        // Get the index to set to
        let new_index = match parsed_value {
            Value::Integer(new_index_val) => new_index_val,
            Value::None => {
                let highest_index_rec = sqlx::query!(
                    "SELECT MAX(index) FROM guild_roles WHERE guild_id = $1",
                    ctx.guild_id.to_string()
                )
                .fetch_one(&ctx.data.pool)
                .await
                .map_err(|e| SettingsError::Generic {
                    message: format!("Failed to get highest index: {:?}", e),
                    src: "base_verify_checks->match parsed_value".to_string(),
                    typ: "internal".to_string(),
                })?
                .max
                .unwrap_or(0);

                let index_i64: i64 = (highest_index_rec + 1).into();

                index_i64
            }
            _ => {
                return Err(SettingsError::MissingOrInvalidField {
                    field: "index".to_string(),
                    src: "base_verify_checks->match parsed_value, _ result".to_string(),
                })
            }
        };

        let Some(Value::String(settings_role_id_str)) = state.get("role_id") else {
            return Err(SettingsError::MissingOrInvalidField {
                field: "role_id".to_string(),
                src: "base_verify_checks".to_string(),
            });
        };

        let settings_role_id = settings_role_id_str
            .parse::<serenity::all::RoleId>()
            .map_err(|e| SettingsError::Generic {
                message: format!(
                    "Failed to parse role id despite already having parsed it: {:?}",
                    e
                ),
                src: "base_verify_checks".to_string(),
                typ: "internal".to_string(),
            })?;

        // Get the new permissions as a Vec<String>
        let Some(Value::List(perms_value)) = state.get("perms") else {
            return Err(SettingsError::MissingOrInvalidField {
                field: "perms".to_string(),
                src: "index->NativeAction [default_pre_checks]".to_string(),
            });
        };

        let mut perms = Vec::with_capacity(perms_value.len());

        for perm in perms_value {
            if let Value::String(perm) = perm {
                perms.push(perm.to_string());
            } else {
                return Err(SettingsError::Generic {
                    message: "Failed to parse permissions".to_string(),
                    src: "NativeAction->index".to_string(),
                    typ: "internal".to_string(),
                });
            }
        }

        let display_name = if let Some(Value::String(display_name)) = state.get("display_name") {
            Some(display_name.to_string())
        } else {
            None
        };

        let guild = sandwich_driver::guild(&ctx.data.cache_http, &ctx.data.reqwest, ctx.guild_id)
            .await
            .map_err(|e| SettingsError::Generic {
                message: format!("Failed to get guild: {:?}", e),
                src: "base_verify_checks".to_string(),
                typ: "internal".to_string(),
            })?;

        // If owner, early return
        if guild.owner_id == ctx.author {
            return Ok(GreBaseVerifyChecksResult {
                index: new_index,
                role_id: settings_role_id,
                perms,
                display_name,
            });
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

        let current_roles = {
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
                src: "base_verify_checks".to_string(),
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

        let author_kittycat_perms = silverpelt::member_permission_calc::get_kittycat_perms(
            &ctx.data.pool,
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
        })?;

        if new_index < lowest_index.into() {
            return Err(SettingsError::Generic {
                message: format!("You do not have permission to edit this role's permissions as the new index would be lower than you: {} < {}", new_index, lowest_index),
                src: "NativeAction->index".to_string(),
                typ: "internal".to_string(),
            });
        }

        match op {
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
                    src: "base_verify_checks".to_string(),
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
                        src: "base_verify_checks".to_string(),
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
                    src: "base_verify_checks".to_string(),
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
                    src: "base_verify_checks".to_string(),
                    typ: "perm_check_failed".to_string(),
                })?;
            }
            _ => {
                return Err(SettingsError::OperationNotSupported { operation: op });
            }
        }

        Ok(GreBaseVerifyChecksResult {
            index: new_index,
            role_id: settings_role_id,
            perms,
            display_name,
        })
    }
}

pub static GUILD_MEMBERS: LazyLock<Setting> = LazyLock::new(|| Setting {
    id: "guild_members".to_string(),
    name: "Server Members".to_string(),
    description: "Manage server members".to_string(),
    primary_key: "user_id".to_string(),
    columns: settings_wrap(vec![
        ar_settings::common_columns::guild_id("guild_id", "Guild ID", "The Guild ID"),
        Column {
            id: "user_id".to_string(),
            name: "User ID".to_string(),
            description: "The user ID. Cannot be updated once set".to_string(),
            column_type: ColumnType::new_scalar(InnerColumnType::String {
                kind: InnerColumnTypeStringKind::User,
                min_length: None,
                max_length: Some(64),
                allowed_values: vec![],
            }),
            nullable: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![OperationType::Update],
            secret: false,
        },
        Column {
            id: "perm_overrides".to_string(),
            name: "Permission Overrides".to_string(),
            description:
                "Any permission overrides the member has. This can and should be edited if needed"
                    .to_string(),
            column_type: ColumnType::new_array(InnerColumnType::String {
                kind: InnerColumnTypeStringKind::KittycatPermission,
                min_length: None,
                max_length: Some(64),
                allowed_values: vec![],
            }),
            nullable: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
        },
        Column {
            id: "public".to_string(),
            name: "Public".to_string(),
            description: "Whether the member is public or not".to_string(),
            column_type: ColumnType::new_scalar(InnerColumnType::Boolean {}),
            nullable: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
        },
        ar_settings::common_columns::created_at(),
    ]),
    title_template: "{user_id}, perm_overrides={perm_overrides}".to_string(),
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
});

pub struct GmeBaseVerifyChecksResult {
    pub user_id: serenity::all::UserId,
    pub perm_overrides: Vec<kittycat::perms::Permission>,
    pub public: bool,
}

#[derive(Clone)]
pub struct GuildMembersExecutor;

impl GuildMembersExecutor {
    async fn get_kittycat_perms_for_user<'a>(
        &self,
        data: &SettingsData,
        conn: &mut sqlx::PgConnection,
        guild_id: serenity::all::GuildId,
        guild_owner_id: serenity::all::UserId,
        user_id: serenity::all::UserId,
    ) -> Result<(Vec<serenity::all::RoleId>, Vec<kittycat::perms::Permission>), SettingsError> {
        let Some(member) =
            sandwich_driver::member_in_guild(&data.cache_http, &data.reqwest, guild_id, user_id)
                .await
                .map_err(|e| SettingsError::Generic {
                    message: format!("Failed to get user {}: {:?}", user_id, e),
                    src: "GuildMembersExecutor".to_string(),
                    typ: "internal".to_string(),
                })?
        else {
            return Ok((Vec::new(), Vec::new()));
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
            src: "GuildMembersExecutor".to_string(),
            typ: "internal".to_string(),
        })?;

        let roles = member
            .roles
            .iter()
            .copied()
            .collect::<Vec<serenity::all::RoleId>>();

        Ok((roles, kittycat_perms))
    }

    async fn verify<'a>(
        &self,
        ctx: &HookContext<'a>,
        state: &indexmap::IndexMap<String, Value>,
        op: OperationType,
    ) -> Result<GmeBaseVerifyChecksResult, silverpelt::Error> {
        // Get the user id as this is required for all operations
        let Some(Value::String(user_id)) = state.get("user_id") else {
            return Err(SettingsError::MissingOrInvalidField {
                field: "user_id".to_string(),
                src: "guildmembers->user_id".to_string(),
            });
        };

        // Parse the user id
        let user_id: serenity::all::UserId =
            user_id.parse().map_err(|e| SettingsError::Generic {
                message: format!("Failed to parse user id: {:?}", e),
                src: "guildmembers->user_id".to_string(),
                typ: "external".to_string(),
            })?;

        let Some(Value::Boolean(public)) = state.get("public") else {
            return Err(SettingsError::MissingOrInvalidField {
                field: "public".to_string(),
                src: "guildmembers->public".to_string(),
            });
        };

        if op == OperationType::Update {
            let current_public = sqlx::query!(
                "SELECT public FROM guild_members WHERE guild_id = $1 AND user_id = $2",
                ctx.guild_id.to_string(),
                user_id.to_string()
            )
            .fetch_one(&ctx.data.pool)
            .await
            .map_err(|e| SettingsError::Generic {
                message: format!("Failed to get current public status: {:?}", e),
                src: "GuildMembersExecutor".to_string(),
                typ: "internal".to_string(),
            })?
            .public;

            if public != current_public && ctx.author != user_id {
                return Err(SettingsError::Generic {
                    message: "Only the user can change their public status".to_string(),
                    src: "guildmembers->public".to_string(),
                    typ: "external".to_string(),
                });
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
            return Ok(GmeBaseVerifyChecksResult {
                user_id,
                perm_overrides,
                public,
            });
        }

        // Get the authors kittycat permissions
        let author_kittycat_perms = match self
            .get_kittycat_perms_for_user(
                &ctx.data,
                &ctx.data.pool,
                ctx.guild_id,
                guild.owner_id,
                ctx.author,
            )
            .await
        {
            Ok((_, author_kittycat_perms)) => author_kittycat_perms,
            Err(e) => {
                return Err(SettingsError::Generic {
                    message: format!("Failed to get author permissions: {:?}", e),
                    src: "GuildMembersExecutor".to_string(),
                    typ: "internal".to_string(),
                })
            }
        };

        // Get the target members current kittycat permissions (if any) as well as their roles (for finding new permissions with overrides taken into account)
        let (target_member_roles, current_user_kittycat_perms) = match self
            .get_kittycat_perms_for_user(
                &ctx.data,
                &ctx.data.pool,
                ctx.guild_id,
                guild.owner_id,
                user_id,
            )
            .await
        {
            Ok((target_member_roles, current_user_kittycat_perms)) => {
                (target_member_roles, current_user_kittycat_perms)
            }
            Err(e) => {
                return Err(SettingsError::Generic {
                    message: format!("Failed to get target member permissions: {:?}", e),
                    src: "GuildMembersExecutor".to_string(),
                    typ: "internal".to_string(),
                })
            }
        };

        // Find new user's permissions with the given perm overrides
        let new_user_kittycat_perms = {
            let roles_str = silverpelt::member_permission_calc::create_roles_list_for_guild(
                &target_member_roles,
                ctx.guild_id,
            );

            let user_positions = silverpelt::member_permission_calc::get_user_positions_from_db(
                &ctx.data.pool,
                ctx.guild_id,
                &roles_str,
            )
            .await
            .map_err(|e| SettingsError::Generic {
                message: format!("Failed to get user positions: {:?}", e),
                src: "GuildMembersExecutor".to_string(),
                typ: "internal".to_string(),
            })?;

            silverpelt::member_permission_calc::rederive_perms_impl(
                ctx.guild_id,
                user_id,
                user_positions,
                perm_overrides,
            )
        };

        // Check permissions
        match op {
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
                    src: "GuildMembersExecutor".to_string(),
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
                    src: "GuildMembersExecutor".to_string(),
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
                    src: "GuildMembersExecutor".to_string(),
                    typ: "perm_check_failed".to_string(),
                })?;
            }
            _ => {
                return Err(SettingsError::OperationNotSupported { operation: op });
            }
        }
    }
}

#[async_trait::async_trait]
impl SettingView for GuildMembersExecutor {
    async fn view<'a>(
        &self,
        context: HookContext<'a>,
        _filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<Vec<indexmap::IndexMap<String, splashcore_rs::value::Value>>, SettingsError> {
        let rows = sqlx::query!("SELECT user_id, perm_overrides, public, created_at FROM guild_members WHERE guild_id = $1", context.guild_id.to_string())
        .fetch_all(&context.data.pool)
        .await
        .map_err(|e| SettingsError::Generic {
            message: format!("Error while fetching guild roles: {}", e),
            src: "GuildRolesExecutor".to_string(),
            typ: "value_error".to_string(),
        })?;

        let mut result = vec![];

        for row in rows {
            let map = indexmap::indexmap! {
                "user_id".to_string() => Value::String(row.user_id),
                "perm_overrides".to_string() => Value::List(row.perm_overrides.iter().map(|x| Value::String(x.to_string())).collect()),
                "public".to_string() => Value::Boolean(row.public),
                "created_at".to_string() => Value::TimestampTz(row.created_at),
            };

            result.push(map);
        }

        Ok(result) // TODO: Implement
    }
}

pub static GUILD_TEMPLATES: LazyLock<Setting> = LazyLock::new(|| {
    Setting {
        id: "guild_templates".to_string(),
        name: "Server Templates".to_string(),
        description: "Configure/Setup Server Templates (Lua/Roblox Luau scripts)".to_string(),
        primary_key: "name".to_string(),
        columns: settings_wrap(vec![
            ar_settings::common_columns::guild_id("guild_id", "Guild ID", "The Guild ID"),
            Column {
                id: "name".to_string(),
                name: "Name".to_string(),
                description: "The name to give to the template".to_string(),
                column_type: ColumnType::new_scalar(InnerColumnType::String {
                    kind: InnerColumnTypeStringKind::Normal,
                    min_length: None,
                    max_length: Some(64),
                    allowed_values: vec![],
                }),
                nullable: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
            },
            Column {
                id: "content".to_string(),
                name: "Content".to_string(),
                description: "The content of the template".to_string(),
                column_type: ColumnType::new_scalar(InnerColumnType::String {
                    kind: InnerColumnTypeStringKind::Textarea {
                        ctx: "template".to_string(),
                    },
                    min_length: None,
                    max_length: None,
                    allowed_values: vec![],
                }),
                nullable: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
            },
            Column {
                id: "events".to_string(),
                name: "Events".to_string(),
                description: "The events that this template can be dispatched on. If empty, this template is never dispatched.".to_string(),
                column_type: ColumnType::new_array(InnerColumnType::String { min_length: None, max_length: None, allowed_values: vec![], kind: InnerColumnTypeStringKind::Normal }),
                nullable: true,
                suggestions: ColumnSuggestion::Static { suggestions: gwevent::core::event_list().to_vec().into_iter().map(|x| x.to_string()).collect() },
                ignored_for: vec![],
                secret: false,
            },
            ar_settings::common_columns::created_at(),
            ar_settings::common_columns::created_by(),
            ar_settings::common_columns::last_updated_at(),
            ar_settings::common_columns::last_updated_by(),
        ]),
        title_template: "{name}".to_string(),
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
        validator: settings_wrap(GuildTemplateValidator {}),
        post_action: settings_wrap(GuildTemplatePostAction {}),
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

        let Some(Value::String(name)) = state.state.get("name") else {
            return Err(SettingsError::MissingOrInvalidField {
                field: "content".to_string(),
                src: "guild_templates->content".to_string(),
            });
        };

        if name.starts_with("$shop/") {
            let (shop_tname, shop_tversion) =
                templating::parse_shop_template(name).map_err(|e| SettingsError::Generic {
                    message: format!("Failed to parse shop template: {:?}", e),
                    src: "guild_templates->name".to_string(),
                    typ: "external".to_string(),
                })?;

            let shop_template = sqlx::query!(
                "SELECT COUNT(*) FROM template_shop WHERE name = $1 AND version = $2",
                shop_tname,
                shop_tversion
            )
            .fetch_one(&ctx.data.pool)
            .await
            .map_err(|e| SettingsError::Generic {
                message: format!("Failed to get shop template: {:?}", e),
                src: "guild_templates->name".to_string(),
                typ: "internal".to_string(),
            })?;

            if shop_template.count.unwrap_or(0) == 0 {
                return Err(SettingsError::Generic {
                    message: "Could not find shop template".to_string(),
                    src: "guild_templates->name".to_string(),
                    typ: "external".to_string(),
                });
            }
        }

        Ok(())
    }
}

pub struct GuildTemplatePostAction;

#[async_trait::async_trait]
impl PostAction for GuildTemplatePostAction {
    async fn post_action<'a>(
        &self,
        context: HookContext<'a>,
        state: &'a mut ar_settings::state::State,
    ) -> Result<(), SettingsError> {
        if context.operation_type == OperationType::View {
            return Ok(());
        }

        // Dispatch a OnStartup event for the template

        // Get template ID
        let Some(Value::String(name)) = state.state.get("name") else {
            return Err(SettingsError::MissingOrInvalidField {
                field: "name".to_string(),
                src: "guild_templates->name".to_string(),
            });
        };

        templating::cache::clear_template_cache(context.guild_id).await;

        silverpelt::ar_event::dispatch_event_to_modules_errflatten(std::sync::Arc::new(
            silverpelt::ar_event::EventHandlerContext {
                guild_id: context.guild_id,
                data: silverpelt::data::Data::get_data(context.data),
                event: silverpelt::ar_event::AntiraidEvent::OnStartup(vec![name.to_string()]),
                serenity_context: context.data.serenity_context.clone(),
            },
        ))
        .await
        .map_err(|e| SettingsError::Generic {
            message: format!("Failed to dispatch OnStartup event: {:?}", e),
            src: "guild_templates->post_action".to_string(),
            typ: "internal".to_string(),
        })?;

        Ok(())
    }
}

pub static GUILD_TEMPLATES_KV: LazyLock<Setting> = LazyLock::new(|| Setting {
    id: "guild_templates_kv".to_string(),
    name: "Server Templates (key-value db)".to_string(),
    description: "Key-value database available to templates on this server".to_string(),
    primary_key: "key".to_string(),
    columns: settings_wrap(vec![
        ar_settings::common_columns::guild_id("guild_id", "Guild ID", "The Guild ID"),
        Column {
            id: "key".to_string(),
            name: "Key".to_string(),
            description: "Key".to_string(),
            column_type: ColumnType::new_scalar(InnerColumnType::String {
                kind: InnerColumnTypeStringKind::Normal,
                min_length: None,
                max_length: Some(templating::LuaKVConstraints::default().max_key_length),
                allowed_values: vec![],
            }),
            nullable: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
        },
        Column {
            id: "value".to_string(),
            name: "Content".to_string(),
            description: "The content of the template".to_string(),
            column_type: ColumnType::new_scalar(InnerColumnType::Json {
                max_bytes: Some(templating::LuaKVConstraints::default().max_value_bytes),
            }),
            nullable: true,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
        },
        ar_settings::common_columns::created_at(),
        ar_settings::common_columns::last_updated_at(),
    ]),
    title_template: "{key}".to_string(),
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
});

pub static GUILD_TEMPLATE_SHOP: LazyLock<Setting> = LazyLock::new(|| {
    Setting {
        id: "template_shop".to_string(),
        name: "Created/Published Templates".to_string(),
        description: "Publish new templates to the shop that can be used by any other server".to_string(),
        primary_key: "id".to_string(),
        columns: settings_wrap(vec![
            Column {
                id: "id".to_string(),
                name: "ID".to_string(),
                description: "The internal ID of the template".to_string(),
                column_type: ColumnType::new_scalar(InnerColumnType::Uuid {}),
                nullable: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![OperationType::Create],
                secret: false,
            },
            Column {
                id: "name".to_string(),
                name: "Name".to_string(),
                description: "The name of the template on the shop. Cannot be updated once set".to_string(),
                column_type: ColumnType::new_scalar(InnerColumnType::String {
                    kind: InnerColumnTypeStringKind::Normal,
                    min_length: None,
                    max_length: Some(64),
                    allowed_values: vec![],
                }),
                nullable: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![OperationType::Update],
                secret: false,
            },
            Column {
                id: "version".to_string(),
                name: "Version".to_string(),
                description: "The version of the template. Cannot be updated once set".to_string(), 
                column_type: ColumnType::new_scalar(InnerColumnType::String {
                    kind: InnerColumnTypeStringKind::Normal,
                    min_length: None,
                    max_length: Some(64),
                    allowed_values: vec![],
                }),
                nullable: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![OperationType::Update],
                secret: false,
            },
            Column {
                id: "description".to_string(),
                name: "Description".to_string(),
                description: "The description of the template".to_string(), 
                column_type: ColumnType::new_scalar(InnerColumnType::String {
                    kind: InnerColumnTypeStringKind::Normal,
                    min_length: None,
                    max_length: Some(4096),
                    allowed_values: vec![],
                }),
                nullable: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
            },
            Column {
                id: "content".to_string(),
                name: "Content".to_string(),
                description: "The content of the template. Cannot be updated once set (use a new version for that)".to_string(),
                column_type: ColumnType::new_scalar(InnerColumnType::String {
                    kind: InnerColumnTypeStringKind::Textarea {
                        ctx: "template".to_string(),
                    },
                    min_length: None,
                    max_length: None,
                    allowed_values: vec![],
                }),
                nullable: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![OperationType::Update],
                secret: false,
            },
            Column {
                id: "type".to_string(),
                name: "Type".to_string(),
                description: "The type of the template".to_string(),
                column_type: ColumnType::new_scalar(InnerColumnType::String {
                    kind: InnerColumnTypeStringKind::Normal,
                    min_length: None,
                    max_length: None,
                    allowed_values: vec!["hook".to_string(), "library".to_string()],
                }),
                nullable: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
            },
            ar_settings::common_columns::guild_id("owner_guild", "Guild ID", "The Guild ID"),
            ar_settings::common_columns::created_at(),
            ar_settings::common_columns::created_by(),
            ar_settings::common_columns::last_updated_at(),
            ar_settings::common_columns::last_updated_by(),
        ]),
        title_template: "{name}".to_string(),
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

pub static GUILD_TEMPLATE_SHOP_PUBLIC_LIST: LazyLock<Setting> = LazyLock::new(|| {
    Setting {
        id: "template_shop_public_list".to_string(),
        name: "Explore the shop!".to_string(),
        description: "Explore other templates published by other servers".to_string(),
        primary_key: "id".to_string(),
        columns: settings_wrap(vec![
            Column {
                id: "id".to_string(),
                name: "ID".to_string(),
                description: "The internal ID of the template".to_string(),
                column_type: ColumnType::new_scalar(InnerColumnType::Uuid {}),
                nullable: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
            },
            Column {
                id: "name".to_string(),
                name: "Name".to_string(),
                description: "The name of the template on the shop. Cannot be updated once set".to_string(),
                column_type: ColumnType::new_scalar(InnerColumnType::String {
                    kind: InnerColumnTypeStringKind::Normal,
                    min_length: None,
                    max_length: Some(64),
                    allowed_values: vec![],
                }),
                nullable: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![OperationType::Update],
                secret: false,
            },
            Column {
                id: "version".to_string(),
                name: "Version".to_string(),
                description: "The version of the template. Cannot be updated once set".to_string(), 
                column_type: ColumnType::new_scalar(InnerColumnType::String {
                    kind: InnerColumnTypeStringKind::Normal,
                    min_length: None,
                    max_length: Some(64),
                    allowed_values: vec![],
                }),
                nullable: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![OperationType::Update],
                secret: false,
            },
            Column {
                id: "description".to_string(),
                name: "Description".to_string(),
                description: "The description of the template".to_string(), 
                column_type: ColumnType::new_scalar(InnerColumnType::String {
                    kind: InnerColumnTypeStringKind::Normal,
                    min_length: None,
                    max_length: Some(4096),
                    allowed_values: vec![],
                }),
                nullable: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
            },
            Column {
                id: "content".to_string(),
                name: "Content".to_string(),
                description: "The content of the template. Cannot be updated once set (use a new version for that)".to_string(),
                column_type: ColumnType::new_scalar(InnerColumnType::String {
                    kind: InnerColumnTypeStringKind::Textarea {
                        ctx: "template".to_string(),
                    },
                    min_length: None,
                    max_length: None,
                    allowed_values: vec![],
                }),
                nullable: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![OperationType::Update, OperationType::View],
                secret: false,
            },
            Column {
                id: "type".to_string(),
                name: "Type".to_string(),
                description: "The type of the template".to_string(),
                column_type: ColumnType::new_scalar(InnerColumnType::String {
                    kind: InnerColumnTypeStringKind::Normal,
                    min_length: None,
                    max_length: None,
                    allowed_values: vec!["hook".to_string(), "library".to_string()],
                }),
                nullable: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
            },
            ar_settings::common_columns::guild_id("owner_guild", "Guild ID", "The Guild ID"),
            ar_settings::common_columns::created_at(),
            ar_settings::common_columns::created_by(),
            ar_settings::common_columns::last_updated_at(),
            ar_settings::common_columns::last_updated_by(),
        ]),
        title_template: "{name}".to_string(),
        operations: indexmap::indexmap! {
            OperationType::View => OperationSpecific {
                columns_to_set: indexmap::indexmap! {},
            },
        },
        validator: settings_wrap(NoOpValidator {}),
        post_action: settings_wrap(NoOpPostAction {}),
    }
});

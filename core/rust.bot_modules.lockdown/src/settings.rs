use async_trait::async_trait;
use ar_settings::types::{
        settings_wrap, Column, ColumnSuggestion, ColumnType, HookContext, InnerColumnType, InnerColumnTypeStringKind, OperationType, Setting, SettingCreator, SettingDeleter, SettingOperations, SettingUpdater, SettingView, SettingsError
    };
use splashcore_rs::value::Value;
use std::sync::LazyLock;

async fn check_perms<'a>(
    ctx: &HookContext<'a>,
    perm: &kittycat::perms::Permission,
) -> Result<(), SettingsError> {
    let res = permission_checks::member_has_kittycat_perm(
        ctx.guild_id,
        ctx.author,
        &ctx.data.pool,
        &ctx.data.serenity_context,
        &ctx.data.reqwest,
        &None,
        perm,
        permission_checks::CheckCommandOptions::default(),
    )
    .await;

    if res.is_ok() {
        return Ok(());
    }

    Err(SettingsError::PermissionError { result: res })
}

pub static LOCKDOWN_SETTINGS: LazyLock<Setting> = LazyLock::new(|| {
    Setting {
        id: "lockdown_guilds".to_string(),
        name: "Lockdown Settings".to_string(),
        description: "Setup standard lockdown settings for a server".to_string(),
        primary_key: "guild_id".to_string(),
        columns: settings_wrap(vec![
            ar_settings::common_columns::guild_id(
                "guild_id",
                "Guild ID",
                "Guild ID of the server in question",
            ),
            Column {
                id: "member_roles".to_string(),
                name: "Member Roles".to_string(),
                description: "Which roles to use as member roles for the purpose of lockdown. These roles will be explicitly modified during lockdown".to_string(),
                column_type: ColumnType::new_array(InnerColumnType::String {
                    kind: InnerColumnTypeStringKind::Role {},
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
                id: "require_correct_layout".to_string(),
                name: "Require Correct Layout".to_string(),
                description: "Whether or not a lockdown can proceed even without correct critical role permissions. May lead to partial lockdowns if disabled".to_string(),
                column_type: ColumnType::new_scalar(InnerColumnType::Boolean {}),
                nullable: false,
                suggestions: ColumnSuggestion::None {},
                ignored_for: vec![],
                secret: false,
            },
            ar_settings::common_columns::created_at(),
            ar_settings::common_columns::created_by(),
            ar_settings::common_columns::last_updated_at(),
            ar_settings::common_columns::last_updated_by(),
        ]),
        title_template: "Lockdown Settings".to_string(),
        operations: LockdownSettingsExecutor.into(),
    }
});

#[derive(Clone)]
pub struct LockdownSettingsExecutor;

#[async_trait]
impl SettingView for LockdownSettingsExecutor {
    async fn view<'a>(
        &self,
        context: HookContext<'a>,
        _filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<Vec<indexmap::IndexMap<String, splashcore_rs::value::Value>>, SettingsError> {
        check_perms(&context, &"lockdown_settings.view".into()).await?;

        let rows = sqlx::query!("SELECT member_roles, require_correct_layout, created_at, created_by, last_updated_at, last_updated_by FROM lockdown__guilds WHERE guild_id = $1", context.guild_id.to_string())
            .fetch_all(&context.data.pool)
            .await
            .map_err(|e| SettingsError::Generic {
                message: format!("Error while fetching lockdowns: {}", e),
                src: "lockdown_view".to_string(),
                typ: "value_error".to_string(),
            })?;

        let mut result = vec![];

        for row in rows {
            let map = indexmap::indexmap! {
                "guild_id".to_string() => Value::String(context.guild_id.to_string()),
                "member_roles".to_string() => Value::List(row.member_roles.into_iter().map(|s| Value::String(s)).collect()),
                "require_correct_layout".to_string() => Value::Boolean(row.require_correct_layout),
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

#[async_trait]
impl SettingCreator for LockdownSettingsExecutor {
    async fn create<'a>(
        &self,
        context: HookContext<'a>,
        entry: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<indexmap::IndexMap<String, splashcore_rs::value::Value>, SettingsError> {
        check_perms(&context, &"lockdown_settings.create".into()).await?;

        let Some(splashcore_rs::value::Value::List(member_roles)) = entry.get("member_roles") else {
            return Err(SettingsError::MissingOrInvalidField {
                field: "member_roles".to_string(),
                src: "lockdown_create_entry".to_string(),
            });
        };

        let member_roles: Vec<String> = member_roles.iter().map(|v| match v {
            Value::String(s) => Ok(s.clone()),
            _ => Err(SettingsError::Generic {
                message: "Invalid member role".to_string(),
                src: "lockdown_create_entry".to_string(),
                typ: "value_error".to_string(),
            }),
        }).collect::<Result<Vec<String>, SettingsError>>()?;
        
        let Some(splashcore_rs::value::Value::Boolean(require_correct_layout)) = entry.get("require_correct_layout") else {
            return Err(SettingsError::MissingOrInvalidField {
                field: "require_correct_layout".to_string(),
                src: "lockdown_create_entry".to_string(),
            });
        };

        sqlx::query!(
            "INSERT INTO lockdown__guilds (guild_id, member_roles, require_correct_layout, created_at, created_by, last_updated_at, last_updated_by) VALUES ($1, $2, $3, NOW(), $4, NOW(), $5)",
            context.guild_id.to_string(),
            &member_roles,
            require_correct_layout,
            context.author.to_string(),
            context.author.to_string(),
        )
        .execute(&context.data.pool)
        .await
        .map_err(|e| SettingsError::Generic {
            message: format!("Error while creating lockdown settings: {}", e),
            src: "lockdown_create_entry".to_string(),
            typ: "value_error".to_string(),
        })?;

        Ok(entry)
    }
}

#[async_trait]
impl SettingUpdater for LockdownSettingsExecutor {
    async fn update<'a>(
        &self,
        context: HookContext<'a>,
        entry: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<indexmap::IndexMap<String, splashcore_rs::value::Value>, SettingsError> {
        check_perms(&context, &"lockdown_settings.uodate".into()).await?;

        let Some(splashcore_rs::value::Value::List(member_roles)) = entry.get("member_roles") else {
            return Err(SettingsError::MissingOrInvalidField {
                field: "member_roles".to_string(),
                src: "lockdown_create_entry".to_string(),
            });
        };

        let member_roles: Vec<String> = member_roles.iter().map(|v| match v {
            Value::String(s) => Ok(s.clone()),
            _ => Err(SettingsError::Generic {
                message: "Invalid member role".to_string(),
                src: "lockdown_create_entry".to_string(),
                typ: "value_error".to_string(),
            }),
        }).collect::<Result<Vec<String>, SettingsError>>()?;
        
        let Some(splashcore_rs::value::Value::Boolean(require_correct_layout)) = entry.get("require_correct_layout") else {
            return Err(SettingsError::MissingOrInvalidField {
                field: "require_correct_layout".to_string(),
                src: "lockdown_create_entry".to_string(),
            });
        };

        let count = sqlx::query!(
            "SELECT COUNT(*) FROM lockdown__guilds WHERE guild_id = $1",
            context.guild_id.to_string(),
        )
        .fetch_one(&context.data.pool)
        .await
        .map_err(|e| SettingsError::Generic {
            message: format!("Error while updating lockdown settings: {}", e),
            src: "lockdown_create_entry".to_string(),
            typ: "value_error".to_string(),
        })?;

        if count.count.unwrap_or(0) == 0 {
            return Err(SettingsError::RowDoesNotExist {
                column_id: "guild_id".to_string(),
            });
        }

        sqlx::query!(
            "UPDATE lockdown__guilds SET member_roles = $2, require_correct_layout = $3, last_updated_at = NOW(), last_updated_by = $4 WHERE guild_id = $1",
            context.guild_id.to_string(),
            &member_roles,
            require_correct_layout,
            context.author.to_string(),
        )
        .execute(&context.data.pool)
        .await
        .map_err(|e| SettingsError::Generic {
            message: format!("Error while creating lockdown settings: {}", e),
            src: "lockdown_create_entry".to_string(),
            typ: "value_error".to_string(),
        })?;

        Ok(entry)
    }
}

#[async_trait]
impl SettingDeleter for LockdownSettingsExecutor {
    async fn delete<'a>(
        &self,
        context: HookContext<'a>,
        _primary_key: splashcore_rs::value::Value,
    ) -> Result<(), SettingsError> {
        check_perms(&context, &"lockdown_settings.delete".into()).await?;

        sqlx::query!("DELETE FROM lockdown__guilds WHERE guild_id = $1", context.guild_id.to_string())
            .execute(&context.data.pool)
            .await
            .map_err(|e| SettingsError::Generic {
                message: format!("Error while deleting lockdown settings: {}", e),
                src: "lockdown_delete".to_string(),
                typ: "value_error".to_string(),
            })?;

        Ok(())
    }
}

pub static LOCKDOWNS: LazyLock<Setting> = LazyLock::new(|| Setting {
    id: "lockdowns".to_string(),
    name: "Lockdowns".to_string(),
    description: "Lockdowns".to_string(),
    primary_key: "id".to_string(),
    columns: settings_wrap(vec![
        Column {
            id: "id".to_string(),
            name: "ID".to_string(),
            description: "The ID of the lockdown".to_string(),
            column_type: ColumnType::new_scalar(InnerColumnType::Uuid {}),
            nullable: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![OperationType::Create],
            secret: false,
        },
        ar_settings::common_columns::guild_id(
            "guild_id",
            "Guild ID",
            "The Guild ID referring to this lockdown",
        ),
        Column {
            id: "type".to_string(),
            name: "Type".to_string(),
            description: "The type of the lockdown.".to_string(),
            column_type: ColumnType::new_scalar(InnerColumnType::String {
                kind: InnerColumnTypeStringKind::Normal {},
                min_length: Some(1),
                max_length: Some(256),
                allowed_values: vec![],
            }),
            nullable: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
        },
        Column {
            id: "data".to_string(),
            name: "Data".to_string(),
            description: "The data stored of the lockdown.".to_string(),
            column_type: ColumnType::new_scalar(InnerColumnType::Json { max_bytes: None }),
            nullable: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![OperationType::Create, OperationType::Update],
            secret: false,
        },
        Column {
            id: "reason".to_string(),
            name: "Reason".to_string(),
            description: "The reason for starting the lockdown.".to_string(),
            column_type: ColumnType::new_scalar(InnerColumnType::String {
                kind: InnerColumnTypeStringKind::Normal {},
                min_length: Some(1),
                max_length: Some(256),
                allowed_values: vec![],
            }),
            nullable: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
        },
        ar_settings::common_columns::created_at(),
    ]),
    title_template: "Reason: {reason}".to_string(),
    operations: SettingOperations::to_view_create_delete_op(LockdownExecutor),
});

#[derive(Clone)]
pub struct LockdownExecutor;

#[async_trait]
impl SettingView for LockdownExecutor {
    async fn view<'a>(
        &self,
        context: HookContext<'a>,
        _filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<Vec<indexmap::IndexMap<String, splashcore_rs::value::Value>>, SettingsError> {
        check_perms(&context, &"lockdowns.view".into()).await?;

        let rows = sqlx::query!("SELECT id, data, type, reason, created_at FROM lockdown__guild_lockdowns WHERE guild_id = $1", context.guild_id.to_string())
            .fetch_all(&context.data.pool)
            .await
            .map_err(|e| SettingsError::Generic {
                message: format!("Error while fetching lockdowns: {}", e),
                src: "lockdown_view".to_string(),
                typ: "value_error".to_string(),
            })?;

        let mut result = vec![];

        for row in rows {
            let map = indexmap::indexmap! {
                "id".to_string() => Value::Uuid(row.id),
                "guild_id".to_string() => Value::String(context.guild_id.to_string()),
                "data".to_string() => Value::Json(row.data),
                "type".to_string() => Value::String(row.r#type),
                "reason".to_string() => Value::String(row.reason),
                "created_at".to_string() => Value::TimestampTz(row.created_at),
            };

            result.push(map);
        }
        
        Ok(result) // TODO: Implement
    }
}

#[async_trait]
impl SettingCreator for LockdownExecutor {
    async fn create<'a>(
        &self,
        context: HookContext<'a>,
        entry: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<indexmap::IndexMap<String, splashcore_rs::value::Value>, SettingsError> {
        check_perms(&context, &"lockdowns.create".into()).await?;

        let silverpelt_cache = silverpelt::data::Data::silverpelt_cache(&context.data);
        if !silverpelt::module_config::is_module_enabled(&silverpelt_cache, &context.data.pool, context.guild_id, "lockdown")
        .await
        .map_err(|e| SettingsError::Generic {
            message: format!("Error while checking if module is enabled: {}", e),
            src: "lockdown_create".to_string(),
            typ: "value_error".to_string(),
        })? {
        return Err(SettingsError::Generic {
            message: "Lockdown module is not enabled".to_string(),
            src: "lockdown_create".to_string(),
            typ: "value_error".to_string(),
        });
    }
    
    let Some(splashcore_rs::value::Value::String(typ)) = entry.get("type") else {
        return Err(SettingsError::MissingOrInvalidField {
            field: "type".to_string(),
            src: "lockdown_create_entry".to_string(),
        });
    };

    let Some(splashcore_rs::value::Value::String(reason)) = entry.get("reason") else {
        return Err(SettingsError::MissingOrInvalidField {
            field: "reason".to_string(),
            src: "lockdown_create_entry".to_string(),
        });
    };

    // Get the current lockdown set
    let mut lockdowns = lockdowns::LockdownSet::guild(context.guild_id, &context.data.pool)
        .await
        .map_err(|e| SettingsError::Generic {
            message: format!("Error while fetching lockdown set: {}", e),
            src: "lockdown_create_entry".to_string(),
            typ: "value_error".to_string(),
        })?;

    // Create the lockdown
    let lockdown_type =
        lockdowns::from_lockdown_mode_string(typ).map_err(|_| SettingsError::Generic {
            message: format!(
                "Invalid lockdown mode: {}.\n\nTIP: The following lockdown modes are supported: {}", 
                typ, 
                {
                    let mut supported_lockdown_modes = String::new();

                    for mode in lockdowns::CREATE_LOCKDOWN_MODES.iter() {
                        let creator = mode.value();
                        supported_lockdown_modes.push_str(&format!("\n- {}", creator.syntax()));
                    }

                    supported_lockdown_modes
                }
            ),
            src: "lockdown_create_entry".to_string(),
            typ: "value_error".to_string(),
        })?;

    let lockdown_data = lockdowns::LockdownData {
        cache_http: context.data.cache_http.clone(),
        pool: context.data.pool.clone(),
        reqwest: context.data.reqwest.clone(),
        object_store: context.data.object_store.clone(),
    };

    lockdowns
        .easy_apply(lockdown_type, &lockdown_data, reason)
        .await
        .map_err(|e| SettingsError::Generic {
            message: format!("Error while applying lockdown: {}", e),
            src: "lockdown_create_entry".to_string(),
            typ: "value_error".to_string(),
        })?;

    let created_lockdown =
        lockdowns
            .lockdowns
            .last()
            .ok_or_else(|| SettingsError::Generic {
                message: "No lockdowns created".to_string(),
                src: "lockdown_create_entry".to_string(),
                typ: "value_error".to_string(),
            })?;
        
        Ok(created_lockdown.to_map())
    }
}

#[async_trait]
impl SettingDeleter for LockdownExecutor {
    async fn delete<'a>(
        &self,
        context: HookContext<'a>,
        primary_key: splashcore_rs::value::Value,
    ) -> Result<(), SettingsError> {
        check_perms(&context, &"lockdowns.delete".into()).await?;
        
        let silverpelt_cache = silverpelt::data::Data::silverpelt_cache(&context.data);
        if !silverpelt::module_config::is_module_enabled(&silverpelt_cache, &context.data.pool, context.guild_id, "lockdown")
        .await
        .map_err(|e| SettingsError::Generic {
            message: format!("Error while checking if module is enabled: {}", e),
            src: "lockdown_create".to_string(),
            typ: "value_error".to_string(),
        })? {
            return Err(SettingsError::Generic {
                message: "Lockdown module is not enabled".to_string(),
                src: "lockdown_create".to_string(),
                typ: "value_error".to_string(),
            });
        }
        
        let primary_key = match primary_key {
            Value::Uuid(primary_key) => primary_key,
            Value::String(primary_key) => primary_key
                .parse()
                .map_err(|_| SettingsError::Generic {
                    message: format!("Invalid primary key: {}", primary_key),
                    src: "lockdown_delete".to_string(),
                    typ: "value_error".to_string(),
                })?,
            _ => {
                return Err(SettingsError::Generic {
                    message: "Primary key must be a string or UUID".to_string(),
                    src: "lockdown_delete".to_string(),
                    typ: "value_error".to_string(),
                })
            }
        };

        // Get the current lockdown set
        let mut lockdowns = lockdowns::LockdownSet::guild(context.guild_id, &context.data.pool)
            .await
            .map_err(|e| SettingsError::Generic {
                message: format!("Error while fetching lockdown set: {}", e),
                src: "lockdown_delete_matching_entries".to_string(),
                typ: "value_error".to_string(),
            })?;

        let lockdown_data = lockdowns::LockdownData {
            cache_http: context.data.cache_http.clone(),
            pool: context.data.pool.clone(),
            reqwest: context.data.reqwest.clone(),
            object_store: context.data.object_store.clone(),
        };        

        // Remove the lockdown
        lockdowns
            .easy_remove(primary_key, &lockdown_data)
            .await
            .map_err(|e| SettingsError::Generic {
                message: format!("Error while removing lockdown: {}", e),
                src: "lockdown_delete_matching_entries".to_string(),
                typ: "value_error".to_string(),
            })?;

        Ok(()) // TODO: Implement
    }
}

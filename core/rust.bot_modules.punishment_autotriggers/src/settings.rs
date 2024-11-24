use ar_settings::types::{
    settings_wrap, Column, ColumnSuggestion, ColumnType, HookContext, InnerColumnType,
    InnerColumnTypeStringKind, OperationType, Setting, SettingCreator, SettingDeleter,
    SettingUpdater, SettingView, SettingsError,
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

pub static AUTOTRIGGERS: LazyLock<Setting> = LazyLock::new(|| Setting {
    id: "punishment_autotriggers".to_string(),
    name: "Punishment Autotriggers".to_string(),
    description: "All punishments that should be trigggred automatically based on stings"
        .to_string(),
    primary_key: "id".to_string(),
    columns: settings_wrap(vec![
        Column {
            id: "id".to_string(),
            name: "ID".to_string(),
            description: "The ID used to refer to this autotrigger".to_string(),
            column_type: ColumnType::new_scalar(InnerColumnType::Uuid {}),
            nullable: true,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![OperationType::Create],
            secret: false,
        },
        ar_settings::common_columns::guild_id(
            "guild_id",
            "Guild ID",
            "Guild ID of the server in question",
        ),
        Column {
            id: "stings".to_string(),
            name: "Stings".to_string(),
            description: "The number of stings required to trigger the action".to_string(),
            column_type: ColumnType::new_scalar(InnerColumnType::Integer {}),
            nullable: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
        },
        Column {
            id: "action".to_string(),
            name: "Action".to_string(),
            description: "The action to trigger when the stings are reached".to_string(),
            column_type: ColumnType::new_scalar(InnerColumnType::String {
                min_length: Some(1),
                max_length: Some(100),
                allowed_values: vec![],
                kind: InnerColumnTypeStringKind::Normal,
            }),
            nullable: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
        },
        Column {
            id: "modifiers".to_string(),
            name: "Modifiers".to_string(),
            description: "Any modifiers to the action".to_string(),
            column_type: ColumnType::new_array(InnerColumnType::String {
                min_length: Some(1),
                max_length: Some(100),
                allowed_values: vec![],
                kind: InnerColumnTypeStringKind::Modifier,
            }),
            nullable: false,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
        },
        Column {
            id: "duration".to_string(),
            name: "Duration".to_string(),
            description: "The duration of the punishment to apply/use".to_string(),
            column_type: ColumnType::new_scalar(InnerColumnType::Interval {}),
            nullable: true,
            suggestions: ColumnSuggestion::None {},
            ignored_for: vec![],
            secret: false,
        },
        ar_settings::common_columns::created_at(),
        ar_settings::common_columns::created_by(),
    ]),
    title_template: "At {stings} stings, {action} will be triggered".to_string(),
    operations: AutotriggerExecutor.into(),
});

#[derive(Clone)]
pub struct AutotriggerExecutor;

#[async_trait::async_trait]
impl SettingView for AutotriggerExecutor {
    async fn view<'a>(
        &self,
        context: HookContext<'a>,
        _filters: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<Vec<indexmap::IndexMap<String, splashcore_rs::value::Value>>, SettingsError> {
        check_perms(&context, &"punishment_autotriggers.view".into()).await?;

        let rows = sqlx::query!("SELECT id, stings, action, modifiers, duration, created_at, created_by FROM punishment_autotriggers__autotriggers WHERE guild_id = $1", context.guild_id.to_string())
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
                "stings".to_string() => Value::Integer(row.stings.into()),
                "action".to_string() => Value::String(row.action),
                "modifiers".to_string() => Value::List(row.modifiers.into_iter().map(Value::String).collect()),
                "duration".to_string() => row.duration.map(|x| Value::Interval(splashcore_rs::utils::pg_interval_to_chrono_duration(x))).unwrap_or(Value::None),
                "created_at".to_string() => Value::TimestampTz(row.created_at),
                "created_by".to_string() => Value::String(row.created_by),
            };

            result.push(map);
        }

        Ok(result) // TODO: Implement
    }
}

#[async_trait::async_trait]
impl SettingCreator for AutotriggerExecutor {
    async fn create<'a>(
        &self,
        context: HookContext<'a>,
        state: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<indexmap::IndexMap<String, splashcore_rs::value::Value>, SettingsError> {
        check_perms(&context, &"punishment_autotriggers.create".into()).await?;

        let Some(splashcore_rs::value::Value::Integer(stings)) = state.get("stings") else {
            return Err(SettingsError::Generic {
                message: "Stings is required".to_string(),
                src: "AutotriggerValidator".to_string(),
                typ: "external".to_string(),
            });
        };

        let Some(splashcore_rs::value::Value::String(action)) = state.get("action") else {
            return Err(SettingsError::Generic {
                message: "Action is required".to_string(),
                src: "AutotriggerValidator".to_string(),
                typ: "external".to_string(),
            });
        };

        if action.is_empty() {
            return Err(SettingsError::Generic {
                message: "Action cannot be empty".to_string(),
                src: "AutotriggerValidator".to_string(),
                typ: "external".to_string(),
            });
        }

        if !["ban", "kick", "timeout", "removeallroles"].contains(&action.as_str()) {
            return Err(SettingsError::Generic {
                message: "Action must be one of ban, kick, timeout and removeallroles".to_string(),
                src: "AutotriggerValidator".to_string(),
                typ: "external".to_string(),
            });
        }

        let Some(splashcore_rs::value::Value::List(modifiers_val)) = state.get("modifiers") else {
            return Err(SettingsError::Generic {
                message: "Modifiers is required".to_string(),
                src: "AutotriggerValidator".to_string(),
                typ: "external".to_string(),
            });
        };

        let mut modifiers = Vec::new();
        for modifier in modifiers_val.iter() {
            match modifier {
                Value::String(s) => {
                    if s.is_empty() {
                        return Err(SettingsError::Generic {
                            message: "Modifier cannot be empty".to_string(),
                            src: "autotrigger_create".to_string(),
                            typ: "value_error".to_string(),
                        });
                    }

                    modifiers.push(s.to_string());
                }
                _ => {
                    return Err(SettingsError::Generic {
                        message: "Modifier must be a string".to_string(),
                        src: "autotrigger_create".to_string(),
                        typ: "value_error".to_string(),
                    })
                }
            }
        }

        let duration = state
            .get("duration")
            .map(|v| match v {
                Value::Interval(i) => Some(i),
                _ => None,
            })
            .flatten();

        let id = sqlx::query!(
            "INSERT INTO punishment_autotriggers__autotriggers (guild_id, stings, action, modifiers, duration, created_at, created_by) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
            context.guild_id.to_string(),
            {
                let stings: i32 = (*stings).try_into().unwrap_or_default();
                stings
            },
            action,
            &modifiers,
            duration.map(|x| splashcore_rs::utils::chrono_duration_to_pg_interval(*x)),
            chrono::Utc::now(),
            context.author.to_string(),
        )
        .fetch_one(&context.data.pool)
        .await
        .map_err(|e| SettingsError::Generic {
            message: format!("Error while creating autotrigger: {}", e),
            src: "autotrigger_create".to_string(),
            typ: "value_error".to_string(),
        })?;

        let mut state = state;
        state.insert("id".to_string(), Value::Uuid(id.id));

        Ok(state)
    }
}

#[async_trait::async_trait]
impl SettingUpdater for AutotriggerExecutor {
    async fn update<'a>(
        &self,
        context: HookContext<'a>,
        state: indexmap::IndexMap<String, splashcore_rs::value::Value>,
    ) -> Result<indexmap::IndexMap<String, splashcore_rs::value::Value>, SettingsError> {
        check_perms(&context, &"punishment_autotriggers.update".into()).await?;

        let mut tx = context
            .data
            .pool
            .begin()
            .await
            .map_err(|e| SettingsError::Generic {
                message: format!("Error while starting transaction: {}", e),
                src: "autotrigger_update".to_string(),
                typ: "value_error".to_string(),
            })?;

        let Some(splashcore_rs::value::Value::Uuid(id)) = state.get("id") else {
            return Err(SettingsError::Generic {
                message: "ID is required".to_string(),
                src: "autotrigger_update".to_string(),
                typ: "value_error".to_string(),
            });
        };

        let count = sqlx::query!(
            "SELECT COUNT(*) FROM punishment_autotriggers__autotriggers WHERE guild_id = $1 AND id = $2",
            context.guild_id.to_string(),
            id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| SettingsError::Generic {
            message: format!("Error while checking if autotrigger exists: {}", e),
            src: "autotrigger_update".to_string(),
            typ: "value_error".to_string(),
        })?;

        if count.count.unwrap_or(0) == 0 {
            return Err(SettingsError::Generic {
                message: "Autotrigger does not exist".to_string(),
                src: "autotrigger_update".to_string(),
                typ: "value_error".to_string(),
            });
        }

        if let Some(splashcore_rs::value::Value::Integer(stings)) = state.get("stings") {
            sqlx::query!(
                "UPDATE punishment_autotriggers__autotriggers SET stings = $1 WHERE id = $2",
                {
                    let stings: i32 = (*stings).try_into().unwrap_or_default();
                    stings
                },
                id
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| SettingsError::Generic {
                message: format!("Error while updating stings: {}", e),
                src: "autotrigger_update".to_string(),
                typ: "value_error".to_string(),
            })?;
        }

        if let Some(splashcore_rs::value::Value::String(action)) = state.get("action") {
            if action.is_empty() {
                return Err(SettingsError::Generic {
                    message: "Action cannot be empty".to_string(),
                    src: "AutotriggerValidator".to_string(),
                    typ: "external".to_string(),
                });
            }

            if !["ban", "kick", "timeout", "removeallroles"].contains(&action.as_str()) {
                return Err(SettingsError::Generic {
                    message: "Action must be one of ban, kick, timeout and removeallroles"
                        .to_string(),
                    src: "AutotriggerValidator".to_string(),
                    typ: "external".to_string(),
                });
            }

            sqlx::query!(
                "UPDATE punishment_autotriggers__autotriggers SET action = $1 WHERE id = $2",
                action,
                id
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| SettingsError::Generic {
                message: format!("Error while updating action: {}", e),
                src: "autotrigger_update".to_string(),
                typ: "value_error".to_string(),
            })?;
        };

        if let Some(splashcore_rs::value::Value::List(modifiers_val)) = state.get("modifiers") {
            let mut modifiers = Vec::new();
            for modifier in modifiers_val.iter() {
                match modifier {
                    Value::String(s) => {
                        if s.is_empty() {
                            return Err(SettingsError::Generic {
                                message: "Modifier cannot be empty".to_string(),
                                src: "autotrigger_create".to_string(),
                                typ: "value_error".to_string(),
                            });
                        }

                        modifiers.push(s.to_string());
                    }
                    _ => {
                        return Err(SettingsError::Generic {
                            message: "Modifier must be a string".to_string(),
                            src: "autotrigger_create".to_string(),
                            typ: "value_error".to_string(),
                        })
                    }
                }
            }

            sqlx::query!(
                "UPDATE punishment_autotriggers__autotriggers SET modifiers = $1 WHERE id = $2",
                &modifiers,
                id
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| SettingsError::Generic {
                message: format!("Error while updating modifiers: {}", e),
                src: "autotrigger_update".to_string(),
                typ: "value_error".to_string(),
            })?;
        }

        if let Some(Value::Interval(duration)) = state.get("duration") {
            sqlx::query!(
                "UPDATE punishment_autotriggers__autotriggers SET duration = $1 WHERE id = $2",
                splashcore_rs::utils::chrono_duration_to_pg_interval(*duration),
                id
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| SettingsError::Generic {
                message: format!("Error while updating duration: {}", e),
                src: "autotrigger_update".to_string(),
                typ: "value_error".to_string(),
            })?;
        }

        Ok(state)
    }
}

#[async_trait::async_trait]
impl SettingDeleter for AutotriggerExecutor {
    async fn delete<'a>(
        &self,
        context: HookContext<'a>,
        primary_key: splashcore_rs::value::Value,
    ) -> Result<(), SettingsError> {
        check_perms(&context, &"punishment_autotriggers.delete".into()).await?;

        let Value::Uuid(id) = primary_key else {
            return Err(SettingsError::Generic {
                message: "ID is required".to_string(),
                src: "autotrigger_delete".to_string(),
                typ: "value_error".to_string(),
            });
        };

        sqlx::query!(
            "DELETE FROM punishment_autotriggers__autotriggers WHERE guild_id = $1 AND id = $2",
            context.guild_id.to_string(),
            id
        )
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

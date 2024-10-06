use crate::types::CanonicalSettingsResult;
use axum::{
    extract::{Path, State},
    Json,
};
use module_settings::{self, canonical_types::CanonicalSettingsError, types::OperationType};
use rust_rpc_server::AppData;
use splashcore_rs::value::Value;

/// Executes an operation on a setting [SettingsOperation]
pub(crate) async fn settings_operation(
    State(AppData {
        data,
        serenity_context,
        cache_http,
        ..
    }): State<AppData>,
    Path((guild_id, user_id)): Path<(serenity::all::GuildId, serenity::all::UserId)>,
    Json(req): Json<crate::types::SettingsOperationRequest>,
) -> Json<crate::types::CanonicalSettingsResult> {
    let op: OperationType = req.op.into();

    // Find the setting
    let Some(module) = data.silverpelt_cache.module_cache.get(&req.module) else {
        return Json(CanonicalSettingsResult::Err {
            error: CanonicalSettingsError::Generic {
                message: "Module not found".to_string(),
                src: "SettingsOperation".to_string(),
                typ: "badRequest".to_string(),
            },
        });
    };

    let config_options = module.config_options(); // Get the config options

    let Some(opt) = config_options.iter().find(|x| x.id == req.setting) else {
        return Json(CanonicalSettingsResult::Err {
            error: CanonicalSettingsError::Generic {
                message: "Setting not found".to_string(),
                src: "SettingsOperation".to_string(),
                typ: "badRequest".to_string(),
            },
        });
    };

    let mut p_fields = indexmap::IndexMap::new();

    // As the order of fields may not be guaranteed, we need to add the fields in the order of the columns
    //
    // We then add the rest of the fields not in columns as well
    for column in opt.columns.iter() {
        if let Some(value) = req.fields.get(column.id) {
            p_fields.insert(column.id.to_string(), Value::from_json(value));
        }
    }

    // Add the rest of the fields
    for (key, value) in req.fields {
        if p_fields.contains_key(&key) {
            continue;
        }

        p_fields.insert(key, Value::from_json(&value));
    }

    if opt.operations.get(&op).is_none() {
        return Json(CanonicalSettingsResult::Err {
            error: CanonicalSettingsError::OperationNotSupported {
                operation: op.into(),
            },
        });
    }

    let perm_res = silverpelt::cmd::check_command(
        &data.silverpelt_cache,
        &opt.get_corresponding_command(op),
        guild_id,
        user_id,
        &data.pool,
        &cache_http,
        &data.reqwest,
        &None,
        silverpelt::cmd::CheckCommandOptions {
            ignore_module_disabled: true,
            ..Default::default()
        },
    )
    .await;

    if !perm_res.is_ok() {
        return Json(CanonicalSettingsResult::PermissionError { res: perm_res });
    }

    match op {
        OperationType::View => {
            match module_settings::cfg::settings_view(
                opt,
                &data.settings_data(serenity_context),
                guild_id,
                user_id,
                p_fields,
            )
            .await
            {
                Ok(res) => Json(CanonicalSettingsResult::Ok {
                    fields: res.into_iter().map(|x| x.into()).collect(),
                }),
                Err(e) => Json(CanonicalSettingsResult::Err { error: e.into() }),
            }
        }
        OperationType::Create => {
            match module_settings::cfg::settings_create(
                opt,
                &data.settings_data(serenity_context),
                guild_id,
                user_id,
                p_fields,
            )
            .await
            {
                Ok(res) => Json(CanonicalSettingsResult::Ok {
                    fields: vec![res.into()],
                }),
                Err(e) => Json(CanonicalSettingsResult::Err { error: e.into() }),
            }
        }
        OperationType::Update => {
            match module_settings::cfg::settings_update(
                opt,
                &data.settings_data(serenity_context),
                guild_id,
                user_id,
                p_fields,
            )
            .await
            {
                Ok(res) => Json(CanonicalSettingsResult::Ok {
                    fields: vec![res.into()],
                }),
                Err(e) => Json(CanonicalSettingsResult::Err { error: e.into() }),
            }
        }
        OperationType::Delete => {
            let Some(pkey) = p_fields.get(opt.primary_key) else {
                return Json(CanonicalSettingsResult::Err {
                    error: CanonicalSettingsError::MissingOrInvalidField {
                        field: opt.primary_key.to_string(),
                        src: "SettingsOperation".to_string(),
                    },
                });
            };

            match module_settings::cfg::settings_delete(
                opt,
                &data.settings_data(serenity_context),
                guild_id,
                user_id,
                pkey.clone(),
            )
            .await
            {
                Ok(res) => Json(CanonicalSettingsResult::Ok {
                    fields: vec![res.into()],
                }),
                Err(e) => Json(CanonicalSettingsResult::Err { error: e.into() }),
            }
        }
    }
}

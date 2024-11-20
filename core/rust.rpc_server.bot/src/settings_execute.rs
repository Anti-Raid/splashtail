use crate::types::CanonicalSettingsResult;
use axum::{
    extract::{Path, State},
    Json,
};
use module_settings::{self, types::OperationType, types::SettingsError};
use rust_rpc_server::AppData;
use splashcore_rs::value::Value;

/// Executes an operation on a setting [SettingsOperation]
pub(crate) async fn settings_operation(
    State(AppData {
        data,
        serenity_context,
        ..
    }): State<AppData>,
    Path((guild_id, user_id)): Path<(serenity::all::GuildId, serenity::all::UserId)>,
    Json(req): Json<crate::types::SettingsOperationRequest>,
) -> Json<crate::types::CanonicalSettingsResult> {
    let op: OperationType = req.op.into();

    // Find the setting
    let Some(setting) = data.silverpelt_cache.settings_cache.get(&req.setting) else {
        return Json(CanonicalSettingsResult::Err {
            error: SettingsError::MissingOrInvalidField {
                field: "$opt".to_string(),
                src: "rpc".to_string(),
            },
        });
    };

    let mut p_fields = indexmap::IndexMap::new();

    // As the order of fields may not be guaranteed, we need to add the fields in the order of the columns
    //
    // We then add the rest of the fields not in columns as well
    for column in setting.columns.iter() {
        if let Some(value) = req.fields.get(&column.id) {
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

    if !setting.supported_operations.contains(&op) {
        return Json(CanonicalSettingsResult::Err {
            error: SettingsError::OperationNotSupported { operation: op },
        });
    }

    match op {
        OperationType::View => {
            match module_settings::cfg::settings_view(
                &setting,
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
        OperationType::Save => {
            match module_settings::cfg::settings_save(
                &setting,
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
            let Some(pkey) = p_fields.get(&setting.primary_key) else {
                return Json(CanonicalSettingsResult::Err {
                    error: SettingsError::MissingOrInvalidField {
                        field: setting.primary_key.to_string(),
                        src: "SettingsOperation".to_string(),
                    },
                });
            };

            match module_settings::cfg::settings_delete(
                &setting,
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

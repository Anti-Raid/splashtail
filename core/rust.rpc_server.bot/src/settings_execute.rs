use crate::types::CanonicalSettingsResult;
use axum::{
    extract::{Path, State},
    Json,
};
use module_settings::{self, types::OperationType, types::SettingsError};
use rust_rpc_server::AppData;

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
                req.fields,
            )
            .await
            {
                Ok(res) => Json(CanonicalSettingsResult::Ok { fields: res }),
                Err(e) => Json(CanonicalSettingsResult::Err { error: e.into() }),
            }
        }
        OperationType::Save => {
            match module_settings::cfg::settings_save(
                &setting,
                &data.settings_data(serenity_context),
                guild_id,
                user_id,
                req.fields,
            )
            .await
            {
                Ok(res) => Json(CanonicalSettingsResult::Ok { fields: vec![res] }),
                Err(e) => Json(CanonicalSettingsResult::Err { error: e.into() }),
            }
        }
        OperationType::Delete => {
            let Some(pkey) = req.fields.get(&setting.primary_key) else {
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

use crate::lang_lua::state;
use governor::clock::Clock;
use mlua::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// An kv executor is used to execute key-value ops from Lua
/// templates
pub struct KvExecutor {
    template_data: Arc<state::TemplateData>,
    guild_id: serenity::all::GuildId,
    pool: sqlx::PgPool,
    kv_constraints: state::LuaKVConstraints,
    ratelimits: Arc<state::LuaKvRatelimit>,
}

/// Represents a full record complete with metadata
#[derive(Serialize, Deserialize)]
pub struct KvRecord {
    pub key: String,
    pub value: serde_json::Value,
    pub exists: bool,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl KvExecutor {
    pub fn base_check(&self, action: String) -> Result<(), crate::Error> {
        if self.template_data.pragma.kv_ops.is_empty() {
            return Err("Key-value operations are disabled on this template".into());
        }

        // Check global ratelimits
        for global_lim in self.ratelimits.global.iter() {
            match global_lim.check_key(&()) {
                Ok(()) => continue,
                Err(wait) => {
                    return Err(format!(
                        "Global ratelimit hit for action '{}', wait time: {:?}",
                        action,
                        wait.wait_time_from(self.ratelimits.clock.now())
                    )
                    .into());
                }
            };
        }

        // Check per bucket ratelimits
        if let Some(per_bucket) = self.ratelimits.per_bucket.get(&action) {
            for lim in per_bucket.iter() {
                match lim.check_key(&()) {
                    Ok(()) => continue,
                    Err(wait) => {
                        return Err(format!(
                            "Per bucket ratelimit hit for action '{}', wait time: {:?}",
                            action,
                            wait.wait_time_from(self.ratelimits.clock.now())
                        )
                        .into());
                    }
                };
            }
        }

        Ok(())
    }
}

impl LuaUserData for KvExecutor {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_async_method("get", |lua, this, key: String| async move {
            this.base_check("get".to_string())
                .map_err(LuaError::external)?;

            if !this.template_data.pragma.kv_ops.contains(&"*".to_string())
                && this
                    .template_data
                    .pragma
                    .kv_ops
                    .contains(&format!("get:{}", key))
                && this.template_data.pragma.kv_ops.contains(&"get:*".to_string())
                && this.template_data.pragma.kv_ops.contains(&key)
            {
                return Err(LuaError::external(
                    format!("Operation `get` not allowed in this template context for key '{}'", key),
                ));
            }

            // Check key length
            if key.len() > this.kv_constraints.max_key_length {
                return Err(LuaError::external("Key length too long"));
            }

            let rec = sqlx::query!(
                "SELECT value FROM guild_templates_kv WHERE guild_id = $1 AND key = $2",
                this.guild_id.to_string(),
                key
            )
            .fetch_optional(&this.pool)
            .await;

            match rec {
                // Return None and true if record was found but value is null
                Ok(Some(rec)) => match rec.value {
                    Some(value) => {
                        let value: LuaValue = lua.to_value(&value)?;
                        Ok((Some(value), true))
                    }
                    None => Ok((None, false)),
                },
                // Return None and 0 if record was not found
                Ok(None) => Ok((None, false)),
                // Return error if query failed
                Err(e) => Err(LuaError::external(e)),
            }
        });

        methods.add_async_method("getrecord", |lua, this, key: String| async move {
            this.base_check("get".to_string())
                .map_err(LuaError::external)?;

            if !this.template_data.pragma.kv_ops.contains(&"*".to_string())
                && this
                    .template_data
                    .pragma
                    .kv_ops
                    .contains(&format!("get:{}", key))
                && this.template_data.pragma.kv_ops.contains(&"get:*".to_string())
                && this.template_data.pragma.kv_ops.contains(&key)
            {
                return Err(LuaError::external(
                    format!("Operation `getrecord` [`get` variant] not allowed in this template context for key '{}'", key),
                ));
            }

            // Check key length
            if key.len() > this.kv_constraints.max_key_length {
                return Err(LuaError::external("Key length too long"));
            }

            let rec = sqlx::query!(
                "SELECT value, created_at, last_updated_at FROM guild_templates_kv WHERE guild_id = $1 AND key = $2",
                this.guild_id.to_string(),
                key
            )
            .fetch_optional(&this.pool)
            .await;

            let record = match rec {
                Ok(Some(rec)) => KvRecord {
                    key,
                    value: rec.value.unwrap_or(serde_json::Value::Null),
                    exists: true,
                    created_at: Some(rec.created_at),
                    last_updated_at: Some(rec.last_updated_at),
                },
                Ok(None) => KvRecord {
                    key,
                    value: serde_json::Value::Null,
                    exists: false,
                    created_at: None,
                    last_updated_at: None,
                },
                Err(e) => return Err(LuaError::external(e)),
            };

            let record: LuaValue = lua.to_value(&record)?;
            Ok(record)
        });

        methods.add_async_method("set", |lua, this, (key, value): (String, LuaValue)| async move {
            let data = lua.from_value::<serde_json::Value>(value)?;
            
            this.base_check("set".to_string())
                .map_err(LuaError::external)?;

            if !this.template_data.pragma.kv_ops.contains(&"*".to_string())
                && this
                    .template_data
                    .pragma
                    .kv_ops
                    .contains(&format!("set:{}", key))
                && this.template_data.pragma.kv_ops.contains(&"set:*".to_string())
                && this.template_data.pragma.kv_ops.contains(&key)
            {
                return Err(LuaError::external(
                    format!("Operation `set` not allowed in this template context for key '{}'", key),
                ));
            }

            // Check key length
            if key.len() > this.kv_constraints.max_key_length {
                return Err(LuaError::external("Key length too long"));
            }

            // Check bytes length
            let data_str = serde_json::to_string(&data)
                .map_err(LuaError::external)?;

            if data_str.as_bytes().len() > this.kv_constraints.max_value_bytes {
                return Err(LuaError::external("Value length too long"));
            }

            let mut tx = this.pool.begin().await
                .map_err(LuaError::external)?;

            let rec = sqlx::query!(
                "SELECT COUNT(*) FROM guild_templates_kv WHERE guild_id = $1",
                this.guild_id.to_string(),
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(LuaError::external)?;

            if rec.count.unwrap_or(0) >= this.kv_constraints.max_keys.try_into().map_err(LuaError::external)? {
                return Err(LuaError::external("Max keys limit reached"));
            }

            sqlx::query!(
                "INSERT INTO guild_templates_kv (guild_id, key, value) VALUES ($1, $2, $3) ON CONFLICT (guild_id, key) DO UPDATE SET value = $3, last_updated_at = NOW()",
                this.guild_id.to_string(),
                key,
                data,
            )
            .execute(&mut *tx)
            .await
            .map_err(LuaError::external)?;

            tx.commit().await
            .map_err(LuaError::external)?;

            Ok(())
        });

        methods.add_async_method("delete", |_lua, this, key: String| async move {            
            this.base_check("delete".to_string())
                .map_err(LuaError::external)?;

            if !this.template_data.pragma.kv_ops.contains(&"*".to_string())
                && this
                    .template_data
                    .pragma
                    .kv_ops
                    .contains(&format!("delete:{}", key))
                && this.template_data.pragma.kv_ops.contains(&"delete:*".to_string())
                && this.template_data.pragma.kv_ops.contains(&key)
            {
                return Err(LuaError::external(
                    format!("Operation `delete` not allowed in this template context for key '{}'", key),
                ));
            }

            // Check key length
            if key.len() > this.kv_constraints.max_key_length {
                return Err(LuaError::external("Key length too long"));
            }

            sqlx::query!(
                "DELETE FROM guild_templates_kv WHERE guild_id = $1 AND key = $2",
                this.guild_id.to_string(),
                key,
            )
            .execute(&this.pool)
            .await
            .map_err(LuaError::external)?;

            Ok(())
        });
    }
}

pub fn init_plugin(lua: &Lua) -> LuaResult<LuaTable> {
    let module = lua.create_table()?;

    module.set(
        "new",
        lua.create_function(|lua, (token,): (String,)| {
            let Some(data) = lua.app_data_ref::<state::LuaUserData>() else {
                return Err(LuaError::external("No app data found"));
            };

            let template_data = data
                .per_template
                .get(&token)
                .ok_or_else(|| LuaError::external("Template not found"))?;

            let executor = KvExecutor {
                template_data: template_data.clone(),
                guild_id: data.guild_id,
                pool: data.pool.clone(),
                ratelimits: data.kv_ratelimits.clone(),
                kv_constraints: data.kv_constraints,
            };

            Ok(executor)
        })?,
    )?;

    module.set_readonly(true); // Block any attempt to modify this table

    Ok(module)
}

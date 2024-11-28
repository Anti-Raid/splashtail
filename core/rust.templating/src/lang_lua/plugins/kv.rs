use crate::lang_lua::state;
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
    ratelimits: Arc<state::LuaRatelimits>,
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

impl KvRecord {
    fn default() -> KvRecord {
        KvRecord {
            key: "".to_string(),
            value: serde_json::Value::Null,
            exists: false,
            created_at: None,
            last_updated_at: None,
        }
    }
}

impl KvExecutor {
    pub fn check(&self, action: String, key: String) -> Result<(), crate::Error> {
        if !self.template_data
        .pragma
        .allowed_caps
        .contains(&"kv:*".to_string()) // KV:* means all KV operations are allowed
        && !self.template_data
        .pragma
        .allowed_caps
        .contains(&format!("kv:{}:*", action)) // kv:{action}:* means that the action can be performed on any key
        && !self.template_data
        .pragma
        .allowed_caps
        .contains(&format!("kv:{}:{}", action, key)) // kv:{action}:{key} means that the action can only be performed on said key
        && !self.template_data
        .pragma
        .allowed_caps
        .contains(&format!("kv:*:{}", key))  // kv:*:{key} means that any action can be performed on said key
        {
            return Err(format!("KV operation `{}` not allowed in this template context for key '{}'", action, key).into());
        }

        self.ratelimits.check(&action)?; // Check rate limits

        Ok(())
    }
}

pub fn plugin_docs() -> templating_docgen::Plugin {
    templating_docgen::Plugin::default()
        .name("@antiraid/kv")
        .description("Utilities for key-value operations.")
        .type_mut(
            "KvRecord",
            "KvRecord represents a key-value record with metadata.",
            |t| {
                t
                .example(std::sync::Arc::new(KvRecord::default()))
                .field("key", |f| f.typ("string").description("The key of the record."))
                .field("value", |f| f.typ("any").description("The value of the record."))
                .field("exists", |f| f.typ("bool").description("Whether the record exists."))
                .field("created_at", |f| f.typ("datetime").description("The time the record was created."))
                .field("last_updated_at", |f| f.typ("datetime").description("The time the record was last updated."))
            },
        )
        .type_mut(
            "KvExecutor",
            "KvExecutor allows templates to get, store and find persistent data within a server.",
            |mut t| {
                t
                .method_mut("find", |mut m| {
                    m.parameter("key", |p| p.typ("string").description("The key to search for. % matches zero or more characters; _ matches a single character. To search anywhere in a string, surround {KEY} with %, e.g. %{KEY}%"))
                })
                .method_mut("get", |mut m| {
                    m.parameter("key", |p| p.typ("string").description("The key to get."))
                    .return_("value", |r| r.typ("any").description("The value of the key."))
                    .return_("exists", |r| r.typ("bool").description("Whether the key exists."))
                })
                .method_mut("getrecord", |mut m| {
                    m.parameter("key", |p| p.typ("string").description("The key to get."))
                    .return_("record", |r| r.typ("KvRecord").description("The record of the key."))
                })
                .method_mut("set", |mut m| {
                    m.parameter("key", |p| p.typ("string").description("The key to set."))
                    .parameter("value", |p| p.typ("any").description("The value to set."))
                })
                .method_mut("delete", |mut m| {
                    m.parameter("key", |p| p.typ("string").description("The key to delete."))
                })
            },
        )
        .method_mut("new", |mut m| {
            m.parameter("token", |p| p.typ("TemplateContext").description("The token of the template to use."))
            .return_("executor", |r| r.typ("KvExecutor").description("A key-value executor."))
        })
}

impl LuaUserData for KvExecutor {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_async_method("find", |lua, this, key: String| async move {
            this.check("find".to_string(), key.clone())
                .map_err(LuaError::external)?;

            // Check key length
            if key.len() > this.kv_constraints.max_key_length {
                return Err(LuaError::external("Key length too long"));
            }

            let rec = sqlx::query!(
                "SELECT key, value, created_at, last_updated_at FROM guild_templates_kv WHERE guild_id = $1 AND key ILIKE $2",
                this.guild_id.to_string(),
                key
            )
            .fetch_all(&this.pool)
            .await
            .map_err(LuaError::external)?;

            let mut records = vec![];

            for rec in rec {
                let record = KvRecord {
                    key: rec.key,
                    value: rec.value.unwrap_or(serde_json::Value::Null),
                    exists: true,
                    created_at: Some(rec.created_at),
                    last_updated_at: Some(rec.last_updated_at),
                };

                records.push(record);
            }

            let records: LuaValue = lua.to_value(&records)?;

            Ok(records)
        });

        methods.add_async_method("get", |lua, this, key: String| async move {
            this.check("get".to_string(), key.clone())
                .map_err(LuaError::external)?;

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
            this.check("get".to_string(), key.clone())
                .map_err(LuaError::external)?;

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
            this.check("set".to_string(), key.clone())
            .map_err(LuaError::external)?;
            
            let data = lua.from_value::<serde_json::Value>(value)?;
            
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
            this.check("delete".to_string(), key.clone())
            .map_err(LuaError::external)?;
            
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
        lua.create_function(|lua, (token,): (crate::TemplateContextRef,)| {
            let Some(data) = lua.app_data_ref::<state::LuaUserData>() else {
                return Err(LuaError::external("No app data found"));
            };

            let executor = KvExecutor {
                template_data: token.template_data.clone(),
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

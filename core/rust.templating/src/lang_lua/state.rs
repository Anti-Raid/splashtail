use std::sync::Arc;

#[allow(dead_code)]
pub struct LuaKVConstraints {
    /// Maximum number of keys allowed in the KV store
    pub max_keys: usize,
    /// Maximum length of a key
    pub max_key_length: usize,
    /// Maximum length of a value (in bytes)
    pub max_value_bytes: usize,
}

impl Default for LuaKVConstraints {
    fn default() -> Self {
        LuaKVConstraints {
            max_keys: 10,
            max_key_length: 64,
            // 50kb max per value
            max_value_bytes: 50 * 1024,
        }
    }
}

#[allow(dead_code)]
pub struct TemplateData {
    pub pragma: crate::TemplatePragma,
}

#[allow(dead_code)]
pub struct LuaUserData {
    pub pool: sqlx::PgPool,
    pub guild_id: serenity::all::GuildId,
    pub kv_constraints: LuaKVConstraints,

    /// Stores a list of tokens to template data
    ///
    /// Used by actions and other things which use pragma
    pub per_template: scc::HashMap<String, Arc<TemplateData>>,
}

pub fn add_template(
    lua: &mlua::Lua,
    pragma: crate::TemplatePragma,
) -> Result<String, crate::Error> {
    let token = botox::crypto::gen_random(32);

    let data = TemplateData { pragma };

    let data = Arc::new(data);

    let app_data = lua
        .app_data_ref::<LuaUserData>()
        .ok_or("Failed to get user data")?;

    app_data
        .per_template
        .insert(token.clone(), data)
        .map_err(|_| "Failed to insert template token")?;

    Ok(token)
}

pub fn remove_template(lua: &mlua::Lua, token: &str) -> Result<(), crate::Error> {
    let app_data = lua
        .app_data_ref::<LuaUserData>()
        .ok_or("Failed to get user data")?;

    app_data.per_template.remove(token);

    Ok(())
}

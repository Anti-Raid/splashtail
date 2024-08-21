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
pub struct LuaUserData {
    pub pool: sqlx::PgPool,
    pub guild_id: serenity::all::GuildId,
    pub kv_constraints: LuaKVConstraints,
}

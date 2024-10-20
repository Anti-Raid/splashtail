use governor::{clock::QuantaClock, DefaultKeyedRateLimiter};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

#[allow(dead_code)]
pub struct LuaActionsRatelimit {
    pub clock: QuantaClock,
    pub global: Vec<DefaultKeyedRateLimiter<()>>,
    pub per_bucket: indexmap::IndexMap<String, Vec<DefaultKeyedRateLimiter<()>>>,
}

impl LuaActionsRatelimit {
    ///
    /// Default global limit: 10 actions per 60 seconds
    /// Bucket limits:
    /// -> ban: [5 per 30 seconds, 10 per 75 seconds]
    pub fn new() -> Result<Self, crate::Error> {
        fn create_quota(
            limit_per: NonZeroU32,
            limit_time: Duration,
        ) -> Result<governor::Quota, crate::Error> {
            let quota = governor::Quota::with_period(limit_time)
                .ok_or("Failed to create quota")?
                .allow_burst(limit_per);

            Ok(quota)
        }

        // Create the global limit
        let global_quota = create_quota(NonZeroU32::new(10).unwrap(), Duration::from_secs(60))?;
        let global1 = DefaultKeyedRateLimiter::keyed(global_quota);
        let global = vec![global1];

        // Create the per-bucket limits
        let ban_quota1 = create_quota(NonZeroU32::new(5).unwrap(), Duration::from_secs(30))?;
        let ban_lim1 = DefaultKeyedRateLimiter::keyed(ban_quota1);
        let ban_quota2 = create_quota(NonZeroU32::new(10).unwrap(), Duration::from_secs(75))?;
        let ban_lim2 = DefaultKeyedRateLimiter::keyed(ban_quota2);

        let kick_quota1 = create_quota(NonZeroU32::new(5).unwrap(), Duration::from_secs(30))?;
        let kick_lim1 = DefaultKeyedRateLimiter::keyed(kick_quota1);
        let kick_quota2 = create_quota(NonZeroU32::new(10).unwrap(), Duration::from_secs(75))?;
        let kick_lim2 = DefaultKeyedRateLimiter::keyed(kick_quota2);

        // Send message channel limits (are smaller to allow for more actions)
        let sendmessage_channel_quota1 =
            create_quota(NonZeroU32::new(15).unwrap(), Duration::from_secs(20))?;
        let sendmessage_channel_lim1 = DefaultKeyedRateLimiter::keyed(sendmessage_channel_quota1);

        // Create the clock
        let clock = QuantaClock::default();

        Ok(LuaActionsRatelimit {
            global,
            per_bucket: indexmap::indexmap!(
                "ban".to_string() => vec![ban_lim1, ban_lim2] as Vec<DefaultKeyedRateLimiter<()>>,
                "kick".to_string() => vec![kick_lim1, kick_lim2] as Vec<DefaultKeyedRateLimiter<()>>,
                "sendmessage_channel".to_string() => vec![sendmessage_channel_lim1] as Vec<DefaultKeyedRateLimiter<()>>,
            ),
            clock,
        })
    }
}

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
    pub cache_http: botox::cache::CacheHttpImpl,
    pub reqwest_client: reqwest::Client,
    pub kv_constraints: LuaKVConstraints,

    /// Stores a list of tokens to template data
    ///
    /// Used by actions and other things which use pragma
    pub per_template: scc::HashMap<String, Arc<TemplateData>>,

    /// Stores the lua actions ratelimiters
    pub ratelimits: Arc<LuaActionsRatelimit>,
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

use governor::clock::Clock;
use governor::{clock::QuantaClock, DefaultKeyedRateLimiter};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

#[allow(dead_code)]
pub struct LuaRatelimits {
    pub clock: QuantaClock,
    pub global: Vec<DefaultKeyedRateLimiter<()>>,
    pub per_bucket: indexmap::IndexMap<String, Vec<DefaultKeyedRateLimiter<()>>>,
}

impl LuaRatelimits {
    fn create_quota(
        limit_per: NonZeroU32,
        limit_time: Duration,
    ) -> Result<governor::Quota, crate::Error> {
        let quota = governor::Quota::with_period(limit_time)
            .ok_or("Failed to create quota")?
            .allow_burst(limit_per);

        Ok(quota)
    }

    pub fn new_action_rl() -> Result<Self, crate::Error> {
        // Create the global limit
        let global_quota =
            Self::create_quota(NonZeroU32::new(10).unwrap(), Duration::from_secs(10))?;
        let global1 = DefaultKeyedRateLimiter::keyed(global_quota);
        let global = vec![global1];

        // Create the per-bucket limits
        let ban_quota1 = Self::create_quota(NonZeroU32::new(5).unwrap(), Duration::from_secs(30))?;
        let ban_lim1 = DefaultKeyedRateLimiter::keyed(ban_quota1);
        let ban_quota2 = Self::create_quota(NonZeroU32::new(10).unwrap(), Duration::from_secs(75))?;
        let ban_lim2 = DefaultKeyedRateLimiter::keyed(ban_quota2);

        let kick_quota1 = Self::create_quota(NonZeroU32::new(5).unwrap(), Duration::from_secs(30))?;
        let kick_lim1 = DefaultKeyedRateLimiter::keyed(kick_quota1);
        let kick_quota2 =
            Self::create_quota(NonZeroU32::new(10).unwrap(), Duration::from_secs(75))?;
        let kick_lim2 = DefaultKeyedRateLimiter::keyed(kick_quota2);

        // Send message channel limits (are smaller to allow for more actions)
        let create_message_quota1 =
            Self::create_quota(NonZeroU32::new(15).unwrap(), Duration::from_secs(20))?;
        let create_message_lim1 = DefaultKeyedRateLimiter::keyed(create_message_quota1);

        // Create the clock
        let clock = QuantaClock::default();

        Ok(Self {
            global,
            per_bucket: indexmap::indexmap!(
                "ban".to_string() => vec![ban_lim1, ban_lim2] as Vec<DefaultKeyedRateLimiter<()>>,
                "kick".to_string() => vec![kick_lim1, kick_lim2] as Vec<DefaultKeyedRateLimiter<()>>,
                "create_message".to_string() => vec![create_message_lim1] as Vec<DefaultKeyedRateLimiter<()>>,
            ),
            clock,
        })
    }

    pub fn new_kv_rl() -> Result<Self, crate::Error> {
        // Create the global limit
        let global_quota =
            Self::create_quota(NonZeroU32::new(10).unwrap(), Duration::from_secs(60))?;
        let global1 = DefaultKeyedRateLimiter::keyed(global_quota);
        let global = vec![global1];

        // Create the clock
        let clock = QuantaClock::default();

        Ok(Self {
            global,
            per_bucket: indexmap::indexmap!(),
            clock,
        })
    }

    pub fn new_stings_rl() -> Result<Self, crate::Error> {
        // Create the global limit
        let global_quota =
            Self::create_quota(NonZeroU32::new(10).unwrap(), Duration::from_secs(60))?;
        let global1 = DefaultKeyedRateLimiter::keyed(global_quota);
        let global = vec![global1];

        // Create the clock
        let clock = QuantaClock::default();

        Ok(Self {
            global,
            per_bucket: indexmap::indexmap!(),
            clock,
        })
    }

    pub fn check(&self, bucket: &str) -> Result<(), crate::Error> {
        // Check global ratelimits
        for global_lim in self.global.iter() {
            match global_lim.check_key(&()) {
                Ok(()) => continue,
                Err(wait) => {
                    return Err(format!(
                        "Global ratelimit hit for bucket '{}', wait time: {:?}",
                        bucket,
                        wait.wait_time_from(self.clock.now())
                    )
                    .into());
                }
            };
        }

        // Check per bucket ratelimits
        if let Some(per_bucket) = self.per_bucket.get(bucket) {
            for lim in per_bucket.iter() {
                match lim.check_key(&()) {
                    Ok(()) => continue,
                    Err(wait) => {
                        return Err(format!(
                            "Per bucket ratelimit hit for '{}', wait time: {:?}",
                            bucket,
                            wait.wait_time_from(self.clock.now())
                        )
                        .into());
                    }
                };
            }
        }

        Ok(())
    }
}

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
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
            max_keys: 2048,
            max_key_length: 128,
            // 50kb max per value
            max_value_bytes: 50 * 1024,
        }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct TemplateData {
    pub path: String,
    pub template: crate::Template,
    pub pragma: crate::TemplatePragma,
}

#[allow(dead_code)]
pub struct LuaUserData {
    pub last_execution_time: Arc<crate::atomicinstant::AtomicInstant>,
    pub pool: sqlx::PgPool,
    pub guild_id: serenity::all::GuildId,
    pub serenity_context: serenity::all::Context,
    pub shard_messenger: serenity::all::ShardMessenger,
    pub reqwest_client: reqwest::Client,
    pub kv_constraints: LuaKVConstraints,

    /// Stores the lua actions ratelimiters
    pub actions_ratelimits: Arc<LuaRatelimits>,

    /// Stores the lua kv ratelimiters
    pub kv_ratelimits: Arc<LuaRatelimits>,

    /// Stores the lua sting ratelimiters
    pub sting_ratelimits: Arc<LuaRatelimits>,

    /// The bytecode cache
    pub vm_bytecode_cache: Arc<super::BytecodeCache>,

    /// Stores the luau compiler
    pub compiler: Arc<mlua::Compiler>,
}

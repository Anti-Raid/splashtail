use super::core::{Limit, LimitGuild, LimitTypes};
use governor::{
    clock::{Clock, QuantaClock},
    DefaultKeyedRateLimiter, Quota,
};
use moka::future::Cache;
use serenity::all::{GuildId, UserId};
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::{Arc, LazyLock};

// Hashmap of limit types to a hashmap of limit ids to its ratelimiter
pub type RatelimiterMap<RlKey> =
    HashMap<LimitTypes, HashMap<String, DefaultKeyedRateLimiter<RlKey>>>;

pub struct GuildLimitsCache {
    pub global: RatelimiterMap<()>,
    pub per_user: RatelimiterMap<UserId>,
    pub clock: QuantaClock,
}

pub struct IHitLimit {
    /// The time at which the bucket will be replenished
    pub time: std::time::Duration,
}

impl GuildLimitsCache {
    /// Attempts to limit a user, returning a tuple of whether the user is allowed to continue, the time at which the bucket will be replenished, limit id that was hit
    pub async fn limit(
        &self,
        user_id: UserId,
        limit_type: LimitTypes,
    ) -> (bool, HashMap<String, IHitLimit>) {
        let mut hit_limits = HashMap::new();

        if let Some(limits) = self.per_user.get(&limit_type) {
            for (limit_id, lim) in limits.iter() {
                match lim.check_key(&user_id) {
                    Ok(()) => continue, // TODO: Return the time at which the bucket will be replenished
                    Err(wait) => {
                        hit_limits.insert(limit_id.clone(), {
                            IHitLimit {
                                time: wait.wait_time_from(self.clock.now()),
                            }
                        });
                    }
                }
            }
        }

        if let Some(global_limits) = self.global.get(&limit_type) {
            for (limit_id, lim) in global_limits.iter() {
                match lim.check_key(&()) {
                    Ok(()) => continue, // TODO: Return the time at which the bucket will be replenished
                    Err(wait) => {
                        hit_limits.insert(limit_id.clone(), {
                            IHitLimit {
                                time: wait.wait_time_from(self.clock.now()),
                            }
                        });
                    }
                }
            }
        }

        (hit_limits.is_empty(), hit_limits)
    }
}

pub type CachedGuildLimit = (
    super::strategy::LimitStrategy,
    LimitGuild,
    GuildLimitsCache,
    HashMap<String, Limit>,
);

pub static GUILD_LIMITS: LazyLock<Cache<GuildId, Arc<CachedGuildLimit>>> =
    LazyLock::new(|| Cache::builder().support_invalidation_closures().build());

pub async fn get_limits(
    data: &silverpelt::data::Data,
    guild_id: GuildId,
) -> Result<Arc<CachedGuildLimit>, silverpelt::Error> {
    if let Some(limits) = GUILD_LIMITS.get(&guild_id).await {
        Ok(limits.clone())
    } else {
        let mut limits = GuildLimitsCache {
            global: HashMap::new(),
            per_user: HashMap::new(),
            clock: QuantaClock::default(),
        };

        let limit_guild = LimitGuild::get(&data.pool, guild_id).await?;

        // Init limits db here
        let limits_db: HashMap<String, Limit> = Limit::guild(&data.pool, guild_id)
            .await?
            .into_iter()
            .map(|a| (a.limit_id.clone(), a))
            .collect();

        for limit in limits_db.values() {
            let quota = create_quota(limit)?;

            let lim = DefaultKeyedRateLimiter::keyed(quota);

            // TODO: Support global limits
            limits
                .per_user
                .entry(limit.limit_type)
                .or_default()
                .insert(limit.limit_id.clone(), lim);
        }

        let limits = Arc::new((
            super::strategy::get_strategy(limit_guild.strategy),
            limit_guild,
            limits,
            limits_db,
        ));

        GUILD_LIMITS.insert(guild_id, limits.clone()).await;

        Ok(limits)
    }
}

pub fn create_quota(limit: &Limit) -> Result<Quota, silverpelt::Error> {
    let limit_per = NonZeroU32::new(limit.limit_per as u32).ok_or("Invalid limit_per")?;
    let quota = Quota::with_period(std::time::Duration::from_secs(limit.limit_time as u64))
        .ok_or("Failed to create quota")?
        .allow_burst(limit_per);

    Ok(quota)
}

#[cfg(test)]
mod test {
    use serenity::all::GuildId;

    #[test]
    pub fn test_create_quota() {
        // Limit of 10 reqs per 5 seconds
        let limit = super::Limit {
            limit_id: "test".to_string(),
            limit_name: "test".to_string(),
            limit_per: 10,
            limit_time: 5,
            limit_type: super::LimitTypes::Ban,
            stings: 1,
            guild_id: GuildId::new(0),
        };

        let quota = super::create_quota(&limit).unwrap();
        let lim = governor::RateLimiter::keyed(quota);

        for _ in 0..10 {
            assert!(lim.check_key(&()).is_ok());
        }

        assert!(lim.check_key(&()).is_err());

        // Make a new bucket
        let lim = governor::RateLimiter::keyed(quota);

        // Make 10 requests
        for _ in 0..10 {
            assert!(lim.check_key(&()).is_ok());
        }

        // Wait 8 seconds
        std::thread::sleep(std::time::Duration::from_secs(8));

        assert!(lim.check_key(&()).is_ok());
        // Next one should fail
        assert!(lim.check_key(&()).is_err());
    }
}

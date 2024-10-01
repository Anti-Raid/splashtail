use super::core::HandleModAction;
use botox::crypto::gen_random;
use std::collections::HashMap;
use dashmap::DashMap;
use std::sync::LazyLock;

pub static STRATEGY: LazyLock<DashMap<String, Box<dyn CreateStrategy>>> =
    LazyLock::new(|| {
        let map: DashMap<String, Box<dyn CreateStrategy>> = DashMap::new();

        map.insert("in-memory".to_string(), Box::new(CreateInMemoryStrategy));
        map.insert("persist".to_string(), Box::new(CreatePersistStrategy));

        map
    });

/// Given a string, returns the limit strategy
pub fn from_limit_strategy_string(s: &str) -> Result<Box<dyn Strategy>, silverpelt::Error> {
    for pair in STRATEGY.iter() {
        let creator = pair.value();
        if let Some(m) = creator.to_strategy(s)? {
            return Ok(m);
        }
    }

    Err("Unknown lockdown mode".into())
}

pub struct StrategyResult {
    pub stings: i32,                                    // How many stings to give
    pub hit_limits: Vec<String>,                        // Which limit IDs were hit
    pub expiries: HashMap<String, std::time::Duration>, // If possible, the time at which the limit will be lifted
    pub expiry: Option<std::time::Duration>, // If possible/needed, the time at which the stings should expire
    pub causes: Option<HashMap<String, Vec<String>>>, // If possible, the actions which caused the limit to be hit
}

pub trait CreateStrategy
where
    Self: Send + Sync, 
{
    /// Returns the syntax for the limit
    ///
    /// E.g. `in-memory` for In Memory Limit
    fn syntax(&self) -> &'static str;

    fn to_strategy(&self, s: &str) -> Result<Option<Box<dyn Strategy>>, silverpelt::Error>;
}

#[async_trait::async_trait]
pub trait Strategy
where
    Self: Send + Sync,
{
    /// Returns the creator for the limit
    #[allow(dead_code)]
    fn creator(&self) -> Box<dyn CreateStrategy>;

    /// Returns the string form of the strategy
    #[allow(dead_code)]
    fn string_form(&self) -> String;

    /// Adds a mod action to the strategy
    async fn add_mod_action(
        &self,
        data: &silverpelt::data::Data,
        ha: &HandleModAction,
        cgl: &super::cache::CachedGuildLimit,
    ) -> Result<StrategyResult, silverpelt::Error>;
}

pub struct CreateInMemoryStrategy;

impl CreateStrategy for CreateInMemoryStrategy {
    fn to_strategy(&self, s: &str) -> Result<Option<Box<dyn Strategy>>, silverpelt::Error> {
        if s == "in-memory" {
            Ok(Some(Box::new(InMemoryStrategy)))
        } else {
            Ok(None)
        }
    }

    fn syntax(&self) -> &'static str {
        "in-memory"
    }
}

pub struct InMemoryStrategy;

#[async_trait::async_trait]
impl Strategy for InMemoryStrategy {
    fn string_form(&self) -> String {
        "in-memory".to_string()
    }

    fn creator(&self) -> Box<dyn CreateStrategy> {
        Box::new(CreateInMemoryStrategy)
    }

    // NOTE: In memory aggregates the causes so there is nothing to return for that
    async fn add_mod_action(
        &self,
        _data: &silverpelt::data::Data,
        ha: &HandleModAction,
        cgl: &super::cache::CachedGuildLimit,
    ) -> Result<StrategyResult, silverpelt::Error> {
        let (ok, result) = cgl.1.limit(ha.user_id, ha.limit).await;

        if ok {
            return Ok(StrategyResult {
                stings: 0,
                hit_limits: Vec::new(),
                expiries: HashMap::new(),
                expiry: None,
                causes: None,
            });
        }

        let mut hit_limits = Vec::new();
        let mut stings = 0;
        let mut expiries = HashMap::new();

        // Count the stings and expiries
        for (limit_id, hit_limit) in result {
            if let Some(limit) = cgl.2.get(&limit_id) {
                stings += limit.stings;
                expiries.insert(limit_id.clone(), hit_limit.time);
                hit_limits.push(limit_id);
            }
        }

        Ok(StrategyResult {
            stings,
            hit_limits,
            expiries,
            expiry: None,
            causes: None,
        })
    }
}

pub struct CreatePersistStrategy;

impl CreateStrategy for CreatePersistStrategy {
    fn to_strategy(&self, s: &str) -> Result<Option<Box<dyn Strategy>>, silverpelt::Error> {
        if s == "persist" {
            Ok(Some(Box::new(PersistStrategy)))
        } else {
            Ok(None)
        }
    }

    fn syntax(&self) -> &'static str {
        "persist"
    }
}

pub struct PersistStrategy;

#[async_trait::async_trait]
impl Strategy for PersistStrategy {
    fn string_form(&self) -> String {
        "persist".to_string()
    }

    fn creator(&self) -> Box<dyn CreateStrategy> {
        Box::new(CreatePersistStrategy)
    }

    async fn add_mod_action(
        &self,
        data: &silverpelt::data::Data,
        ha: &HandleModAction,
        cgl: &super::cache::CachedGuildLimit,
    ) -> Result<StrategyResult, silverpelt::Error> {
        let mut tx = data.pool.begin().await?;

        let action_id = gen_random(48);

        sqlx::query!(
            "INSERT INTO limits__user_actions (action_id, guild_id, user_id, target, limit_type, action_data, created_at, stings) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            action_id,
            ha.guild_id.to_string(),
            ha.user_id.to_string(),
            ha.target,
            ha.limit.to_string(),
            ha.action_data,
            sqlx::types::chrono::Utc::now(),
            0
        )
        .execute(&mut *tx)
        .await?;

        let mut hit_limits = Vec::new();

        let mut stings = 0;
        let mut largest_expiry = 0;
        let mut expiries = HashMap::new();
        for (_limit_id, guild_limit) in cgl.2.iter() {
            let stings_from_limit = guild_limit.stings;
            let limit_time_from_limit = guild_limit.limit_time;

            expiries.insert(
                guild_limit.limit_id.clone(),
                std::time::Duration::from_secs(limit_time_from_limit as u64),
            );

            // Ensure the expiry is based on all limits, not just infringing
            if limit_time_from_limit > largest_expiry {
                largest_expiry = limit_time_from_limit;
            }

            // Check the limit type and user_id and guild to see if it is in the cache
            let infringing_actions = sqlx::query!(
                "select action_id from limits__user_actions where guild_id = $1 and user_id = $2 and limit_type = $3 and created_at + make_interval(secs => $4) > now()",
                ha.guild_id.to_string(),
                ha.user_id.to_string(),
                ha.limit.to_string(),
                guild_limit.limit_time as f64,
            )
            .fetch_all(&mut *tx)
            .await?;

            if infringing_actions.len() >= guild_limit.limit_per as usize {
                hit_limits.push((
                    infringing_actions
                        .into_iter()
                        .map(|v| v.action_id)
                        .collect::<Vec<String>>(),
                    guild_limit,
                ));

                stings += stings_from_limit;
            }
        }

        if stings > 0 || largest_expiry > 0 {
            sqlx::query!(
                "UPDATE limits__user_actions SET stings = $1, stings_expiry = $2 WHERE action_id = $3",
                stings,
                sqlx::types::chrono::Utc::now() + chrono::Duration::seconds(largest_expiry),
                action_id
            )
            .execute(&mut *tx)
            .await?;

            // Delete older user actions
            sqlx::query!(
            "DELETE FROM limits__user_actions WHERE user_id = $1 AND guild_id = $2 AND created_at < now() - make_interval(secs => $3)",
            ha.user_id.to_string(),
            ha.guild_id.to_string(),
            largest_expiry as f64,
        )
        .execute(&mut *tx)
        .await?;
        }

        tx.commit().await?;

        let hit_limits_result = hit_limits.iter().flat_map(|(ids, _)| ids.clone()).collect();

        let mut causes = HashMap::new();

        for (ids, limit) in hit_limits {
            causes.insert(limit.limit_id.clone(), ids);
        }

        Ok(StrategyResult {
            stings,
            hit_limits: hit_limits_result,
            expiries,
            expiry: None,
            causes: Some(causes),
        })
    }
}

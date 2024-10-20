use super::core::HandleModAction;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::LazyLock;

pub static STRATEGY: LazyLock<DashMap<String, Box<dyn CreateStrategy>>> = LazyLock::new(|| {
    let map: DashMap<String, Box<dyn CreateStrategy>> = DashMap::new();

    map.insert("in-memory".to_string(), Box::new(CreateInMemoryStrategy));
    map.insert("template".to_string(), Box::new(CreateTemplateStrategy));

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

#[derive(serde::Serialize, serde::Deserialize)]
pub struct StrategyResult {
    pub stings: i32,                               // How many stings to give
    pub sting_expiry: Option<std::time::Duration>, // The expiry of the stings, in seconds
    pub reason: Option<String>,                    // The reason for the stings
    pub data: Option<serde_json::Value>,           // The strategy data
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
        ctx: &serenity::all::Context,
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

    // NOTE: In memory aggregates the causes
    async fn add_mod_action(
        &self,
        _ctx: &serenity::all::Context,
        ha: &HandleModAction,
        cgl: &super::cache::CachedGuildLimit,
    ) -> Result<StrategyResult, silverpelt::Error> {
        const DEFAULT_EXPIRY: std::time::Duration = std::time::Duration::from_secs(60 * 5);

        let (ok, result) = cgl.1.limit(ha.user_id, ha.limit).await;

        if ok {
            return Ok(StrategyResult {
                stings: 0,
                reason: None,
                sting_expiry: None,
                data: None,
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

        let sting_expiry = {
            // Get the longest duration from expiries
            let max = expiries.iter().max();

            match max {
                Some((_, expiry)) => *expiry,
                None => DEFAULT_EXPIRY,
            }
        };

        Ok(StrategyResult {
            stings,
            sting_expiry: Some(sting_expiry),
            reason: Some(format!("Hit limits: {:?}", hit_limits)),
            data: Some(serde_json::json!({
                "hit_limits": hit_limits,
                "expiries": expiries,
            })),
        })
    }
}

pub struct CreateTemplateStrategy;

impl CreateStrategy for CreateTemplateStrategy {
    fn to_strategy(&self, s: &str) -> Result<Option<Box<dyn Strategy>>, silverpelt::Error> {
        if s.starts_with("template:") && s.len() > 9 {
            Ok(Some(Box::new(TemplateStrategy(s[9..].to_string()))))
        } else {
            Ok(None)
        }
    }

    fn syntax(&self) -> &'static str {
        "template:<template name>"
    }
}

pub struct TemplateStrategy(String);

#[async_trait::async_trait]
impl Strategy for TemplateStrategy {
    fn string_form(&self) -> String {
        format!("template:{}", self.0)
    }

    fn creator(&self) -> Box<dyn CreateStrategy> {
        Box::new(CreateTemplateStrategy)
    }

    async fn add_mod_action(
        &self,
        ctx: &serenity::all::Context,
        ha: &HandleModAction,
        cgl: &super::cache::CachedGuildLimit,
    ) -> Result<StrategyResult, silverpelt::Error> {
        let data = ctx.data::<silverpelt::data::Data>();
        templating::execute::<_, StrategyResult>(
            ha.guild_id,
            templating::Template::Named(self.0.clone()),
            data.pool.clone(),
            botox::cache::CacheHttpImpl::from_ctx(ctx),
            data.reqwest.clone(),
            TemplateStrategyContext {
                handle_mod_action: ha.clone(),
                limits: cgl.2.clone(),
                limit_guild: cgl.0.clone(),
            },
        )
        .await
    }
}

/// A TemplateStrategyContext is a context for template strategy templates
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct TemplateStrategyContext {
    pub handle_mod_action: HandleModAction,
    pub limits: std::collections::HashMap<String, super::core::Limit>,
    pub limit_guild: super::core::LimitGuild,
}

#[typetag::serde]
impl templating::Context for TemplateStrategyContext {}

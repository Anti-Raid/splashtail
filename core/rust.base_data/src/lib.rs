pub mod limits;
pub mod permissions;
pub mod permodule;

use splashcore_rs::{animusmagic::client::AnimusMagicRequestClient, objectstore::ObjectStore};
use std::sync::Arc;
use tokio::sync::RwLock;

pub type Error = Box<dyn std::error::Error + Send + Sync>; // This is constant and should be copy pasted
pub type Context<'a> = poise::Context<'a, Data, Error>;

/*#[derive(Clone)]
pub struct AnimusMagicBaseData {
    pub pool: sqlx::PgPool,
    pub redis_pool: fred::prelude::RedisPool,
    pub reqwest: reqwest::Client,
    pub cache_http: botox::cache::CacheHttpImpl,
}*/

/// This struct stores base/standard command data, which is stored and accessible in all command invocations
pub struct Data {
    pub pool: sqlx::PgPool,
    pub redis_pool: fred::prelude::RedisPool,
    pub reqwest: reqwest::Client,
    pub object_store: Arc<ObjectStore>,
    pub shards_ready: Arc<dashmap::DashMap<u16, bool>>,
    pub proxy_support_data: RwLock<Option<proxy_support::ProxySupportData>>, // Shard ID, WebsocketConfiguration
    pub props: Arc<dyn Props>,

    /// Any extra data
    pub extra_data: Arc<dyn std::any::Any + Send + Sync>,
}

impl Data {
    /// A container for a extra_data type that can be used in command execution.
    ///
    /// The purpose of the extra_data field is to be accessible and persistent across contexts; that is,
    /// data can be modified by one context, and will persist through the future and be accessible
    /// through other contexts. This is useful for anything that should "live" through the program:
    /// counters, database connections, custom user caches, etc.
    ///
    /// # Panics
    /// Panics if the generic provided is not equal to the type provided in its creation
    #[must_use]
    pub fn extra_data<Data: Send + Sync + 'static>(&self) -> Arc<Data> {
        Arc::clone(&self.extra_data)
            .downcast()
            .expect("Type provided to extra_data should be the same as data.")
    }
}

pub trait Props
where
    Self: Send + Sync,
{
    /// Returns the underlying client for animus magic
    fn underlying_am_client(&self) -> Result<Box<dyn AnimusMagicRequestClient>, Error>;

    /// Returns the per module executor of the context
    fn permodule_executor(&self) -> Box<dyn permodule::PermoduleFunctionExecutor>;

    /// Adds a permodule function to the executor
    fn add_permodule_function(&self, module: &str, function: &str, func: permodule::ToggleFunc);

    /// The name of the service
    fn name(&self) -> String;

    /// If applicable, the shards associated with the service
    fn shards(&self) -> Vec<u16>;

    /// If applicable, the shard count
    fn shard_count(&self) -> u16;

    /// The cluster ID
    fn cluster_id(&self) -> u16;

    /// The cluster name
    fn cluster_name(&self) -> String;

    /// The total number of clusters
    fn cluster_count(&self) -> u16;

    /// The number of available clusters
    fn available_clusters(&self) -> usize;

    /// Total number of guilds
    ///
    /// Note that this statistic may not always be available, in such cases, 0 will be returned
    fn total_guilds(&self) -> u64;

    /// Total number of users
    ///
    /// Note that this statistic may not always be available, in such cases, 0 will be returned
    fn total_users(&self) -> u64;
}

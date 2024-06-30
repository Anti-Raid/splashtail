pub mod config;
pub mod permissions;
pub mod permodule;

use splashcore_rs::objectstore::ObjectStore;
use std::sync::Arc;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Clone)]
pub struct AnimusMagicBaseData {
    pub pool: sqlx::PgPool,
    pub redis_pool: fred::prelude::RedisPool,
    pub reqwest: reqwest::Client,
    pub cache_http: botox::cache::CacheHttpImpl,
}

/// This struct stores base/standard command data, which is stored and accessible in all command invocations
pub struct Data {
    pub pool: sqlx::PgPool,
    pub redis_pool: fred::prelude::RedisPool,
    pub reqwest: reqwest::Client,
    //pub mewld_ipc: Arc<MewldIpcClient>,
    //pub animus_magic_ipc: OnceLock<Arc<AnimusMagicClient>>, // a rwlock is needed as the cachehttp is only available after the client is started
    pub object_store: Arc<ObjectStore>,
    pub shards_ready: Arc<dashmap::DashMap<u16, bool>>,
    //pub proxy_support_data: RwLock<Option<ProxySupportData>>, // Shard ID, WebsocketConfiguration
    pub props: Box<dyn Props>,

    /// Any extra data
    extra_data: Arc<dyn std::any::Any + Send + Sync>,
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

/// Core statistics about the service
pub struct Statistics {
    /// The name of the service
    pub name: String,
    /// If applicable, the shards associated with the service
    pub shards: Vec<u16>,
    /// If applicable, the shard count
    pub shard_count: u16,
    /// If applicable, the shard count as a NonZeroU16
    pub shard_count_nonzero: std::num::NonZeroU16,
    /// The cluster ID
    pub cluster_id: u16,
    /// The cluster name
    pub cluster_name: String,
    /// The number of clusters
    pub cluster_count: u16,
}

pub trait Props
where
    Self: Send + Sync,
{
    /// Returns the underlying client for animus magic
    fn underlying_am_client(
        &self,
    ) -> Arc<splashcore_rs::animusmagic::client::UnderlyingClient<AnimusMagicBaseData>>;

    /// Returns the per module executor of the context
    fn permodule_executor(&self) -> Box<dyn permodule::PermoduleFunctionExecutor>;

    /// Adds a permodule function to the executor
    fn add_permodule_function(&self, module: &str, function: &str, func: permodule::ToggleFunc);

    /// Returns the statistics of the service
    fn statistics(&self) -> Statistics;
}

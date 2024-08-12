pub mod limits;
pub mod permissions;
pub mod permodule;

use splashcore_rs::{animusmagic::client::AnimusMagicRequestClient, objectstore::ObjectStore};
use std::sync::Arc;

pub type Error = Box<dyn std::error::Error + Send + Sync>; // This is constant and should be copy pasted

/// This struct stores base/standard command data, which is stored and accessible in all command invocations
#[derive(Clone)]
pub struct Data {
    pub pool: sqlx::PgPool,
    pub redis_pool: fred::prelude::RedisPool,
    pub reqwest: reqwest::Client,
    pub object_store: Arc<ObjectStore>,
    pub props: Arc<dyn Props + Send + Sync>,

    /// Any extra data
    pub extra_data: Arc<dyn std::any::Any + Send + Sync>,
}

#[async_trait::async_trait]
pub trait Props
where
    Self: Send + Sync,
{
    /// Converts the props to std::any::Any
    fn as_any(&self) -> &(dyn std::any::Any + Send + Sync);

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

    /// Proxy support data
    async fn get_proxysupport_data(&self) -> Option<Arc<proxy_support::ProxySupportData>>;

    /// Set the proxy support data
    async fn set_proxysupport_data(
        &self,
        data: proxy_support::ProxySupportData,
    ) -> Result<(), Error>;

    /// Total number of guilds
    ///
    /// Note that this statistic may not always be available, in such cases, 0 will be returned
    fn total_guilds(&self) -> u64;

    /// Total number of users
    ///
    /// Note that this statistic may not always be available, in such cases, 0 will be returned
    fn total_users(&self) -> u64;

    /// Reset the can_use_bot whitelist
    async fn reset_can_use_bot(&self) -> Result<(), Error>;
}

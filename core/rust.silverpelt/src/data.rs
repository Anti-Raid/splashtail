use splashcore_rs::objectstore::ObjectStore;
use std::fmt::Debug;
use std::sync::Arc;

/// This struct stores base/standard command data, which is stored and accessible in all command invocations
#[derive(Clone)]
pub struct Data {
    pub pool: sqlx::PgPool,
    pub reqwest: reqwest::Client,
    pub object_store: Arc<ObjectStore>,
    pub props: Arc<dyn Props + Send + Sync>,

    /// Any extra data represented as a key-value map
    pub extra_data: dashmap::DashMap<i32, Arc<dyn std::any::Any + Send + Sync>>,

    /// The silverpelt cache to use for this module
    pub silverpelt_cache: Arc<crate::cache::SilverpeltCache>,
}

impl Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Data")
            .field("pool", &"sqlx::PgPool")
            .field("reqwest", &"reqwest::Client")
            .field("object_store", &"Arc<ObjectStore>")
            .field("props", &"Arc<dyn Props + Send + Sync>")
            .field("extra_data", &self.extra_data.len())
            .field("silverpelt_cache", &"Arc<crate::cache::SilverpeltCache>")
            .finish()
    }
}

impl Data {
    /// Given the Data and a cache_http, returns the settings data
    pub fn settings_data(
        &self,
        serenity_context: serenity::all::Context,
    ) -> module_settings::types::SettingsData {
        module_settings::types::SettingsData {
            pool: self.pool.clone(),
            reqwest: self.reqwest.clone(),
            object_store: self.object_store.clone(),
            cache_http: botox::cache::CacheHttpImpl::from_ctx(&serenity_context),
            serenity_context,
        }
    }

    /// Given a settings data, return the silverpelt cache
    ///
    /// This is just a wrapper for <serenity_context>.data::<Data>().silverpelt_cache.clone()
    pub fn silverpelt_cache(
        settings_data: &module_settings::types::SettingsData,
    ) -> Arc<crate::cache::SilverpeltCache> {
        settings_data
            .serenity_context
            .data::<Data>()
            .silverpelt_cache
            .clone()
    }

    /// Given a settings data, return the data
    ///
    /// This is just a wrapper for settings_data.serenity_context.data::<Data>().clone()
    pub fn data(settings_data: &module_settings::types::SettingsData) -> Arc<Self> {
        settings_data.serenity_context.data::<Data>().clone()
    }
}

#[async_trait::async_trait]
pub trait Props
where
    Self: Send + Sync,
{
    /// Converts the props to std::any::Any
    fn as_any(&self) -> &(dyn std::any::Any + Send + Sync);

    /// The name of the service
    fn name(&self) -> String;

    /// If applicable, the shards associated with the service
    async fn shards(&self) -> Result<Vec<u16>, crate::Error>;

    /// If applicable, the shard count
    async fn shard_count(&self) -> Result<u16, crate::Error>;

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
    ) -> Result<(), crate::Error>;

    /// Total number of guilds
    ///
    /// Note that this statistic may not always be available, in such cases, 0 will be returned
    async fn total_guilds(&self) -> Result<u64, crate::Error>;

    /// Total number of users
    ///
    /// Note that this statistic may not always be available, in such cases, 0 will be returned
    async fn total_users(&self) -> Result<u64, crate::Error>;
}

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
    const SILVERPELT_CACHE_KEY_ID: usize = 0;

    /// Given the Data and a cache_http, returns the settings data
    pub fn settings_data(
        &self,
        cache_http: botox::cache::CacheHttpImpl,
    ) -> module_settings::types::SettingsData {
        module_settings::types::SettingsData {
            pool: self.pool.clone(),
            reqwest: self.reqwest.clone(),
            object_store: self.object_store.clone(),
            cache_http,
            extra_data: vec![(Self::SILVERPELT_CACHE_KEY_ID, self.silverpelt_cache.clone())],
        }
    }

    /// Given a settings data, return the silverpelt cache
    pub fn silverpelt_cache(
        settings_data: &module_settings::types::SettingsData,
    ) -> Arc<crate::cache::SilverpeltCache> {
        for (slot, data) in &settings_data.extra_data {
            if slot == &Self::SILVERPELT_CACHE_KEY_ID {
                return data
                    .clone()
                    .downcast::<crate::cache::SilverpeltCache>()
                    .expect("Silverpelt cache not found in settings data [downcast failure]");
            }
        }

        panic!("Silverpelt cache not found in settings data");
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

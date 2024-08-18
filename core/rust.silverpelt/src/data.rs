use serenity::all::{GuildId, UserId};
use splashcore_rs::{animusmagic::client::AnimusMagicRequestClient, objectstore::ObjectStore};
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

impl Data {
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
            permodule_executor: self.props.permodule_executor(),
        }
    }
}

#[async_trait::async_trait]
pub trait Props
where
    Self: Send + Sync,
{
    /// Converts the props to std::any::Any
    fn as_any(&self) -> &(dyn std::any::Any + Send + Sync);

    /// Returns the underlying client for animus magic
    fn underlying_am_client(&self) -> Result<Box<dyn AnimusMagicRequestClient>, crate::Error>;

    /// Returns the per module executor of the context
    fn permodule_executor(
        &self,
    ) -> Box<dyn splashcore_rs::permodule_functions::PermoduleFunctionExecutor>;

    /// Adds a permodule function to the executor
    fn add_permodule_function(
        &self,
        module: &str,
        function: &str,
        func: splashcore_rs::permodule_functions::ToggleFunc,
    );

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

    /// Reset the can_use_bot whitelist
    async fn reset_can_use_bot(&self) -> Result<(), crate::Error>;

    /// Returns if a user is whitelisted to use the bot
    async fn is_whitelisted(
        &self,
        guild_id: Option<GuildId>,
        user_id: UserId,
    ) -> Result<bool, crate::Error>;

    /// Returns the maintenace message for the bot
    fn maint_message<'a>(&self) -> poise::CreateReply<'a>;
}

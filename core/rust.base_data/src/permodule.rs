use async_trait::async_trait;
use dashmap::DashMap;
use futures_util::future::BoxFuture;
use splashcore_rs::value::Value;

pub type ToggleFunc = Box<
    dyn Send
        + Sync
        + for<'a> Fn(
            &'a botox::cache::CacheHttpImpl,
            &'a indexmap::IndexMap<String, Value>, // Options sent
        ) -> BoxFuture<'a, Result<(), crate::Error>>,
>;

/// In order to allow modules to implement their own internal caches/logic without polluting the animus magic protocol,
/// we implement PERMODULE_FUNCTIONS which any module can register/add on to
///
/// Format of a permodule toggle is (module_name, toggle)
pub type PermoduleFunctionMap = DashMap<(String, String), ToggleFunc>;

#[async_trait]
pub trait PermoduleFunctionExecutor: Send + Sync {
    async fn execute_permodule_function(
        &self,
        cache_http: &botox::cache::CacheHttpImpl,
        module: &str,
        function: &str,
        arguments: &indexmap::IndexMap<String, Value>,
    ) -> Result<(), crate::Error>;
}

// Dummy PermoduleFunctionExecutor
pub struct DummyPermoduleFunctionExecutor;

#[async_trait]
impl PermoduleFunctionExecutor for DummyPermoduleFunctionExecutor {
    async fn execute_permodule_function(
        &self,
        _cache_http: &botox::cache::CacheHttpImpl,
        _module: &str,
        _function: &str,
        _arguments: &indexmap::IndexMap<String, Value>,
    ) -> Result<(), crate::Error> {
        Ok(())
    }
}

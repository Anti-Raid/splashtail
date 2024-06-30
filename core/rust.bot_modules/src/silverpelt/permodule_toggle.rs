use dashmap::DashMap;
use futures::future::BoxFuture;
use once_cell::sync::Lazy;
use splashcore_rs::value::Value;

pub type ToggleFunc = Box<
    dyn Send
        + Sync
        + for<'a> Fn(
            &'a botox::cache::CacheHttpImpl,
            &'a indexmap::IndexMap<String, Value>, // Options sent
        ) -> BoxFuture<'a, Result<(), crate::Error>>,
>;

// In order to allow modules to implement their own internal caches/logic without polluting the animus magic protocol,
// we implement PERMODULE_FUNCTIONS which any module can register/add on to
//
// Format of a permodule toggle is (module_name, toggle)
pub static PERMODULE_FUNCTIONS: Lazy<DashMap<(String, String), ToggleFunc>> =
    Lazy::new(DashMap::new);

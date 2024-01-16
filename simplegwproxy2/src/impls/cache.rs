use std::sync::Arc;

use serenity::all::{Cache, CacheHttp, Http};

/// A Simple struct that implements the CacheHttp trait because serenity can't seem to keep this stable
/// 
/// Unlike botv2, this struct also includes a shard_manager field to allow easy access to the shard manager
#[derive(Debug, Clone)]
pub struct CacheHttpImpl {
    pub cache: Arc<Cache>,
    pub http: Arc<Http>,
    pub shard_manager: Arc<serenity::all::ShardManager>,
}

impl CacheHttp for CacheHttpImpl {
    fn http(&self) -> &Http {
        &self.http
    }

    fn cache(&self) -> Option<&Arc<Cache>> {
        Some(&self.cache)
    }
}

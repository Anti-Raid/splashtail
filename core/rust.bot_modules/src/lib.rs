pub mod modules;
pub mod silverpelt;

pub type Context<'a> = poise::Context<'a, Data, Error>;

pub use base_data::{Data, Error};
pub use botox::cache::CacheHttpImpl;
pub use silverpelt::silverpelt_cache::SILVERPELT_CACHE;
pub use splashcore_rs::value::Value;

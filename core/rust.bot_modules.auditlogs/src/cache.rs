use moka::future::Cache;
use serenity::all::GuildId;
use sqlx::types::Uuid;
use sqlx::PgPool;
use std::sync::Arc;
use std::sync::LazyLock;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Sink {
    pub id: Uuid,
    pub sink: String,
    pub events: Option<Vec<String>>,
    pub template: Option<String>,
}

pub static SINKS_CACHE: LazyLock<Cache<GuildId, Arc<Vec<Sink>>>> = LazyLock::new(|| {
    Cache::builder()
        .support_invalidation_closures()
        .time_to_idle(Duration::from_secs(60 * 5)) // Expire the audit log sink cache after 5 minutes
        .build()
});

pub async fn get_sinks(guild_id: GuildId, pool: &PgPool) -> Result<Arc<Vec<Sink>>, sqlx::Error> {
    if let Some(sinks) = SINKS_CACHE.get(&guild_id).await {
        return Ok(sinks.clone());
    }

    let sinks = sqlx::query_as!(
        Sink,
        "SELECT id, sink, events, template FROM auditlogs__sinks WHERE guild_id = $1 AND broken = false",
        guild_id.to_string(),
    )
    .fetch_all(pool)
    .await?;

    let sinks = Arc::new(sinks);

    SINKS_CACHE.insert(guild_id, sinks.clone()).await;

    Ok(sinks)
}

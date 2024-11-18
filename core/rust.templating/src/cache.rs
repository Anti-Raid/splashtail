use crate::GuildTemplate;
use moka::future::Cache;
use serenity::all::GuildId;
use std::sync::Arc;
use std::sync::LazyLock;
use std::time::Duration;

pub static TEMPLATES_CACHE: LazyLock<Cache<GuildId, Arc<Vec<GuildTemplate>>>> =
    LazyLock::new(|| {
        Cache::builder()
            .support_invalidation_closures()
            .time_to_idle(Duration::from_secs(60 * 5)) // Expire the audit log sink cache after 5 minutes
            .build()
    });

#[allow(dead_code)]
pub async fn get_all_guild_templates(
    guild_id: GuildId,
    pool: &sqlx::PgPool,
) -> Result<Arc<Vec<GuildTemplate>>, crate::Error> {
    if let Some(templates) = TEMPLATES_CACHE.get(&guild_id).await {
        return Ok(templates.clone());
    }

    let names = sqlx::query!(
        "SELECT name FROM guild_templates WHERE guild_id = $1",
        guild_id.to_string()
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| r.name)
    .collect::<Vec<String>>();

    let mut templates = Vec::new();

    for name in names {
        let template = crate::get_template(guild_id, &name, pool).await?;
        templates.push(template);
    }

    let templates = Arc::new(templates);

    // Store the templates in the cache
    TEMPLATES_CACHE.insert(guild_id, templates.clone()).await;

    Ok(templates)
}

#[allow(dead_code)]
pub async fn clear_template_cache(guild_id: GuildId) {
    TEMPLATES_CACHE.remove(&guild_id).await;
}

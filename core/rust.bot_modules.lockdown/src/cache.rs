use moka::future::Cache;
use serenity::all::GuildId;
use std::collections::HashSet;
use std::sync::{Arc, LazyLock};

pub struct GuildLockdownSettings {
    pub member_roles: HashSet<serenity::all::RoleId>,
    pub require_correct_layout: bool,
}

impl Default for GuildLockdownSettings {
    fn default() -> Self {
        Self {
            member_roles: HashSet::new(),
            require_correct_layout: true,
        }
    }
}

pub static GUILD_LOCKDOWN_SETTINGS: LazyLock<Cache<GuildId, Arc<GuildLockdownSettings>>> =
    LazyLock::new(|| Cache::builder().support_invalidation_closures().build());

pub async fn get_guild_lockdown_settings(
    pool: &sqlx::PgPool,
    guild_id: GuildId,
) -> Result<Arc<GuildLockdownSettings>, silverpelt::Error> {
    if let Some(settings) = GUILD_LOCKDOWN_SETTINGS.get(&guild_id).await {
        Ok(settings.clone())
    } else {
        let settings = match sqlx::query!(
            "SELECT member_roles, require_correct_layout FROM lockdown__guilds WHERE guild_id = $1",
            guild_id.to_string(),
        )
        .fetch_optional(pool)
        .await?
        {
            Some(settings) => {
                let member_roles = settings
                    .member_roles
                    .iter()
                    .map(|r| r.parse().unwrap())
                    .collect();

                let settings = GuildLockdownSettings {
                    member_roles,
                    require_correct_layout: settings.require_correct_layout,
                };

                Arc::new(settings)
            }
            None => Arc::new(GuildLockdownSettings::default()),
        };

        GUILD_LOCKDOWN_SETTINGS
            .insert(guild_id, settings.clone())
            .await;

        GUILD_LOCKDOWN_SETTINGS
            .insert(guild_id, settings.clone())
            .await;

        Ok(settings)
    }
}

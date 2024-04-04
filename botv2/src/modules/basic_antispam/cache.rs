use once_cell::sync::Lazy;
use moka::future::Cache;
use futures::future::FutureExt;

#[derive(Debug, Clone)]
pub struct BasicAntispamConfig {
    pub anti_invite: bool,
    pub anti_everyone: bool,
    pub minimum_account_age: Option<i64>,
    pub maximum_account_age: Option<i64>, // Not sure why you'd ever want this, but it's here
}

impl Default for BasicAntispamConfig {
    fn default() -> Self {
        Self {
            anti_invite: false,
            anti_everyone: true,
            minimum_account_age: None,
            maximum_account_age: None,
        }
    }
}

pub static BASIC_ANTISPAM_CONFIG_CACHE: Lazy<Cache<serenity::all::GuildId, BasicAntispamConfig>> =
    Lazy::new(|| Cache::builder().support_invalidation_closures().build());

pub async fn setup_cache_initial(data: &sqlx::PgPool) -> Result<(), crate::Error> {
    let config = sqlx::query!(
        "SELECT guild_id, anti_invite, anti_everyone, minimum_account_age, maximum_account_age FROM basic_antispam__options",
    )
    .fetch_all(data)
    .await?;

    for row in config {
        let guild_id = row.guild_id.parse::<serenity::all::GuildId>()?;
        let anti_invite = row.anti_invite;
        let anti_everyone = row.anti_everyone;
        let minimum_account_age = row.minimum_account_age;
        let maximum_account_age = row.maximum_account_age;

        BASIC_ANTISPAM_CONFIG_CACHE.insert(guild_id, BasicAntispamConfig {
            anti_invite,
            anti_everyone,
            minimum_account_age,
            maximum_account_age,
        }).await;
    }

    Ok(())
}

pub async fn get_config(pool: &sqlx::PgPool, guild_id: serenity::all::GuildId) -> Result<BasicAntispamConfig, crate::Error> {
    if let Some(config) = BASIC_ANTISPAM_CONFIG_CACHE.get(&guild_id).await {
        Ok(config.clone())
    } else {
        let config = sqlx::query!(
            "SELECT anti_invite, anti_everyone, minimum_account_age, maximum_account_age FROM basic_antispam__options WHERE guild_id = $1",
            guild_id.to_string(),
        )
        .fetch_optional(pool)
        .await?;

        if let Some(config) = config {
            let anti_invite = config.anti_invite;
            let anti_everyone = config.anti_everyone;
            let minimum_account_age = config.minimum_account_age;
            let maximum_account_age = config.maximum_account_age;

            BASIC_ANTISPAM_CONFIG_CACHE.insert(guild_id, BasicAntispamConfig {
                anti_invite,
                anti_everyone,
                minimum_account_age,
                maximum_account_age,
            }).await;

            Ok(BasicAntispamConfig {
                anti_invite,
                anti_everyone,
                minimum_account_age,
                maximum_account_age,
            })
        } else {
            let bas_cfg = BasicAntispamConfig::default();

            BASIC_ANTISPAM_CONFIG_CACHE.insert(guild_id, bas_cfg.clone()).await;

            Ok(bas_cfg)
        }
    }
}

pub async fn setup_am_toggle(_data: &sqlx::PgPool) -> Result<(), crate::Error> {
    async fn clear(options: &indexmap::IndexMap<String, serde_cbor::Value>) -> Result<(), crate::Error> {
        let Some(serde_cbor::Value::Text(guild_id)) = options.get("gulld_id") else {
            return Err("No guild_id provided".into());
        };

        let guild_id = guild_id.parse::<serenity::all::GuildId>()?;

        BASIC_ANTISPAM_CONFIG_CACHE.remove(&guild_id).await;

        Ok(())
    }

    crate::ipc::animus_magic::bot::dynamic::PERMODULE_CACHE_TOGGLES
        .insert(("basic_antispam".to_string(), "clear".to_string()), Box::new(
            move |options| clear(options).boxed()
        ));

    Ok(())
}
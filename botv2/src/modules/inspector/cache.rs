use super::types::{DehoistOptions, FakeBotDetectionOptions, GuildProtectionOptions};
use dashmap::DashMap;
use futures::future::FutureExt;
use moka::future::Cache;
use once_cell::sync::Lazy;
use serenity::all::UserId;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FakeBots {
    pub bot_id: UserId,
    pub name: String,
    pub official_bot_ids: Vec<UserId>,
}

pub static FAKE_BOTS_CACHE: Lazy<DashMap<UserId, FakeBots>> = Lazy::new(DashMap::new);

pub async fn setup_fake_bots(data: &crate::Data) -> Result<(), crate::Error> {
    // Clear the cache
    FAKE_BOTS_CACHE.clear();

    let fake_bots =
        sqlx::query!("SELECT bot_id, name, official_bot_ids FROM inspector__fake_bots",)
            .fetch_all(&data.pool)
            .await?;

    for row in fake_bots {
        let bot_id = row.bot_id.parse::<UserId>()?;
        let name = row.name.to_lowercase();
        let official_bot_ids = row
            .official_bot_ids
            .iter()
            .map(|id| id.parse::<UserId>())
            .collect::<Result<Vec<UserId>, _>>()?;

        FAKE_BOTS_CACHE.insert(
            bot_id,
            FakeBots {
                bot_id,
                name,
                official_bot_ids,
            },
        );
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub struct BasicAntispamConfig {
    pub anti_invite: Option<i32>, // None = disabled, Some(<stings>) othersise
    pub anti_everyone: Option<i32>, // None = disabled, Some(<stings>) othersise
    pub fake_bot_detection: FakeBotDetectionOptions,
    pub guild_protection: GuildProtectionOptions,
    pub hoist_detection: DehoistOptions,
    pub minimum_account_age: Option<i64>,
    pub maximum_account_age: Option<i64>, // Not sure why you'd ever want this, but it's here
    pub sting_retention: i32,             // Number of seconds to keep stings for
}

impl Default for BasicAntispamConfig {
    fn default() -> Self {
        Self {
            anti_invite: Some(0),
            anti_everyone: Some(0),
            fake_bot_detection: FakeBotDetectionOptions::NORMALIZE_NAMES
                | FakeBotDetectionOptions::EXACT_NAME_CHECK
                | FakeBotDetectionOptions::SIMILAR_NAME_CHECK, // The default checks should protect against most case of scam 'dyno' bot nukes
            guild_protection: GuildProtectionOptions::DISABLED, // Many people dont want antiraid constantly monitoring their servers name
            hoist_detection: DehoistOptions::STRIP_SPECIAL_CHARS_STARTSWITH
                | DehoistOptions::STRIP_NON_ASCII,
            minimum_account_age: None,
            maximum_account_age: None,
            sting_retention: 60 * 60, // one hour retention
        }
    }
}

pub static BASIC_ANTISPAM_CONFIG_CACHE: Lazy<Cache<serenity::all::GuildId, BasicAntispamConfig>> =
    Lazy::new(|| Cache::builder().support_invalidation_closures().build());

pub async fn setup_cache_initial(data: &sqlx::PgPool) -> Result<(), crate::Error> {
    let config = sqlx::query!(
        "SELECT guild_id, anti_invite, anti_everyone, fake_bot_detection, guild_protection, hoist_detection, minimum_account_age, maximum_account_age, sting_retention FROM inspector__options",
    )
    .fetch_all(data)
    .await?;

    for row in config {
        let guild_id = row.guild_id.parse::<serenity::all::GuildId>()?;

        BASIC_ANTISPAM_CONFIG_CACHE
            .insert(
                guild_id,
                BasicAntispamConfig {
                    anti_invite: row.anti_invite,
                    anti_everyone: row.anti_everyone,
                    fake_bot_detection: FakeBotDetectionOptions::from_bits_truncate(
                        row.fake_bot_detection,
                    ),
                    hoist_detection: DehoistOptions::from_bits_truncate(row.hoist_detection),
                    guild_protection: GuildProtectionOptions::from_bits_truncate(
                        row.guild_protection,
                    ),
                    minimum_account_age: row.minimum_account_age,
                    maximum_account_age: row.maximum_account_age,
                    sting_retention: row.sting_retention,
                },
            )
            .await;
    }

    Ok(())
}

pub async fn get_config(
    pool: &sqlx::PgPool,
    guild_id: serenity::all::GuildId,
) -> Result<BasicAntispamConfig, crate::Error> {
    if let Some(config) = BASIC_ANTISPAM_CONFIG_CACHE.get(&guild_id).await {
        Ok(config.clone())
    } else {
        let row = sqlx::query!(
            "SELECT anti_invite, anti_everyone, fake_bot_detection, guild_protection, hoist_detection, minimum_account_age, maximum_account_age, sting_retention FROM inspector__options WHERE guild_id = $1",
            guild_id.to_string(),
        )
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            let bac = BasicAntispamConfig {
                anti_invite: row.anti_invite,
                anti_everyone: row.anti_everyone,
                fake_bot_detection: FakeBotDetectionOptions::from_bits_truncate(
                    row.fake_bot_detection,
                ),
                hoist_detection: DehoistOptions::from_bits_truncate(row.hoist_detection),
                guild_protection: GuildProtectionOptions::from_bits_truncate(row.guild_protection),
                minimum_account_age: row.minimum_account_age,
                maximum_account_age: row.maximum_account_age,
                sting_retention: row.sting_retention,
            };

            BASIC_ANTISPAM_CONFIG_CACHE
                .insert(guild_id, bac.clone())
                .await;

            Ok(bac)
        } else {
            let bas_cfg = BasicAntispamConfig::default();

            BASIC_ANTISPAM_CONFIG_CACHE
                .insert(guild_id, bas_cfg.clone())
                .await;

            Ok(bas_cfg)
        }
    }
}

pub async fn setup_am_toggle(_pool: &sqlx::PgPool) -> Result<(), crate::Error> {
    async fn clear(
        options: &indexmap::IndexMap<String, serde_cbor::Value>,
    ) -> Result<(), crate::Error> {
        let Some(serde_cbor::Value::Text(guild_id)) = options.get("gulld_id") else {
            return Err("No guild_id provided".into());
        };

        let guild_id = guild_id.parse::<serenity::all::GuildId>()?;

        BASIC_ANTISPAM_CONFIG_CACHE.remove(&guild_id).await;

        Ok(())
    }

    crate::ipc::animus_magic::bot::dynamic::PERMODULE_FUNCTIONS.insert(
        ("basic_antispam".to_string(), "clear".to_string()),
        Box::new(move |_, options| clear(options).boxed()),
    );

    Ok(())
}

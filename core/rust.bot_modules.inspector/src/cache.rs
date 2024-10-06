use super::types::{
    AutoResponseMemberJoinOptions, DehoistOptions, FakeBotDetectionOptions, GuildProtectionOptions,
};
use dashmap::DashMap;
use moka::future::Cache;
use serenity::all::UserId;
use std::sync::LazyLock;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FakeBots {
    pub bot_id: UserId,
    pub name: String,
    pub official_bot_ids: Vec<UserId>,
}

pub static FAKE_BOTS_CACHE: LazyLock<DashMap<UserId, FakeBots>> = LazyLock::new(DashMap::new);

pub async fn setup_fake_bots_cache(pool: &sqlx::PgPool) -> Result<(), silverpelt::Error> {
    let fake_bots =
        sqlx::query!("SELECT bot_id, name, official_bot_ids FROM inspector__fake_bots",)
            .fetch_all(pool)
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
pub struct InspectorGlobalOptions {
    pub fake_bot_detection: FakeBotDetectionOptions,
    pub guild_protection: GuildProtectionOptions,
    pub auto_response_memberjoin: AutoResponseMemberJoinOptions,
    pub hoist_detection: DehoistOptions,
    pub minimum_account_age: Option<i64>,
    pub maximum_account_age: Option<i64>, // Not sure why you'd ever want this, but it's here
}

impl Default for InspectorGlobalOptions {
    fn default() -> Self {
        Self {
            fake_bot_detection: FakeBotDetectionOptions::NORMALIZE_NAMES
                | FakeBotDetectionOptions::EXACT_NAME_CHECK
                | FakeBotDetectionOptions::SIMILAR_NAME_CHECK, // The default checks should protect against most case of scam 'dyno' bot nukes
            guild_protection: GuildProtectionOptions::DISABLED, // Needs extra setup
            auto_response_memberjoin: AutoResponseMemberJoinOptions::DISABLED, // This needs to be enabled/disabled when theres an actual problem
            hoist_detection: DehoistOptions::STRIP_SPECIAL_CHARS_STARTSWITH
                | DehoistOptions::STRIP_NON_ASCII,
            minimum_account_age: None,
            maximum_account_age: None,
        }
    }
}

pub static INSPECTOR_GLOBAL_OPTIONS_CACHE: LazyLock<
    Cache<serenity::all::GuildId, InspectorGlobalOptions>,
> = LazyLock::new(|| Cache::builder().support_invalidation_closures().build());

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct InspectorSpecificOptions {
    pub id: sqlx::types::uuid::Uuid,
    pub anti_invite: Option<i32>, // None = disabled, Some(<stings>) othersise
    pub anti_everyone: Option<i32>, // None = disabled, Some(<stings>) othersise
    pub sting_retention: i32,     // Number of seconds to keep stings for
    pub modifier: Vec<splashcore_rs::modifier::Modifier>,
}

impl Default for InspectorSpecificOptions {
    fn default() -> Self {
        Self {
            id: sqlx::types::uuid::Uuid::new_v4(),
            anti_invite: Some(0),
            anti_everyone: Some(0),
            sting_retention: 60 * 60, // one hour retention
            modifier: vec![],
        }
    }
}

impl InspectorSpecificOptions {
    /// Returns the best value for ``anti_invite`` given a full list of them
    pub fn get<T: Ord>(
        opts: &[Self],
        val_fn: fn(&Self) -> Option<T>,
        user_id: serenity::all::UserId,
        channel_id: Option<serenity::all::ChannelId>,
    ) -> Option<T> {
        // Create a matcher that matches the user_id
        let mut matcher = splashcore_rs::modifier::ModifierMatcher::default();
        matcher.add_user_id(user_id);

        if let Some(channel_id) = channel_id {
            matcher.add_channel_id(channel_id);
        }

        matcher.add_variable("source".to_string(), "inspector".to_string());

        let mut best = (val_fn(&InspectorSpecificOptions::default()), -1);
        for opt in opts.iter() {
            let matches = matcher.match_modifiers(opt.modifier.clone());

            // Go over the matches and check if any have a greater specificity than best
            // If they do, they become the new best
            // If they are equal, the highesst value is chosen
            for mod_match in matches {
                let specificity = mod_match.specificity();

                if specificity > best.1 || (specificity == best.1 && val_fn(opt) > best.0) {
                    best = (val_fn(opt), specificity);
                }
            }
        }

        best.0
    }
}

pub static INSPECTOR_SPECIFIC_OPTIONS_CACHE: LazyLock<
    Cache<serenity::all::GuildId, Vec<InspectorSpecificOptions>>,
> = LazyLock::new(|| Cache::builder().support_invalidation_closures().build());

pub async fn get_global_config(
    pool: &sqlx::PgPool,
    guild_id: serenity::all::GuildId,
) -> Result<InspectorGlobalOptions, silverpelt::Error> {
    if let Some(config) = INSPECTOR_GLOBAL_OPTIONS_CACHE.get(&guild_id).await {
        Ok(config.clone())
    } else {
        let row = sqlx::query!(
            "SELECT fake_bot_detection, guild_protection, auto_response_memberjoin, hoist_detection, minimum_account_age, maximum_account_age FROM inspector__global_options WHERE guild_id = $1",
            guild_id.to_string(),
        )
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            let bac = InspectorGlobalOptions {
                fake_bot_detection: FakeBotDetectionOptions::from_bits_truncate(
                    row.fake_bot_detection,
                ),
                hoist_detection: DehoistOptions::from_bits_truncate(row.hoist_detection),
                guild_protection: GuildProtectionOptions::from_bits_truncate(row.guild_protection),
                auto_response_memberjoin: AutoResponseMemberJoinOptions::from_bits_truncate(
                    row.auto_response_memberjoin,
                ),
                minimum_account_age: row.minimum_account_age,
                maximum_account_age: row.maximum_account_age,
            };

            INSPECTOR_GLOBAL_OPTIONS_CACHE
                .insert(guild_id, bac.clone())
                .await;

            Ok(bac)
        } else {
            let bas_cfg = InspectorGlobalOptions::default();

            INSPECTOR_GLOBAL_OPTIONS_CACHE
                .insert(guild_id, bas_cfg.clone())
                .await;

            Ok(bas_cfg)
        }
    }
}

pub async fn get_specific_configs(
    pool: &sqlx::PgPool,
    guild_id: serenity::all::GuildId,
) -> Result<Vec<InspectorSpecificOptions>, silverpelt::Error> {
    if let Some(config) = INSPECTOR_SPECIFIC_OPTIONS_CACHE.get(&guild_id).await {
        Ok(config.clone())
    } else {
        let row = sqlx::query!(
            "SELECT id, anti_invite, anti_everyone, sting_retention, modifier FROM inspector__specific_options WHERE guild_id = $1",
            guild_id.to_string(),
        )
        .fetch_all(pool)
        .await?;

        let mut v = Vec::new();

        for row in row {
            v.push(InspectorSpecificOptions {
                id: row.id,
                anti_invite: row.anti_invite,
                anti_everyone: row.anti_everyone,
                sting_retention: row.sting_retention,
                modifier: {
                    let mut modifiers = vec![];

                    for modifier in row.modifier {
                        match splashcore_rs::modifier::Modifier::from_repr(&modifier) {
                            Ok(modifier) => {
                                modifiers.push(modifier);
                            }
                            Err(_) => {
                                log::warn!("Invalid modifier: {}", modifier);
                                continue;
                            }
                        }
                    }

                    modifiers
                },
            });
        }

        INSPECTOR_SPECIFIC_OPTIONS_CACHE
            .insert(guild_id, v.clone())
            .await;
        Ok(v)
    }
}

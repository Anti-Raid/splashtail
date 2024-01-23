use poise::serenity_prelude::CreateMessage;

use crate::impls::target_types::TargetType;

struct EntityData {
    typ: TargetType,
    id: String,
    username: String,
}

pub async fn premium_remove(
    pool: &sqlx::PgPool,
    cache_http: &crate::impls::cache::CacheHttpImpl,
) -> Result<(), crate::Error> {
    let res = sqlx::query!(
        "
        SELECT id, start_premium_period, premium_period_length FROM guilds 
		WHERE (
			premium_tier IS NOT NULL 
			AND (start_premium_period + premium_period_length) < NOW()
		)
        "
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Error while checking for expired premium users: {}", e))?;

    let mut data = vec![];
    for row in res {
        let id = row.id;

        data.push(EntityData {
            typ: TargetType::Guild,
            id: id.to_string(),
            username: id.to_string(), // TODO: Get guild name
        });
    }

    for entity in data {
        log::info!("Removing premium from {} {}", entity.typ, entity.id);

        match entity.typ {
            TargetType::Guild => {
                sqlx::query!(
                    "UPDATE guilds SET premium_tier = NULL WHERE id = $1",
                    entity.id
                )
                .execute(pool)
                .await
                .map_err(|e| {
                    format!(
                        "Error while removing premium from guild {}: {}",
                        entity.id, e
                    )
                })?;
            }
            TargetType::User => todo!("User premium removal"),
        }

        let msg = format!(
            "{} {} ({}) has been removed from the premium list because it is not/no longer approved or certified.", 
            entity.typ,
            entity.username,
            entity.id
        );

        crate::config::CONFIG
            .channels
            .mod_logs
            .send_message(&cache_http, CreateMessage::new().content(msg))
            .await?;
    }

    Ok(())
}

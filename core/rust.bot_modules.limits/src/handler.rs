use crate::strategy::Strategy;

use super::core::HandleModAction;
use silverpelt::Error;
use std::collections::HashSet;

const DEFAULT_EXPIRY: std::time::Duration = std::time::Duration::from_secs(60 * 5);

pub async fn handle_mod_action(
    ctx: &serenity::all::Context,
    ha: &HandleModAction,
) -> Result<(), Error> {
    let data = ctx.data::<silverpelt::data::Data>();

    // Check limits cache
    let guild_limits = super::cache::get_limits(&data, ha.guild_id).await?;

    if guild_limits.3.is_empty() {
        // No limits for this guild
        return Ok(());
    }

    let strategy_result = guild_limits
        .0
        .add_mod_action(&data, ha, &guild_limits)
        .await?;

    if strategy_result.stings > 0 {
        // Add to limits__user_stings
        let expiry_duration = strategy_result.expiry.unwrap_or({
            // Get the longest duration from expiries
            let max = strategy_result.expiries.iter().max();

            match max {
                Some((_, expiry)) => *expiry,
                None => DEFAULT_EXPIRY,
            }
        });

        sqlx::query!(
            "INSERT INTO limits__user_stings (guild_id, user_id, stings, hit_limits, causes, expiry) VALUES ($1, $2, $3, $4, $5, $6)",
            ha.guild_id.to_string(),
            ha.user_id.to_string(),
            strategy_result.stings as i32,
            &strategy_result.hit_limits,
            serde_json::json!({
                "action_data": ha.action_data,
                "target": ha.target,
                "causes": strategy_result.causes,
                "expiries": strategy_result.expiries
            }),
            chrono::Utc::now() + chrono::Duration::from_std(expiry_duration).unwrap()  
        )
        .execute(&data.pool)
        .await?;

        if silverpelt::module_config::is_module_enabled(
            &data.silverpelt_cache,
            &data.pool,
            ha.guild_id,
            "punishments",
        )
        .await?
        {
            // Create a new punishment
            log::info!(
                "Triggering punishment for user_id: {} due to hit limits {:?}",
                ha.user_id,
                strategy_result.hit_limits
            );

            match bot_modules_punishments::core::trigger_punishment(
                ctx,
                ha.guild_id,
                ha.user_id,
                HashSet::new(),
            )
            .await
            {
                Ok(()) => {
                    log::info!(
                        "Punishment triggered successfully with cause: {:?}",
                        strategy_result.causes
                    );
                }
                Err(e) => {
                    log::error!("Failed to trigger punishment: {:?}, cause: {:?}", e, strategy_result.causes);
                }
            }
        }
    }

    Ok(())
}

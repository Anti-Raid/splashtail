use super::core::HandleModAction;
use silverpelt::Error;

const DEFAULT_EXPIRY: std::time::Duration = std::time::Duration::from_secs(60 * 5);

/// Handles a mod action, returning true if the user has hit limits
pub(crate) async fn handle_mod_action(
    ctx: &serenity::all::Context,
    ha: &HandleModAction,
) -> Result<bool, Error> {
    // Bot itself performed action. Ignore
    if ha.user_id == ctx.cache.current_user().id {
        return Ok(false);
    }

    let data = ctx.data::<silverpelt::data::Data>();

    // Check limits cache
    let guild_limits = super::cache::get_limits(&data, ha.guild_id).await?;

    if guild_limits.2.is_empty() {
        // No limits for this guild
        return Ok(false);
    }

    let create_strategy = super::strategy::from_limit_strategy_string(&guild_limits.0.strategy)?;

    let strategy_result = create_strategy
        .add_mod_action(&data, ha, &guild_limits)
        .await?;

    if strategy_result.stings > 0 {
        // Add to stings db
        let expiry_duration = strategy_result.expiry.unwrap_or({
            // Get the longest duration from expiries
            let max = strategy_result.expiries.iter().max();

            match max {
                Some((_, expiry)) => *expiry,
                None => DEFAULT_EXPIRY,
            }
        });

        // Create a new sting
        log::info!(
            "Adding {} stings for user_id: {} due to hit limits {:?}",
            strategy_result.stings,
            ha.user_id,
            strategy_result.hit_limits
        );

        silverpelt::stings::StingCreate {
            module: "limits".to_string(),
            src: None,
            stings: strategy_result.stings,
            reason: Some(format!("Hit limits: {:?}", strategy_result.hit_limits)),
            void_reason: None,
            guild_id: ha.guild_id,
            creator: silverpelt::stings::StingTarget::System,
            target: silverpelt::stings::StingTarget::User(ha.user_id),
            state: silverpelt::stings::StingState::Active,
            duration: Some(expiry_duration),
            sting_data: Some(serde_json::json!({
                "action_data": ha.action_data,
                "target": ha.target,
                "causes": strategy_result.causes,
                "expiries": strategy_result.expiries,
                "hit_limits": strategy_result.hit_limits,
                "strategy": guild_limits.0.strategy,
            })),
        }
        .create(ctx.clone(), &data.pool)
        .await?;
    }

    Ok(strategy_result.stings > 0)
}

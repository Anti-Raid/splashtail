use super::core::Limit;
use silverpelt::Context;

pub async fn limits_autocomplete<'a>(
    ctx: Context<'_>,
    partial: &'a str,
) -> Vec<serenity::all::AutocompleteChoice<'a>> {
    // Fetch all limits available
    let data = ctx.data();

    let guild_id = ctx.guild_id();

    if guild_id.is_none() {
        return Vec::new();
    }

    let guild_id = guild_id.unwrap();

    let limits = Limit::guild(&data.pool, guild_id).await;

    if let Ok(limits) = limits {
        let mut choices = Vec::new();

        for limit in limits {
            if limit.limit_name.starts_with(partial) {
                choices.push(serenity::all::AutocompleteChoice::new(
                    limit.limit_name,
                    limit.limit_id,
                ));
            }
        }

        return choices;
    }

    Vec::new()
}

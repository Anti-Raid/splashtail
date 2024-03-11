use crate::{Context, Error};
use serenity::all::{UserId, CreateEmbed, ChannelId};
use crate::impls::utils::{get_icon_of_state, REPLACE_CHANNEL, parse_numeric_list_to_str, parse_duration_string, create_special_allocation_from_str};
use std::collections::HashMap;


/*
// Options that can be set when pruning a message
//
// Either one of PruneFrom or MaxMessages must be set. If both are set, then both will be used.
type MessagePruneOpts struct {
	UserID             string         `description:"If set, the user id to prune messages of"`
	Channels           []string       `description:"If set, the channels to prune messages from"`
	IgnoreErrors       bool           `description:"If set, ignore errors while pruning"`
	MaxMessages        int            `description:"The maximum number of messages to prune"`
	PruneFrom          timex.Duration `description:"If set, the time to prune messages from."`
	PerChannel         int            `description:"The minimum number of messages to prune per channel"`
	RolloverLeftovers  bool           `description:"Whether to attempt rollover of leftover message quota to another channels or not"`
	SpecialAllocations map[string]int `description:"Specific channel allocation overrides"`
}
*/

fn create_message_prune_serde(
    user_id: UserId,
    channels: Option<String>,
    ignore_errors: Option<bool>,
    max_messages: Option<i32>,
    prune_from: Option<String>,
    per_channel: Option<i32>,
    rollover_leftovers: Option<bool>,
    special_allocations: Option<String>,
) -> Result<serde_json::Value, Error> {
    let channels = if let Some(ref channels) = channels {
        parse_numeric_list_to_str::<ChannelId>(channels, &REPLACE_CHANNEL)?
    } else {
        vec![]
    };

    let prune_from = if let Some(ref prune_from) = prune_from {
        let (dur, unit) = parse_duration_string(prune_from)?;

        dur * unit.to_seconds()
    } else {
        0
    };

    let special_allocations = if let Some(ref special_allocations) = special_allocations {
        create_special_allocation_from_str(special_allocations)?
    } else {
        HashMap::new()
    };

    Ok(serde_json::json!(
        {
            "UserID": user_id.to_string(),
            "Channels": channels,
            "IgnoreErrors": ignore_errors.unwrap_or(false),
            "MaxMessages": max_messages.unwrap_or(1000),
            "PruneFrom": prune_from,
            "PerChannel": per_channel.unwrap_or(100),
            "RolloverLeftovers": rollover_leftovers.unwrap_or(false),
            "SpecialAllocations": special_allocations,
        }
    ))
}

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = "5",
    required_bot_permissions = "KICK_MEMBERS | MANAGE_MESSAGES",
)]
#[allow(clippy::too_many_arguments)]
pub async fn kick(
    ctx: Context<'_>,
    #[description = "The member to kick"] member: serenity::all::Member,
    #[description = "The reason for the kick"] reason: String,
    #[description = "Whether or not to prune messages"] prune_messages: Option<bool>,
    #[description = "Channels to prune from, otherwise will prune from all channels"] prune_channels: Option<String>,
    #[description = "Whether or not to avoid errors while pruning"] prune_ignore_errors: Option<bool>,
    #[description = "How many messages at maximum to prune"] prune_max_messages: Option<i32>,
    #[description = "The duration to prune from. Format: <number> days/hours/minutes/seconds"] prune_from: Option<String>,
    #[description = "The minimum number of messages to prune per channel"] prune_per_channel: Option<i32>,
    #[description = "Whether to attempt rollover of leftover message quota to another channels or not"] prune_rollover_leftovers: Option<bool>,
    #[description = "Specific channel allocation overrides"] prune_special_allocations: Option<String>,
) -> Result<(), Error> {
    let mut embed = CreateEmbed::new()
    .title("Kicking Member...")
    .description(format!("{} | Kicking Member...", get_icon_of_state("pending")));

    // Try kicking them
    member.kick_with_reason(&ctx.http(), &reason).await?;

    Ok(())
}

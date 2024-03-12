use crate::ipc::animus_magic::{
    client::{AnimusMessage, AnimusResponse},
    jobserver::{JobserverAnimusMessage, JobserverAnimusResponse},
};
use crate::ipc::argparse::MEWLD_ARGS;
use splashcore_rs::animusmagic_protocol::{AnimusTarget, default_request_timeout};
use splashcore_rs::animusmagic_ext::{AnimusAnyResponse, AnimusMagicClientExt};
use crate::{Context, Error};
use serenity::all::{User, UserId, GuildId, CreateEmbed, EditMember, EditMessage, ChannelId, Member, Message, Mentionable, Timestamp};
use serenity::utils::shard_id;
use std::sync::Arc;
use poise::CreateReply;
use crate::impls::utils::{get_icon_of_state, REPLACE_CHANNEL, parse_numeric_list_to_str, parse_duration_string, create_special_allocation_from_str, Unit};
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
#[allow(clippy::too_many_arguments)]
fn create_message_prune_serde(
    user_id: UserId,
    guild_id: GuildId,
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
            "ServerID": guild_id.to_string(),
            "Options": {
                "UserID": user_id.to_string(),
                "Channels": channels,
                "IgnoreErrors": ignore_errors.unwrap_or(false),
                "MaxMessages": max_messages.unwrap_or(1000),
                "PruneFrom": prune_from,
                "PerChannel": per_channel.unwrap_or(100),
                "RolloverLeftovers": rollover_leftovers.unwrap_or(false),
                "SpecialAllocations": special_allocations,    
            }
        }
    ))
}

fn username(
    m: &User
) -> String {
    if let Some(ref global_name) = m.global_name {
        global_name.to_string()
    } else {
        m.tag()
    }
}

fn to_log_format(
    moderator: &User,
    member: &User,
    reason: &str,
) -> String {
    format!("{} | Handled '{}' for reason '{}'", username(moderator), username(member), reason)
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
    #[description = "Whether or not to show prune status updates"] prune_debug: Option<bool>,
    #[description = "Channels to prune from, otherwise will prune from all channels"] prune_channels: Option<String>,
    #[description = "Whether or not to avoid errors while pruning"] prune_ignore_errors: Option<bool>,
    #[description = "How many messages at maximum to prune"] prune_max_messages: Option<i32>,
    #[description = "The duration to prune from. Format: <number> days/hours/minutes/seconds"] prune_from: Option<String>,
    #[description = "The minimum number of messages to prune per channel"] prune_per_channel: Option<i32>,
    #[description = "Whether to attempt rollover of leftover message quota to another channels or not"] prune_rollover_leftovers: Option<bool>,
    #[description = "Specific channel allocation overrides"] prune_special_allocations: Option<String>,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command can only be used in a guild".into());
    };

    let mut embed = CreateEmbed::new()
    .title("Kicking Member...")
    .description(format!("{} | Kicking {}", get_icon_of_state("pending"), member.mention()));

    let mut base_message = ctx.send(CreateReply::new().embed(embed)).await?.into_message().await?;

    let Some(author) = ctx.author_member().await else {
        return Err("This command can only be used in a guild".into());
    };

    // Try kicking them
    member.kick_with_reason(&ctx.http(), &to_log_format(&author.user, &member.user, &reason)).await?;

    // If we're pruning messages, do that
    if prune_messages.unwrap_or(false) {
        let prune_opts = create_message_prune_serde(
            member.user.id,
            guild_id,
            prune_channels,
            prune_ignore_errors,
            prune_max_messages,
            prune_from,
            prune_per_channel,
            prune_rollover_leftovers,
            prune_special_allocations,
        )?;

        let data = ctx.data();

        let task_id = match data
        .animus_magic_ipc
        .request(
            AnimusTarget::Jobserver,
            shard_id(guild_id, MEWLD_ARGS.shard_count),
            AnimusMessage::Jobserver(JobserverAnimusMessage::SpawnTask {
                name: "message_prune".to_string(),
                data: prune_opts,
                create: true,
                execute: true,
                task_id: None,
            }),
            default_request_timeout()
        )
        .await
        .map_err(|e| format!("Failed to create backup task: {}", e))?
        {
            AnimusAnyResponse::Response(AnimusResponse::Jobserver(JobserverAnimusResponse::SpawnTask { task_id })) => task_id,
            AnimusAnyResponse::Error(e) => return Err(format!("Failed to create backup task: {}", e.message).into()),
            _ => return Err("Invalid response from jobserver".into()),
        };

        embed = CreateEmbed::new()
        .title("Kicking Member...")
        .description(format!("{} | Kicking Member...", get_icon_of_state("pending")))
        .field("Pruning Messages", format!(":yellow_circle: Created task with Task ID of {}", task_id), false);

        base_message.edit(
            &ctx.http(),
            EditMessage::new()
            .embed(embed.clone())
        )
        .await?;

        let ch = crate::impls::cache::CacheHttpImpl {
            cache: ctx.serenity_context().cache.clone(),
            http: ctx.serenity_context().http.clone(),
        };

        async fn update_base_message(
            member: Arc<Member>,
            prune_debug: bool,
            cache_http: crate::impls::cache::CacheHttpImpl,
            mut base_message: Message,
            task: Arc<crate::jobserver::Task>,
        ) -> Result<(), Error> {
            let new_task_msg = crate::jobserver::taskpoll::embed(
                &task,
                vec![
                    CreateEmbed::default()
                    .title("Kicking Member...")
                    .description(format!("{} | Kicking {}", get_icon_of_state(&task.state), member.mention())),
                ],
                prune_debug,
            )?;

            let prefix_msg = new_task_msg.to_prefix_edit(EditMessage::default());

            base_message
                .edit(
                    &cache_http,
                    prefix_msg,
                )
                .await?;

            Ok(())
        }

        let marc = Arc::new(member);

        // Use jobserver::reactive to keep updating the message
        let prune_debug = prune_debug.unwrap_or(false);
        crate::jobserver::taskpoll::reactive(
            &ch,
            &ctx.data().pool,
            &task_id,
            |cache_http, task| {
                Box::pin(update_base_message(
                    marc.clone(),
                    prune_debug,
                    cache_http.clone(),
                    base_message.clone(),
                    task.clone(),
                ))
            },
            crate::jobserver::taskpoll::PollTaskOptions { interval: Some(1) },
        )
        .await?;
    } else {
        embed = CreateEmbed::new()
        .title("Kicking Member...")
        .description(format!("{} | Kicking {}", get_icon_of_state("completed"), member.mention()));

        base_message.edit(
            &ctx.http(),
            EditMessage::new()
            .embed(embed)
        )
        .await?;
    }

    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = "5",
    required_bot_permissions = "BAN_MEMBERS | MANAGE_MESSAGES",
)]
#[allow(clippy::too_many_arguments)]
pub async fn ban(
    ctx: Context<'_>,
    #[description = "The member to ban"] member: serenity::all::Member,
    #[description = "The reason for the ban"] reason: String,
    #[description = "Whether or not to prune messages"] prune_messages: Option<bool>,
    #[description = "Whether or not to show prune status updates"] prune_debug: Option<bool>,
    #[description = "How many messages to prune using discords autopruner [dmd] (days)"] prune_dmd: Option<u8>,
    #[description = "Channels to prune from, otherwise will prune from all channels"] prune_channels: Option<String>,
    #[description = "Whether or not to avoid errors while pruning"] prune_ignore_errors: Option<bool>,
    #[description = "How many messages at maximum to prune"] prune_max_messages: Option<i32>,
    #[description = "The duration to prune from. Format: <number> days/hours/minutes/seconds"] prune_from: Option<String>,
    #[description = "The minimum number of messages to prune per channel"] prune_per_channel: Option<i32>,
    #[description = "Whether to attempt rollover of leftover message quota to another channels or not"] prune_rollover_leftovers: Option<bool>,
    #[description = "Specific channel allocation overrides"] prune_special_allocations: Option<String>,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command can only be used in a guild".into());
    };

    let mut embed = CreateEmbed::new()
    .title("Banning Member...")
    .description(format!("{} | Banning {}", get_icon_of_state("pending"), member.mention()));

    let mut base_message = ctx.send(CreateReply::new().embed(embed)).await?.into_message().await?;

    // Try banning them
    let dmd = {
        if prune_messages.unwrap_or(false) {
            if let Some(prune_dmd) = prune_dmd {
                prune_dmd
            } else {
                0
            }
        } else {
            0
        }
    };

    let Some(author) = ctx.author_member().await else {
        return Err("This command can only be used in a guild".into());
    };

    member.ban_with_reason(ctx.http(), dmd, &to_log_format(&author.user, &member.user, &reason)).await?;

    // If we're pruning messages, do that
    if prune_messages.unwrap_or(false) {
        let prune_opts = create_message_prune_serde(
            member.user.id,
            guild_id,
            prune_channels,
            prune_ignore_errors,
            prune_max_messages,
            prune_from,
            prune_per_channel,
            prune_rollover_leftovers,
            prune_special_allocations,
        )?;

        let data = ctx.data();

        let task_id = match data
        .animus_magic_ipc
        .request(
            AnimusTarget::Jobserver,
            shard_id(guild_id, MEWLD_ARGS.shard_count),
            AnimusMessage::Jobserver(JobserverAnimusMessage::SpawnTask {
                name: "message_prune".to_string(),
                data: prune_opts,
                create: true,
                execute: true,
                task_id: None,
            }),
            default_request_timeout()
        )
        .await
        .map_err(|e| format!("Failed to create task: {}", e))?
        {
            AnimusAnyResponse::Response(AnimusResponse::Jobserver(JobserverAnimusResponse::SpawnTask { task_id })) => task_id,
            AnimusAnyResponse::Error(e) => return Err(format!("Failed to create task: {}", e.message).into()),
            _ => return Err("Invalid response from jobserver".into()),
        };

        embed = CreateEmbed::new()
        .title("Banning Member...")
        .description(format!("{} | Banning Member...", get_icon_of_state("pending")))
        .field("Pruning Messages", format!(":yellow_circle: Created task with Task ID of {}", task_id), false);

        base_message.edit(
            &ctx.http(),
            EditMessage::new()
            .embed(embed.clone())
        )
        .await?;

        let ch = crate::impls::cache::CacheHttpImpl {
            cache: ctx.serenity_context().cache.clone(),
            http: ctx.serenity_context().http.clone(),
        };

        async fn update_base_message(
            member: Arc<Member>,
            prune_debug: bool,
            cache_http: crate::impls::cache::CacheHttpImpl,
            mut base_message: Message,
            task: Arc<crate::jobserver::Task>,
        ) -> Result<(), Error> {
            let new_task_msg = crate::jobserver::taskpoll::embed(
                &task,
                vec![
                    CreateEmbed::default()
                    .title("Banning Member...")
                    .description(format!("{} | Banning {}", get_icon_of_state(&task.state), member.mention())),
                ],
                prune_debug,
            )?;

            let prefix_msg = new_task_msg.to_prefix_edit(EditMessage::default());

            base_message
                .edit(
                    &cache_http,
                    prefix_msg,
                )
                .await?;

            Ok(())
        }

        let marc = Arc::new(member);

        // Use jobserver::reactive to keep updating the message
        let prune_debug = prune_debug.unwrap_or(false);
        crate::jobserver::taskpoll::reactive(
            &ch,
            &ctx.data().pool,
            &task_id,
            |cache_http, task| {
                Box::pin(update_base_message(
                    marc.clone(),
                    prune_debug,
                    cache_http.clone(),
                    base_message.clone(),
                    task.clone(),
                ))
            },
            crate::jobserver::taskpoll::PollTaskOptions { interval: Some(1) },
        )
        .await?;
    } else {
        embed = CreateEmbed::new()
        .title("Banning Member...")
        .description(format!("{} | Banning {}", get_icon_of_state("completed"), member.mention()));

        base_message.edit(
            &ctx.http(),
            EditMessage::new()
            .embed(embed)
        )
        .await?;
    }

    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = "5",
    required_bot_permissions = "MODERATE_MEMBERS | MANAGE_MESSAGES",
)]
#[allow(clippy::too_many_arguments)]
pub async fn timeout(
    ctx: Context<'_>,
    #[description = "The member to timeout"] mut member: serenity::all::Member,
    #[description = "The duration of the timeout"] duration: String,
    #[description = "The reason for the timeout"] reason: String,
    #[description = "Whether or not to prune messages"] prune_messages: Option<bool>,
    #[description = "Whether or not to show prune status updates"] prune_debug: Option<bool>,
    #[description = "Channels to prune from, otherwise will prune from all channels"] prune_channels: Option<String>,
    #[description = "Whether or not to avoid errors while pruning"] prune_ignore_errors: Option<bool>,
    #[description = "How many messages at maximum to prune"] prune_max_messages: Option<i32>,
    #[description = "The duration to prune from. Format: <number> days/hours/minutes/seconds"] prune_from: Option<String>,
    #[description = "The minimum number of messages to prune per channel"] prune_per_channel: Option<i32>,
    #[description = "Whether to attempt rollover of leftover message quota to another channels or not"] prune_rollover_leftovers: Option<bool>,
    #[description = "Specific channel allocation overrides"] prune_special_allocations: Option<String>,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command can only be used in a guild".into());
    };

    let mut embed = CreateEmbed::new()
    .title("Timing out Member...")
    .description(format!("{} | Timing out {}", get_icon_of_state("pending"), member.mention()));

    let mut base_message = ctx.send(CreateReply::new().embed(embed)).await?.into_message().await?;

    // Try timing them out
    let (duration, unit) = parse_duration_string(&duration)?;

    // Ensure less than 28 days = 4 weeks = 672 hours = 40320 minutes = 2419200 seconds
    if duration > 7 && unit == Unit::Weeks {
        return Err("Timeout duration must be less than 28 days (4 weeks)".into());
    } else if duration > 28 && unit == Unit::Days {
        return Err("Timeout duration must be less than 28 days".into());
    } else if duration > 672 && unit == Unit::Hours {
        return Err("Timeout duration must be less than 28 days (672 hours)".into());
    } else if duration > 40320 && unit == Unit::Minutes {
        return Err("Timeout duration must be less than 28 days (40320 minutes)".into());
    } else if duration > 2419200 && unit == Unit::Seconds {
        return Err("Timeout duration must be less than 28 days (2419200 seconds)".into());
    }

    let Some(author) = ctx.author_member().await else {
        return Err("This command can only be used in a guild".into());
    };

    let time = (duration * unit.to_seconds() * 1000) as i64;
    member.edit(
        &ctx.http(),
        EditMember::new()
        .disable_communication_until(
            Timestamp::from_millis(
                Timestamp::now().unix_timestamp() * 1000 + time
            )?
        )
        .audit_log_reason(&to_log_format(&author.user, &member.user, &reason))
    ).await?;

    // If we're pruning messages, do that
    if prune_messages.unwrap_or(false) {
        let prune_opts = create_message_prune_serde(
            member.user.id,
            guild_id,
            prune_channels,
            prune_ignore_errors,
            prune_max_messages,
            prune_from,
            prune_per_channel,
            prune_rollover_leftovers,
            prune_special_allocations,
        )?;

        let data = ctx.data();

        let task_id = match data
        .animus_magic_ipc
        .request(
            AnimusTarget::Jobserver,
            shard_id(guild_id, MEWLD_ARGS.shard_count),
            AnimusMessage::Jobserver(JobserverAnimusMessage::SpawnTask {
                name: "message_prune".to_string(),
                data: prune_opts,
                create: true,
                execute: true,
                task_id: None,
            }),
            default_request_timeout()
        )
        .await
        .map_err(|e| format!("Failed to create backup task: {}", e))?
        {
            AnimusAnyResponse::Response(AnimusResponse::Jobserver(JobserverAnimusResponse::SpawnTask { task_id })) => task_id,
            AnimusAnyResponse::Error(e) => return Err(format!("Failed to create backup task: {}", e.message).into()),
            _ => return Err("Invalid response from jobserver".into()),
        };

        embed = CreateEmbed::new()
        .title("Timing Out Member...")
        .description(format!("{} | Timing Out Member...", get_icon_of_state("pending")))
        .field("Pruning Messages", format!(":yellow_circle: Created task with Task ID of {}", task_id), false);

        base_message.edit(
            &ctx.http(),
            EditMessage::new()
            .embed(embed.clone())
        )
        .await?;

        let ch = crate::impls::cache::CacheHttpImpl {
            cache: ctx.serenity_context().cache.clone(),
            http: ctx.serenity_context().http.clone(),
        };

        async fn update_base_message(
            member: Arc<Member>,
            prune_debug: bool,
            cache_http: crate::impls::cache::CacheHttpImpl,
            mut base_message: Message,
            task: Arc<crate::jobserver::Task>,
        ) -> Result<(), Error> {
            let new_task_msg = crate::jobserver::taskpoll::embed(
                &task,
                vec![
                    CreateEmbed::default()
                    .title("Timing Out Member...")
                    .description(format!("{} | Timing Out {}", get_icon_of_state(&task.state), member.mention())),
                ],
                prune_debug,
            )?;

            let prefix_msg = new_task_msg.to_prefix_edit(EditMessage::default());

            base_message
                .edit(
                    &cache_http,
                    prefix_msg,
                )
                .await?;

            Ok(())
        }

        let marc = Arc::new(member);

        // Use jobserver::reactive to keep updating the message
        let prune_debug = prune_debug.unwrap_or(false);
        crate::jobserver::taskpoll::reactive(
            &ch,
            &ctx.data().pool,
            &task_id,
            |cache_http, task| {
                Box::pin(update_base_message(
                    marc.clone(),
                    prune_debug,
                    cache_http.clone(),
                    base_message.clone(),
                    task.clone(),
                ))
            },
            crate::jobserver::taskpoll::PollTaskOptions { interval: Some(1) },
        )
        .await?;
    }

    Ok(())
}
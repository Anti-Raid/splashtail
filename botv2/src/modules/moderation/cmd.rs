use bothelpers::utils::{
    create_special_allocation_from_str, get_icon_of_state, parse_duration_string,
    parse_numeric_list_to_str, Unit, REPLACE_CHANNEL,
};
use crate::ipc::animus_magic::{
    client::{AnimusMessage, AnimusResponse},
    jobserver::{JobserverAnimusMessage, JobserverAnimusResponse},
};
use crate::ipc::argparse::MEWLD_ARGS;
use crate::{Context, Error};
use poise::CreateReply;
use serenity::all::{
    ChannelId, CreateEmbed, EditMember, EditMessage, GuildId, Mentionable, Message,
    Timestamp, User, UserId,
};
use serenity::utils::shard_id;
use splashcore_rs::animusmagic_ext::{AnimusAnyResponse, AnimusMagicClientExt};
use splashcore_rs::animusmagic_protocol::{default_request_timeout, AnimusTarget};
use std::collections::HashMap;
use std::sync::Arc;

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
    channels: &Option<String>,
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

fn username(m: &User) -> String {
    if let Some(ref global_name) = m.global_name {
        global_name.to_string()
    } else {
        m.tag()
    }
}

fn to_log_format(moderator: &User, member: &User, reason: &str) -> String {
    format!(
        "{} | Handled '{}' for reason '{}'",
        username(moderator),
        username(member),
        reason
    )
}

/// Prune messages from a user
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = "5",
    required_bot_permissions = "KICK_MEMBERS | MANAGE_MESSAGES"
)]
#[allow(clippy::too_many_arguments)]
pub async fn prune_user(
    ctx: Context<'_>,
    #[description = "The user to prune"] user: serenity::all::User,
    #[description = "The reason for the prune"] reason: String,
    #[description = "Number of stings to give. Defaults to configured base stings"] stings: Option<i32>,
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
        .title("Pruning User Messages...")
        .description(format!(
            "{} | Pruning User Messages {}",
            get_icon_of_state("pending"),
            user.mention()
        ));

    let mut base_message = ctx
        .send(CreateReply::new().embed(embed))
        .await?
        .into_message()
        .await?;

    let Some(author) = ctx.author_member().await else {
        return Err("This command can only be used in a guild".into());
    };

    let stings = stings.unwrap_or(1);

    if stings < 0 {
        return Err("Stings must be greater than or equal to 0".into());
    }

    let mut tx = ctx.data().pool.begin().await?;

    sqlx::query!(
        "INSERT INTO moderation__actions (guild_id, user_id, moderator, action, stings, reason) VALUES ($1, $2, $3, $4, $5, $6)",
        guild_id.to_string(),
        user.id.to_string(),
        author.user.id.to_string(),
        "prune",
        stings,
        reason,
    )
    .execute(&mut *tx)
    .await?;

    // If we're pruning messages, do that
    let prune_opts = create_message_prune_serde(
        user.id,
        guild_id,
        &prune_channels,
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
                data: prune_opts.clone(),
                create: true,
                execute: true,
                task_id: None,
                user_id: author.user.id.to_string(),
            }),
            default_request_timeout(),
        )
        .await
        .map_err(|e| format!("Failed to create backup task: {}", e))?
    {
        AnimusAnyResponse::Response(AnimusResponse::Jobserver(
            JobserverAnimusResponse::SpawnTask { task_id },
        )) => task_id,
        AnimusAnyResponse::Error(e) => {
            return Err(format!("Failed to create backup task: {}", e.message).into())
        }
        _ => return Err("Invalid response from jobserver".into()),
    };

    tx.commit().await?;

    // Send audit logs if Audit Logs module is enabled
    if crate::silverpelt::module_config::is_module_enabled(&ctx.data().pool, guild_id, "auditlogs").await? {
        let imap = indexmap::indexmap!{
            "log".to_string() => gwevent::core::Field {
                value: vec![to_log_format(&author.user, &user, &reason).into()],
                category: "log".to_string(),
            },
            "prune_opts".to_string() => gwevent::core::Field {
                value: vec![prune_opts.into()],
                category: "log".to_string(),
            },
            "channels".to_string() => gwevent::core::Field {
                value: vec![
                    if let Some(ref channels) = prune_channels {
                        parse_numeric_list_to_str::<ChannelId>(channels, &REPLACE_CHANNEL)?.into()
                    } else {
                        gwevent::field_type::FieldType::None
                    }
                ],
                category: "log".to_string(),
            },
        };

        crate::modules::auditlogs::events::dispatch_audit_log(
            ctx.serenity_context(),
            "AR/PruneMessageBegin",
            "(Anti-Raid) Prune Messages Begin",
            imap,
            guild_id
        )
        .await?;
    }

    embed = CreateEmbed::new()
        .title("Pruning User Messages...")
        .description(format!(
            "{} | Pruning User Messages...",
            get_icon_of_state("pending")
        ))
        .field(
            "Pruning Messages",
            format!(":yellow_circle: Created task with Task ID of {}", task_id),
            false,
        );

    base_message
        .edit(&ctx.http(), EditMessage::new().embed(embed.clone()))
        .await?;

    let ch = bothelpers::cache::CacheHttpImpl {
        cache: ctx.serenity_context().cache.clone(),
        http: ctx.serenity_context().http.clone(),
    };

    async fn update_base_message(
        user: Arc<User>,
        prune_debug: bool,
        cache_http: bothelpers::cache::CacheHttpImpl,
        mut base_message: Message,
        task: Arc<jobserver::Task>,
    ) -> Result<(), Error> {
        let new_task_msg = jobserver::taskpoll::embed(
            &crate::config::CONFIG.sites.api.get(),
            &task,
            vec![CreateEmbed::default()
                .title("Pruning User Messages...")
                .description(format!(
                    "{} | Pruning User Messages {}",
                    get_icon_of_state(&task.state),
                    user.mention()
                ))],
            prune_debug,
        )?;

        let prefix_msg = new_task_msg.to_prefix_edit(EditMessage::default());

        base_message.edit(&cache_http, prefix_msg).await?;

        Ok(())
    }

    let uarc = Arc::new(user);

    // Use jobserver::reactive to keep updating the message
    let prune_debug = prune_debug.unwrap_or(false);
    jobserver::taskpoll::reactive(
        &ch,
        &ctx.data().pool,
        &task_id,
        |cache_http, task| {
            Box::pin(update_base_message(
                uarc.clone(),
                prune_debug,
                cache_http.clone(),
                base_message.clone(),
                task.clone(),
            ))
        },
        jobserver::taskpoll::PollTaskOptions::default(),
    )
    .await?;

    Ok(())
}

/// Kicks a member from the server with optional purge/stinging abilities
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = "5",
    required_bot_permissions = "KICK_MEMBERS | MANAGE_MESSAGES"
)]
pub async fn kick(
    ctx: Context<'_>,
    #[description = "The member to kick"] member: serenity::all::Member,
    #[description = "The reason for the kick"] reason: String,
    #[description = "Number of stings to give. Defaults to configured base stings"] stings: Option<i32>,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command can only be used in a guild".into());
    };

    let mut embed = CreateEmbed::new()
        .title("Kicking Member...")
        .description(format!(
            "{} | Kicking {}",
            get_icon_of_state("pending"),
            member.mention()
        ));

    let mut base_message = ctx
        .send(CreateReply::new().embed(embed))
        .await?
        .into_message()
        .await?;

    let Some(author) = ctx.author_member().await else {
        return Err("This command can only be used in a guild".into());
    };

    // Try kicking them
    let stings = stings.unwrap_or(1);

    if stings < 0 {
        return Err("Stings must be greater than or equal to 0".into());
    }

    let mut tx = ctx.data().pool.begin().await?;

    sqlx::query!(
        "INSERT INTO moderation__actions (guild_id, user_id, moderator, action, stings, reason) VALUES ($1, $2, $3, $4, $5, $6)",
        guild_id.to_string(),
        member.user.id.to_string(),
        author.user.id.to_string(),
        "kick",
        stings,
        reason,
    )
    .execute(&mut *tx)
    .await?;

    // Send audit logs if Audit Logs module is enabled
    if crate::silverpelt::module_config::is_module_enabled(&ctx.data().pool, guild_id, "auditlogs").await? {
        let imap = indexmap::indexmap!{
            "target".to_string() => gwevent::core::Field {
                value: vec![member.user.clone().into()],
                category: "user".to_string(),
            },
            "moderator".to_string() => gwevent::core::Field {
                value: vec![author.user.clone().into()],
                category: "moderator".to_string(),
            },
            "reason".to_string() => gwevent::core::Field {
                value: vec![reason.clone().into()],
                category: "reason".to_string(),
            },
            "stings".to_string() => gwevent::core::Field {
                value: vec![stings.into()],
                category: "punishment".to_string(),
            },
            "log".to_string() => gwevent::core::Field {
                value: vec![to_log_format(&author.user, &member.user, &reason).into()],
                category: "log".to_string(),
            },
        };

        crate::modules::auditlogs::events::dispatch_audit_log(
            ctx.serenity_context(),
            "AR/KickMember",
            "(Anti-Raid) Kick Member",
            imap,
            guild_id
        )
        .await?;
    }

    member
        .kick_with_reason(
            &ctx.http(),
            &to_log_format(&author.user, &member.user, &reason),
        )
        .await?;

    tx.commit().await?;

    embed = CreateEmbed::new()
        .title("Kicking Member...")
        .description(format!(
            "{} | Kicked {}",
            get_icon_of_state("completed"),
            member.mention()
        ));

    base_message
        .edit(&ctx.http(), EditMessage::new().embed(embed))
        .await?;

    Ok(())
}

/// Bans a member from the server with optional purge/stinging abilities
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = "5",
    required_bot_permissions = "BAN_MEMBERS | MANAGE_MESSAGES"
)]
pub async fn ban(
    ctx: Context<'_>,
    #[description = "The member to ban"] member: serenity::all::User,
    #[description = "The reason for the ban"] reason: String,
    #[description = "Number of stings to give. Defaults to configured base stings"] stings: Option<i32>,
    #[description = "How many messages to prune using discords autopruner [dmd] (days)"] prune_dmd: Option<u8>,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command can only be used in a guild".into());
    };

    let mut embed = CreateEmbed::new()
        .title("Banning Member...")
        .description(format!(
            "{} | Banning {}",
            get_icon_of_state("pending"),
            member.mention()
        ));

    let mut base_message = ctx
        .send(CreateReply::new().embed(embed))
        .await?
        .into_message()
        .await?;

    // Try banning them
    let dmd = if let Some(prune_dmd) = prune_dmd {
        prune_dmd
    } else {
        0
    };

    let Some(author) = ctx.author_member().await else {
        return Err("This command can only be used in a guild".into());
    };

    let stings = stings.unwrap_or(1);

    if stings < 0 {
        return Err("Stings must be greater than or equal to 0".into());
    }

    let mut tx = ctx.data().pool.begin().await?;

    sqlx::query!(
        "INSERT INTO moderation__actions (guild_id, user_id, moderator, action, stings, reason) VALUES ($1, $2, $3, $4, $5, $6)",
        guild_id.to_string(),
        member.id.to_string(),
        author.user.id.to_string(),
        "ban",
        stings,
        reason,
    )
    .execute(&mut *tx)
    .await?;

    // Send audit logs if Audit Logs module is enabled
    if crate::silverpelt::module_config::is_module_enabled(&ctx.data().pool, guild_id, "auditlogs").await? {
        let imap = indexmap::indexmap!{
            "target".to_string() => gwevent::core::Field {
                value: vec![member.clone().into()],
                category: "user".to_string(),
            },
            "moderator".to_string() => gwevent::core::Field {
                value: vec![author.user.clone().into()],
                category: "moderator".to_string(),
            },
            "reason".to_string() => gwevent::core::Field {
                value: vec![reason.clone().into()],
                category: "reason".to_string(),
            },
            "stings".to_string() => gwevent::core::Field {
                value: vec![stings.into()],
                category: "punishment".to_string(),
            },
            "prune_dmd".to_string() => gwevent::core::Field {
                value: vec![dmd.into()],
                category: "log".to_string(),
            },
            "log".to_string() => gwevent::core::Field {
                value: vec![to_log_format(&author.user, &member, &reason).into()],
                category: "log".to_string(),
            },
        };

        crate::modules::auditlogs::events::dispatch_audit_log(
            ctx.serenity_context(),
            "AR/BanMember",
            "(Anti-Raid) Ban Member",
            imap,
            guild_id
        )
        .await?;
    }

    guild_id
    .ban_with_reason(
        ctx.http(),
        member.id,
        dmd,
        &to_log_format(&author.user, &member, &reason),
    )
    .await?;

    tx.commit().await?;

    embed = CreateEmbed::new()
        .title("Banning Member...")
        .description(format!(
            "{} | Banned {}",
            get_icon_of_state("completed"),
            member.mention()
        ));

    base_message
        .edit(&ctx.http(), EditMessage::new().embed(embed))
        .await?;

    Ok(())
}

/// Temporaily bans a member from the server with optional purge/stinging abilities
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = "5",
    required_bot_permissions = "BAN_MEMBERS | MANAGE_MESSAGES"
)]
pub async fn tempban(
    ctx: Context<'_>,
    #[description = "The member to ban"] member: serenity::all::User,
    #[description = "The reason for the ban"] reason: String,
    #[description = "Number of stings to give. Defaults to configured base stings"] stings: Option<i32>,
    #[description = "The duration of the ban"] duration: String,
    #[description = "How many messages to prune using discords autopruner [dmd] (days)"] prune_dmd: Option<u8>,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command can only be used in a guild".into());
    };

    let duration = parse_duration_string(&duration)?;

    let mut embed = CreateEmbed::new()
        .title("(Temporarily) Banning Member...")
        .description(format!(
            "{} | Banning {}",
            get_icon_of_state("pending"),
            member.mention()
        ));

    let mut base_message = ctx
        .send(CreateReply::new().embed(embed))
        .await?
        .into_message()
        .await?;

    // Try banning them
    let dmd = if let Some(prune_dmd) = prune_dmd {
        prune_dmd
    } else {
        0
    };

    let Some(author) = ctx.author_member().await else {
        return Err("This command can only be used in a guild".into());
    };

    let stings = stings.unwrap_or(1);

    if stings < 0 {
        return Err("Stings must be greater than or equal to 0".into());
    }

    let mut tx = ctx.data().pool.begin().await?;

    sqlx::query!(
        "INSERT INTO moderation__actions (guild_id, user_id, duration, moderator, action, stings, reason) VALUES ($1, $2, make_interval(secs => $3), $4, $5, $6, $7)",
        guild_id.to_string(),
        member.id.to_string(),
        (duration.0 * duration.1.to_seconds()) as f64,
        author.user.id.to_string(),
        "ban",
        stings,
        reason,
    )
    .execute(&mut *tx)
    .await?;

    // Send audit logs if Audit Logs module is enabled
    if crate::silverpelt::module_config::is_module_enabled(&ctx.data().pool, guild_id, "auditlogs").await? {
        let imap = indexmap::indexmap!{
            "target".to_string() => gwevent::core::Field {
                value: vec![member.clone().into()],
                category: "user".to_string(),
            },
            "moderator".to_string() => gwevent::core::Field {
                value: vec![author.user.clone().into()],
                category: "moderator".to_string(),
            },
            "reason".to_string() => gwevent::core::Field {
                value: vec![reason.clone().into()],
                category: "reason".to_string(),
            },
            "stings".to_string() => gwevent::core::Field {
                value: vec![stings.into()],
                category: "punishment".to_string(),
            },
            "log".to_string() => gwevent::core::Field {
                value: vec![to_log_format(&author.user, &member, &reason).into()],
                category: "log".to_string(),
            },
        };

        crate::modules::auditlogs::events::dispatch_audit_log(
            ctx.serenity_context(),
            "AR/BanMemberTemporary",
            "(Anti-Raid) Ban Member (Temporary)",
            imap,
            guild_id
        )
        .await?;
    }

    guild_id
    .ban_with_reason(
        ctx.http(),
        member.id,
        dmd,
        &to_log_format(&author.user, &member, &reason),
    )
    .await?;

    tx.commit().await?;

    embed = CreateEmbed::new()
        .title("(Temporarily) Banned Member...")
        .description(format!(
            "{} | Banned {}",
            get_icon_of_state("completed"),
            member.mention()
        ));

    base_message
        .edit(&ctx.http(), EditMessage::new().embed(embed))
        .await?;

    Ok(())
}

/// Unbans a member from the server with optional purge/stinging abilities
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = "5",
    required_bot_permissions = "BAN_MEMBERS | MANAGE_MESSAGES"
)]
pub async fn unban(
    ctx: Context<'_>,
    #[description = "The user to ban"] user: serenity::all::User,
    #[description = "The reason for the ban"] reason: String,
    #[description = "Number of stings to give. Defaults to 0"] stings: Option<i32>,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command can only be used in a guild".into());
    };

    let mut embed = CreateEmbed::new()
        .title("Unbanning Member...")
        .description(format!(
            "{} | Unbanning {}",
            get_icon_of_state("pending"),
            user.mention()
        ));

    let mut base_message = ctx
        .send(CreateReply::new().embed(embed))
        .await?
        .into_message()
        .await?;


    let Some(author) = ctx.author_member().await else {
        return Err("This command can only be used in a guild".into());
    };

    let stings = stings.unwrap_or(0);

    if stings < 0 {
        return Err("Stings must be greater than or equal to 0".into());
    }

    let mut tx = ctx.data().pool.begin().await?;

    sqlx::query!(
        "INSERT INTO moderation__actions (guild_id, user_id, moderator, action, stings, reason) VALUES ($1, $2, $3, $4, $5, $6)",
        guild_id.to_string(),
        user.id.to_string(),
        author.user.id.to_string(),
        "unban",
        stings,
        reason,
    )
    .execute(&mut *tx)
    .await?;

    // Send audit logs if Audit Logs module is enabled
    if crate::silverpelt::module_config::is_module_enabled(&ctx.data().pool, guild_id, "auditlogs").await? {
        let imap = indexmap::indexmap!{
            "target".to_string() => gwevent::core::Field {
                value: vec![user.clone().into()],
                category: "user".to_string(),
            },
            "moderator".to_string() => gwevent::core::Field {
                value: vec![author.user.clone().into()],
                category: "moderator".to_string(),
            },
            "reason".to_string() => gwevent::core::Field {
                value: vec![reason.clone().into()],
                category: "reason".to_string(),
            },
            "stings".to_string() => gwevent::core::Field {
                value: vec![stings.into()],
                category: "punishment".to_string(),
            },
            "log".to_string() => gwevent::core::Field {
                value: vec![to_log_format(&author.user, &user, &reason).into()],
                category: "log".to_string(),
            },
        };

        crate::modules::auditlogs::events::dispatch_audit_log(
            ctx.serenity_context(),
            "AR/UnbanMember",
            "(Anti-Raid) Unban Member",
            imap,
            guild_id
        )
        .await?;
    }

    ctx.http().remove_ban(guild_id, user.id, Some(&to_log_format(&author.user, &user, &reason))).await?;

    tx.commit().await?;

    embed = CreateEmbed::new()
        .title("Unbanning Member...")
        .description(format!(
            "{} | Unbanned {}",
            get_icon_of_state("completed"),
            user.mention()
        ));

    base_message
        .edit(&ctx.http(), EditMessage::new().embed(embed))
        .await?;

    Ok(())
}

/// Times out a member from the server with optional purge/stinging abilities
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = "5",
    required_bot_permissions = "MODERATE_MEMBERS | MANAGE_MESSAGES"
)]
pub async fn timeout(
    ctx: Context<'_>,
    #[description = "The member to timeout"] mut member: serenity::all::Member,
    #[description = "The duration of the timeout"] duration: String,
    #[description = "The reason for the timeout"] reason: String,
    #[description = "Number of stings to give. Defaults to configured base stings"] stings: Option<i32>,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command can only be used in a guild".into());
    };

    let mut embed = CreateEmbed::new()
        .title("Timing out Member...")
        .description(format!(
            "{} | Timing out {}",
            get_icon_of_state("pending"),
            member.mention()
        ));

    let mut base_message = ctx
        .send(CreateReply::new().embed(embed))
        .await?
        .into_message()
        .await?;

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

    let stings = stings.unwrap_or(1);

    if stings < 0 {
        return Err("Stings must be greater than or equal to 0".into());
    }

    let mut tx = ctx.data().pool.begin().await?;

    sqlx::query!(
        "INSERT INTO moderation__actions (guild_id, user_id, duration, moderator, action, stings, reason) VALUES ($1, $2, make_interval(secs => $3), $4, $5, $6, $7)",
        guild_id.to_string(),
        member.user.id.to_string(),
        time as f64,
        author.user.id.to_string(),
        "timeout",
        stings,
        reason,
    )
    .execute(&mut *tx)
    .await?;

    // Send audit logs if Audit Logs module is enabled
    if crate::silverpelt::module_config::is_module_enabled(&ctx.data().pool, guild_id, "auditlogs").await? {
        let imap = indexmap::indexmap!{
            "target".to_string() => gwevent::core::Field {
                value: vec![member.user.clone().into()],
                category: "user".to_string(),
            },
            "moderator".to_string() => gwevent::core::Field {
                value: vec![author.user.clone().into()],
                category: "moderator".to_string(),
            },
            "reason".to_string() => gwevent::core::Field {
                value: vec![reason.clone().into()],
                category: "reason".to_string(),
            },
            "stings".to_string() => gwevent::core::Field {
                value: vec![stings.into()],
                category: "punishment".to_string(),
            },
            "log".to_string() => gwevent::core::Field {
                value: vec![to_log_format(&author.user, &member.user, &reason).into()],
                category: "log".to_string(),
            },
        };

        crate::modules::auditlogs::events::dispatch_audit_log(
            ctx.serenity_context(),
            "AR/TimeoutMember",
            "(Anti-Raid) Timeout Member",
            imap,
            guild_id
        )
        .await?;
    }

    member
        .edit(
            ctx.http(),
            EditMember::new()
                .disable_communication_until(Timestamp::from_millis(
                    Timestamp::now().unix_timestamp() * 1000 + time,
                )?)
                .audit_log_reason(&to_log_format(&author.user, &member.user, &reason)),
        )
        .await?;

    tx.commit().await?;

    embed = CreateEmbed::new()
    .title("Unbanned Member...")
    .description(format!(
        "{} | Unbanning {}",
        get_icon_of_state("completed"),
        member.mention()
    ));

    base_message
    .edit(&ctx.http(), EditMessage::new().embed(embed))
    .await?;

    Ok(())
}
